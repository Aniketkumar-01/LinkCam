use super::clipboard::WindowsClipboard;
use super::storage::CaptureStorage;
use linkcam_protocol::{InstantCaptureMeta, Packet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveredCaptureEvent {
    pub transfer_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_path: String,
    pub file_size: u64,
    pub preview_base64: Option<String>,
}

struct ActiveTransfer {
    meta: InstantCaptureMeta,
    chunks: Vec<Option<Vec<u8>>>,
    received_count: u32,
}

pub struct CaptureReceiver {
    storage: Arc<CaptureStorage>,
    transfers: Arc<RwLock<HashMap<String, ActiveTransfer>>>,
    delivered_tx: mpsc::Sender<DeliveredCaptureEvent>,
}

impl CaptureReceiver {
    pub fn new(delivered_tx: mpsc::Sender<DeliveredCaptureEvent>) -> Self {
        Self {
            storage: Arc::new(CaptureStorage::new()),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            delivered_tx,
        }
    }

    pub async fn process_chunk(&self, packet: Packet) -> Result<Option<DeliveredCaptureEvent>, String> {
        // First 4 bytes of InstantCaptureChunk payload is the length of JSON metadata header
        if packet.payload.len() < 4 {
            return Err("Chunk payload too small".to_string());
        }

        let meta_len = u32::from_be_bytes(packet.payload[0..4].try_into().unwrap()) as usize;
        if packet.payload.len() < 4 + meta_len {
            return Err("Corrupted chunk header".to_string());
        }

        let meta_json = &packet.payload[4..4 + meta_len];
        let meta: InstantCaptureMeta = serde_json::from_slice(meta_json)
            .map_err(|e| format!("Failed to parse capture meta: {}", e))?;

        let chunk_data = packet.payload[4 + meta_len..].to_vec();

        let mut transfers = self.transfers.write().await;
        let transfer = transfers.entry(meta.transfer_id.clone()).or_insert_with(|| {
            let total = meta.total_chunks as usize;
            ActiveTransfer {
                meta: meta.clone(),
                chunks: vec![None; total],
                received_count: 0,
            }
        });

        let idx = meta.chunk_index as usize;
        if idx < transfer.chunks.len() && transfer.chunks[idx].is_none() {
            transfer.chunks[idx] = Some(chunk_data);
            transfer.received_count += 1;
        }

        // Check if all chunks received
        if transfer.received_count == transfer.meta.total_chunks {
            // Reassemble complete file
            let mut complete_data = Vec::with_capacity(transfer.meta.total_size as usize);
            for chunk_opt in &transfer.chunks {
                if let Some(chunk) = chunk_opt {
                    complete_data.extend_from_slice(chunk);
                } else {
                    return Err("Missing chunk during reassembly".to_string());
                }
            }

            // Save to disk
            let saved_path = self.storage
                .save_capture(&transfer.meta.file_name, &complete_data)
                .map_err(|e| format!("Failed to save file: {}", e))?;

            // Copy to Windows clipboard
            let _ = WindowsClipboard::copy_file_or_image(
                &saved_path,
                &transfer.meta.mime_type,
                &complete_data,
            );

            let event = DeliveredCaptureEvent {
                transfer_id: transfer.meta.transfer_id.clone(),
                file_name: transfer.meta.file_name.clone(),
                mime_type: transfer.meta.mime_type.clone(),
                file_path: saved_path.to_string_lossy().to_string(),
                file_size: complete_data.len() as u64,
                preview_base64: None,
            };

            let _ = self.delivered_tx.send(event.clone()).await;

            // Remove finished transfer
            let id = transfer.meta.transfer_id.clone();
            transfers.remove(&id);

            return Ok(Some(event));
        }

        Ok(None)
    }
}
