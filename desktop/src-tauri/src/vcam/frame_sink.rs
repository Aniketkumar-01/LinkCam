use linkcam_protocol::Packet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct VideoFrameInfo {
    pub width: u32,
    pub height: u32,
    pub timestamp_us: u64,
    pub sequence: u32,
    pub is_keyframe: bool,
    pub data: Vec<u8>,
}

/// Windows Virtual Camera Video Frame Sink
pub struct VirtualCameraSink {
    is_active: AtomicBool,
    frames_processed: AtomicU64,
    latest_frame: Arc<RwLock<Option<VideoFrameInfo>>>,
}

impl VirtualCameraSink {
    pub fn new() -> Self {
        Self {
            is_active: AtomicBool::new(false),
            frames_processed: AtomicU64::new(0),
            latest_frame: Arc::new(RwLock::new(None)),
        }
    }

    pub fn start(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub async fn push_packet(&self, packet: Packet) {
        if !self.is_active() {
            return;
        }

        let frame_info = VideoFrameInfo {
            width: 1920, // default or parsed from packet header
            height: 1080,
            timestamp_us: packet.header.timestamp,
            sequence: packet.header.sequence,
            is_keyframe: packet.header.is_keyframe(),
            data: packet.payload,
        };

        {
            let mut latest = self.latest_frame.write().await;
            *latest = Some(frame_info);
        }

        self.frames_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn get_latest_frame(&self) -> Option<VideoFrameInfo> {
        let latest = self.latest_frame.read().await;
        latest.clone()
    }

    pub fn total_frames(&self) -> u64 {
        self.frames_processed.load(Ordering::Relaxed)
    }
}
