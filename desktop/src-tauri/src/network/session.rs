use linkcam_protocol::{CameraControl, ConnectionMode, TelemetryHeartbeat};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Disconnected,
    PairingPending,
    Connected,
    Streaming,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub client_name: String,
    pub model: String,
    pub os_version: String,
    pub supported_lenses: Vec<String>,
}

pub struct ClientSession {
    pub state: SessionState,
    pub device_info: Option<DeviceInfo>,
    pub connection_mode: ConnectionMode,
    pub last_telemetry: Option<TelemetryHeartbeat>,
    pub sequence_counter: Arc<AtomicU32>,
    pub control_tx: mpsc::Sender<CameraControl>,
    pub last_audio_level_rms: f32,
    pub total_audio_frames: u64,
}

impl ClientSession {
    pub fn new(control_tx: mpsc::Sender<CameraControl>) -> Self {
        Self {
            state: SessionState::Disconnected,
            device_info: None,
            connection_mode: ConnectionMode::WiFi,
            last_telemetry: None,
            sequence_counter: Arc::new(AtomicU32::new(1)),
            control_tx,
            last_audio_level_rms: 0.0,
            total_audio_frames: 0,
        }
    }

    pub fn next_sequence(&self) -> u32 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
    }

    pub fn update_telemetry(&mut self, telemetry: TelemetryHeartbeat) {
        self.connection_mode = telemetry.connection_mode;
        self.last_telemetry = Some(telemetry);
    }

    pub fn update_audio_level(&mut self, rms: f32) {
        self.last_audio_level_rms = rms;
        self.total_audio_frames = self.total_audio_frames.saturating_add(1);
    }

    pub async fn send_control(&self, cmd: CameraControl) -> Result<(), mpsc::error::SendError<CameraControl>> {
        self.control_tx.send(cmd).await
    }
}
