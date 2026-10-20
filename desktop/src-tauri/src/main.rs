#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod network;
mod vcam;

use capture::CaptureReceiver;
use network::{NetworkServer, UdpVideoServer};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use vcam::{VirtualCameraDevice, VirtualCameraSink};

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
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
            let srv_clone = server_arc.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = srv_clone.start().await {
                    eprintln!("[LinkCam] Network server error: {:?}", e);
                }
            });

            // Start UDP video datagram server on port 8444 (zero-HOL blocking)
            let udp_srv = Arc::new(UdpVideoServer::new(8444, server_arc.get_video_frame_tx()));
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
