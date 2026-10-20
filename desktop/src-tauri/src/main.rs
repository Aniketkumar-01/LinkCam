#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod network;
mod vcam;

use capture::{CaptureReceiver, DeliveredCaptureEvent};
use linkcam_protocol::{CameraControl, CameraLens, CaptureMode, ConnectionMode, TelemetryHeartbeat};
use network::{ClientSession, NetworkServer, PairingManager, PairingPayload, SessionState};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use vcam::{VirtualCameraDevice, VirtualCameraSink};

pub struct AppState {
    pub server: Arc<NetworkServer>,
    pub vcam: Arc<RwLock<VirtualCameraDevice>>,
    pub capture_receiver: Arc<CaptureReceiver>,
    pub latest_telemetry: Arc<RwLock<Option<TelemetryHeartbeat>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[LinkCam] Starting LinkCam Windows Desktop Companion Engine...");

    let (capture_chunk_tx, mut capture_chunk_rx) = mpsc::channel(64);
    let (delivered_tx, mut delivered_rx) = mpsc::channel(16);

    let (server, mut video_frame_rx) = NetworkServer::new(
        8443,
        "Windows-PC".to_string(),
        capture_chunk_tx,
    );
    let server_arc = Arc::new(server);

    let vcam_sink = Arc::new(VirtualCameraSink::new());
    let vcam_device = Arc::new(RwLock::new(VirtualCameraDevice::new(vcam_sink.clone())));

    // Register Windows Virtual Camera
    {
        let mut dev = vcam_device.write().await;
        if let Err(e) = dev.register() {
            eprintln!("[LinkCam] Warning: Virtual camera registration failed: {:?}", e);
        }
    }

    let capture_receiver = Arc::new(CaptureReceiver::new(delivered_tx));

    // Spawn video frame consumer to virtual camera sink
    let sink_clone = vcam_sink.clone();
    tokio::spawn(async move {
        while let Ok(packet) = video_frame_rx.recv().await {
            sink_clone.push_packet(packet).await;
        }
    });

    // Spawn capture chunk consumer
    let receiver_clone = capture_receiver.clone();
    tokio::spawn(async move {
        while let Some(packet) = capture_chunk_rx.recv().await {
            let _ = receiver_clone.process_chunk(packet).await;
        }
    });

    // Spawn delivered capture notification logger
    tokio::spawn(async move {
        while let Some(event) = delivered_rx.recv().await {
            println!(
                "[LinkCam Notification] Captured '{}' ({} bytes) -> Saved to: {}",
                event.file_name, event.file_size, event.file_path
            );
        }
    });

    // Start network server in background
    let srv_clone = server_arc.clone();
    tokio::spawn(async move {
        if let Err(e) = srv_clone.start().await {
            eprintln!("[LinkCam] Network server error: {:?}", e);
        }
    });

    println!("[LinkCam] Companion engine ready. QR Pairing available on port 8443.");

    // Launch standalone chromeless desktop UI window
    #[cfg(target_os = "windows")]
    {
        tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            let app_url = "http://127.0.0.1:8443";
            let appdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
            let profile_dir = format!("{}\\LinkCam\\ui_profile", appdata);

            let _ = std::process::Command::new("cmd")
                .args(&[
                    "/c",
                    "start",
                    "msedge.exe",
                    &format!("--app={}", app_url),
                    &format!("--user-data-dir={}", profile_dir),
                    "--window-size=1180,780",
                ])
                .spawn();
        });
    }

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("[LinkCam] Shutting down...");

    let mut dev = vcam_device.write().await;
    let _ = dev.unregister();

    Ok(())
}
