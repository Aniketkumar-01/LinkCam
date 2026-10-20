#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod network;
mod vcam;

use capture::CaptureReceiver;
use network::{NetworkServer, PairingPayload, UdpVideoServer};
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

        Ok(SessionStatusResponse {
            is_connected: true,
            device_name: dev_name,
            battery_pct,
            is_charging,
            fps,
            bitrate_kbps,
            latency_ms,
            connection_mode: conn_mode,
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

    let srv_manage = server_arc.clone();
    let srv_setup = server_arc.clone();

    tauri::Builder::default()
        .manage(srv_manage)
        .invoke_handler(tauri::generate_handler![
            get_pairing_payload,
            rotate_credentials,
            get_connection_status
        ])
        .setup(move |_app| {
            println!("[LinkCam] Starting LinkCam Windows Desktop Companion Engine...");

            let vcam_sink = Arc::new(VirtualCameraSink::new());
            let vcam_device = Arc::new(RwLock::new(VirtualCameraDevice::new(vcam_sink.clone())));

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

            let capture_receiver = Arc::new(CaptureReceiver::new(delivered_tx));

            // Spawn video frame consumer to virtual camera sink
            let sink_clone = vcam_sink.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(packet) = video_frame_rx.recv().await {
                    sink_clone.push_packet(packet).await;
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

            println!("[LinkCam] Companion engine ready. QR Pairing on 8443, UDP video stream on 8444.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LinkCam application");
}
