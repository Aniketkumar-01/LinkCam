use super::clipboard::WindowsClipboard;
use super::storage::CaptureStorage;
use linkcam_protocol::{InstantCaptureMeta, Packet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

const MAX_TRANSFER_CHUNKS: u32 = 10_000;
const MAX_TRANSFER_BYTES: u64 = 256 * 1024 * 1024; // 256 MB max capture
const TRANSFER_TTL_SECS: u64 = 300; // 5 minutes timeout

struct ActiveTransfer {
    meta: InstantCaptureMeta,
    chunks: Vec<Option<Vec<u8>>>,
    received_count: u32,
    created_at: std::time::Instant,
}

pub struct CaptureReceiver {
    storage: Arc<CaptureStorage>,
    transfers: Arc<RwLock<HashMap<String, ActiveTransfer>>>,
    delivered_tx: mpsc::Sender<DeliveredCaptureEvent>,
}

impl CaptureReceiver {
    pub fn new(delivered_tx: mpsc::Sender<DeliveredCaptureEvent>, storage: Arc<CaptureStorage>) -> Self {
        Self {
            storage,
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

        // Validate metadata bounds
        if meta.total_chunks == 0 || meta.total_chunks > MAX_TRANSFER_CHUNKS {
            return Err(format!("Invalid total_chunks count: {}", meta.total_chunks));
        }
        if meta.chunk_index >= meta.total_chunks {
            return Err(format!("Chunk index {} out of bounds ({})", meta.chunk_index, meta.total_chunks));
        }
        if meta.total_size > MAX_TRANSFER_BYTES {
            return Err(format!("Transfer total_size {} exceeds 256MB limit", meta.total_size));
        }

        let chunk_data = packet.payload[4 + meta_len..].to_vec();

        let mut transfers = self.transfers.write().await;
        // Prune stale transfers older than TTL
        transfers.retain(|_, t| t.created_at.elapsed().as_secs() < TRANSFER_TTL_SECS);

        if !transfers.contains_key(&meta.transfer_id) && transfers.len() >= 50 {
            return Err("Max concurrent active transfers reached".to_string());
        }

        let transfer = transfers.entry(meta.transfer_id.clone()).or_insert_with(|| {
            let total = meta.total_chunks as usize;
            ActiveTransfer {
                meta: meta.clone(),
                chunks: vec![None; total],
                received_count: 0,
                created_at: std::time::Instant::now(),
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

            // Verify cryptographic SHA-256 integrity before saving
            if !transfer.meta.sha256_checksum.is_empty() {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&complete_data);
                let computed_hash = format!("{:x}", hasher.finalize());

                if !computed_hash.eq_ignore_ascii_case(&transfer.meta.sha256_checksum) {
                    eprintln!(
                        "[LinkCam Security] Checksum mismatch for transfer {}: expected {}, computed {}",
                        transfer.meta.transfer_id, transfer.meta.sha256_checksum, computed_hash
                    );
                    return Err(format!(
                        "Integrity verification failed: checksum mismatch (expected {}, computed {})",
                        transfer.meta.sha256_checksum, computed_hash
                    ));
                }
            }

            // Save to disk using desktop-generated safe filename based on validated MIME type
            let saved_path = self.storage
                .save_capture(&transfer.meta.mime_type, &complete_data)
                .map_err(|e| format!("Failed to save file: {}", e))?;

            // Copy to Windows clipboard
            let _ = WindowsClipboard::copy_file_or_image(
                &saved_path,
                &transfer.meta.mime_type,
                &complete_data,
            );

            let saved_file_name = saved_path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("capture.bin")
                .to_string();

            let event = DeliveredCaptureEvent {
                transfer_id: transfer.meta.transfer_id.clone(),
                file_name: saved_file_name,
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
