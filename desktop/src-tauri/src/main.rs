#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod network;
mod vcam;

use capture::{CaptureReceiver, CaptureStorage};
use network::{AdbDeviceInfo, AdbManager, NetworkServer, PairingPayload, UdpVideoServer};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use vcam::{VirtualCameraDevice, VirtualCameraSink};

#[derive(Debug, Clone, Serialize)]
pub struct SessionStatusResponse {
    pub is_connected: bool,
    pub device_name: String,
    pub battery_pct: u8,
    pub is_charging: bool,
    pub fps: f32,
    pub bitrate_kbps: u32,
    pub latency_ms: u32,
    pub connection_mode: String,
    pub audio_level_rms: f32,
    pub has_audio: bool,
}

#[tauri::command]
async fn get_pairing_payload(server: tauri::State<'_, Arc<NetworkServer>>) -> Result<PairingPayload, String> {
    let mgr = server.get_pairing_manager();
    let pm = mgr.read().await;
    Ok(pm.generate_pairing_payload())
}

#[tauri::command]
async fn rotate_credentials(server: tauri::State<'_, Arc<NetworkServer>>) -> Result<PairingPayload, String> {
    let mgr = server.get_pairing_manager();
    let mut pm = mgr.write().await;
    pm.rotate_credentials();
    Ok(pm.generate_pairing_payload())
}

#[tauri::command]
async fn get_connection_status(server: tauri::State<'_, Arc<NetworkServer>>) -> Result<SessionStatusResponse, String> {
    let sess_arc = server.get_session();
    let sess_opt = sess_arc.read().await;
    if let Some(s) = sess_opt.as_ref() {
        let (fps, bitrate_kbps, latency_ms, battery_pct, is_charging, conn_mode) = if let Some(t) = &s.last_telemetry {
            (t.fps, t.bitrate_kbps, t.latency_ms, t.battery_pct, t.is_charging, format!("{:?}", t.connection_mode))
        } else {
            (60.0, 4800, 28, 85, false, format!("{:?}", s.connection_mode))
        };

        let dev_name = s.device_info.as_ref()
            .map(|d| d.client_name.clone())
            .unwrap_or_else(|| "Android Device".to_string());

        let has_audio = s.total_audio_frames > 0;
        let audio_level_rms = s.last_audio_level_rms;

        Ok(SessionStatusResponse {
            is_connected: true,
            device_name: dev_name,
            battery_pct,
            is_charging,
            fps,
            bitrate_kbps,
            latency_ms,
            connection_mode: conn_mode,
            audio_level_rms,
            has_audio,
        })
    } else {
        Ok(SessionStatusResponse {
            is_connected: false,
            device_name: "No Device Connected".to_string(),
            battery_pct: 0,
            is_charging: false,
            fps: 0.0,
            bitrate_kbps: 0,
            latency_ms: 0,
            connection_mode: "Disconnected".to_string(),
            audio_level_rms: 0.0,
            has_audio: false,
        })
    }
}

#[tauri::command]
async fn send_camera_control(
    server: tauri::State<'_, Arc<NetworkServer>>,
    action: String,
    params: serde_json::Value,
) -> Result<(), String> {
    let cmd = match action.as_str() {
        "set_zoom" => {
            let ratio = params.get("ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            linkcam_protocol::CameraControl::SetZoom { ratio }
        }
        "set_exposure" => {
            let ev = params.get("ev")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            linkcam_protocol::CameraControl::SetExposure { ev, auto: false }
        }
        "set_torch" => {
            let enabled = params.get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            linkcam_protocol::CameraControl::SetTorch { enabled }
        }
        "set_lens" => {
            let lens_str = params.get("lens")
                .and_then(|v| v.as_str())
                .unwrap_or("RearWide");
            let lens = match lens_str {
                "Front" => linkcam_protocol::CameraLens::Front,
                "RearUltraWide" => linkcam_protocol::CameraLens::RearUltraWide,
                "RearTelephoto" => linkcam_protocol::CameraLens::RearTelephoto,
                _ => linkcam_protocol::CameraLens::RearWide,
            };
            linkcam_protocol::CameraControl::SetCameraLens { lens }
        }
        "set_meeting_mode" => {
            let enabled = params.get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            linkcam_protocol::CameraControl::SetMeetingMode { enabled }
        }
        _ => return Err(format!("Unknown camera control action: {}", action)),
    };

    server.send_camera_control(cmd).await
        .map_err(|e| format!("Failed to send camera control: {}", e))
}

#[tauri::command]
async fn get_save_destination(storage: tauri::State<'_, Arc<CaptureStorage>>) -> Result<String, String> {
    Ok(storage.get_base_dir().to_string_lossy().to_string())
}

#[tauri::command]
async fn set_save_destination(storage: tauri::State<'_, Arc<CaptureStorage>>, path: String) -> Result<String, String> {
    storage.set_base_dir(&path)
}

#[tauri::command]
async fn get_adb_devices(server: tauri::State<'_, Arc<NetworkServer>>) -> Result<Vec<AdbDeviceInfo>, String> {
    let mut devices = AdbManager::list_devices()?;
    let mgr = server.get_pairing_manager();
    let pm = mgr.read().await;
    for dev in &mut devices {
        dev.is_trusted = pm.is_device_trusted(&dev.serial);
    }
    Ok(devices)
}

#[tauri::command]
async fn trust_adb_device(server: tauri::State<'_, Arc<NetworkServer>>, serial: String) -> Result<(), String> {
    let mgr = server.get_pairing_manager();
    let mut pm = mgr.write().await;
    pm.trust_adb_device(&serial);
    println!("[LinkCam] User authorized and trusted USB device: {}", serial);
    Ok(())
}

#[tauri::command]
async fn pair_adb_device(server: tauri::State<'_, Arc<NetworkServer>>, serial: String) -> Result<String, String> {
    let mgr = server.get_pairing_manager();
    let mut pm = mgr.write().await;

    if !pm.is_device_trusted(&serial) {
        return Err("ERR_USB_NOT_TRUSTED: This USB device must be explicitly trusted by user before pairing".to_string());
    }

    // Configure ADB reverse port forwarding strictly for TCP port 8443
    AdbManager::setup_reverse(Some(&serial), 8443)?;

    // Generate dynamic one-time USB pairing token and session ID bound to this serial
    let (token, session_id) = pm.generate_usb_token(&serial);
    let cert_fp = pm.cert_fingerprint().to_string();

    // Push token directly to Android phone via ADB Intent with TLS fingerprint
    AdbManager::push_pairing_token(&serial, &token, &session_id, &cert_fp, 8443)?;

    Ok(format!("Pairing token successfully pushed to USB device {}", serial))
}

#[tauri::command]
async fn manual_usb_pair(server: tauri::State<'_, Arc<NetworkServer>>) -> Result<String, String> {
    let mgr = server.get_pairing_manager();
    let mut pm = mgr.write().await;

    // Trust default connection
    pm.trust_adb_device("default");

    // Configure ADB reverse for tcp:8443
    AdbManager::setup_reverse(None, 8443)?;

    // Generate token
    let (token, session_id) = pm.generate_usb_token("default");
    let cert_fp = pm.cert_fingerprint().to_string();

    // Push token to default attached device
    AdbManager::push_pairing_token("", &token, &session_id, &cert_fp, 8443)?;

    Ok("Port 8443 reversed & USB pairing token sent to connected phone!".to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct LatestFrameMeta {
    pub has_frame: bool,
    pub width: u32,
    pub height: u32,
    pub timestamp_us: u64,
    pub sequence: u32,
    pub is_keyframe: bool,
    pub total_frames: u64,
}

#[tauri::command]
async fn get_latest_frame_info(sink: tauri::State<'_, Arc<VirtualCameraSink>>) -> Result<LatestFrameMeta, String> {
    let total = sink.total_frames();
    if let Some(f) = sink.get_latest_frame().await {
        Ok(LatestFrameMeta {
            has_frame: true,
            width: f.width,
            height: f.height,
            timestamp_us: f.timestamp_us,
            sequence: f.sequence,
            is_keyframe: f.is_keyframe,
            total_frames: total,
        })
    } else {
        Ok(LatestFrameMeta {
            has_frame: false,
            width: 1920,
            height: 1080,
            timestamp_us: 0,
            sequence: 0,
            is_keyframe: false,
            total_frames: total,
        })
    }
}

fn main() {
    let (capture_chunk_tx, mut capture_chunk_rx) = mpsc::channel(64);
    let (delivered_tx, mut delivered_rx) = mpsc::channel(16);

    let (server, mut video_frame_rx) = NetworkServer::new(
        8443,
        "Windows-PC".to_string(),
        capture_chunk_tx,
    );
    let server_arc = Arc::new(server);
    let storage_arc = Arc::new(CaptureStorage::new());

    let vcam_sink = Arc::new(VirtualCameraSink::new());
    vcam_sink.start();

    let srv_manage = server_arc.clone();
    let storage_manage = storage_arc.clone();
    let srv_setup = server_arc.clone();
    let sink_manage = vcam_sink.clone();
    let sink_setup = vcam_sink.clone();

    tauri::Builder::default()
        .manage(srv_manage)
        .manage(storage_manage)
        .manage(sink_manage)
        .invoke_handler(tauri::generate_handler![
            get_pairing_payload,
            rotate_credentials,
            get_connection_status,
            get_save_destination,
            set_save_destination,
            get_adb_devices,
            trust_adb_device,
            pair_adb_device,
            manual_usb_pair,
            get_latest_frame_info,
            send_camera_control
        ])
        .setup(move |_app| {
            println!("[LinkCam] Starting LinkCam Windows Desktop Companion Engine...");

            let vcam_device = Arc::new(RwLock::new(VirtualCameraDevice::new(sink_setup.clone())));

            // Register Windows Virtual Camera in DirectShow/MediaFoundation catalog
            {
                let vcam_clone = vcam_device.clone();
                tauri::async_runtime::spawn(async move {
                    let mut dev = vcam_clone.write().await;
                    if let Err(e) = dev.register() {
                        eprintln!("[LinkCam] Warning: Virtual camera registration: {:?}", e);
                    }
                });
            }

            let capture_receiver = Arc::new(CaptureReceiver::new(delivered_tx, storage_arc.clone()));

            // Spawn video frame consumer to virtual camera sink
            let sink_clone = sink_setup.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match video_frame_rx.recv().await {
                        Ok(packet) => {
                            sink_clone.push_packet(packet).await;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            eprintln!("[LinkCam VCam] Frame consumer lagged by {} frames; resuming", skipped);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            });

            // Spawn capture chunk consumer
            let receiver_clone = capture_receiver.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(packet) = capture_chunk_rx.recv().await {
                    let _ = receiver_clone.process_chunk(packet).await;
                }
            });

            // Spawn delivered capture notification logger
            tauri::async_runtime::spawn(async move {
                while let Some(event) = delivered_rx.recv().await {
                    println!(
                        "[LinkCam Notification] Captured '{}' ({} bytes) -> Saved to: {}",
                        event.file_name, event.file_size, event.file_path
                    );
                }
            });

            // Start network server in background
            let srv_clone = srv_setup.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = srv_clone.start().await {
                    eprintln!("[LinkCam] Network server error: {:?}", e);
                }
            });

            // Start UDP video datagram server on port 8444 (zero-HOL blocking)
            let udp_srv = Arc::new(UdpVideoServer::new(8444, srv_setup.get_video_frame_tx()));
            tauri::async_runtime::spawn(async move {
                if let Err(e) = udp_srv.start().await {
                    eprintln!("[LinkCam] UDP video server error: {:?}", e);
                }
            });

            println!("[LinkCam] Companion engine ready. QR Pairing & TCP Stream on 8443, UDP video stream on 8444.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LinkCam application");
}
