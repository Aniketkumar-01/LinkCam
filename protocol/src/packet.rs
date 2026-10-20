use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor, Read, Write};
use thiserror::Error;

/// Protocol Magic Bytes: "LNKC" (LinkCam)
pub const MAGIC_BYTES: [u8; 4] = [b'L', b'N', b'K', b'C'];
pub const PROTOCOL_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 24;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid magic header: expected 'LNKC', found {0:?}")]
    InvalidMagic([u8; 4]),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    #[error("Unknown packet type: {0:#04x}")]
    UnknownPacketType(u8),

    #[error("Packet payload too large: {0} bytes (max {1})")]
    PayloadTooLarge(usize, usize),

    #[error("Packet incomplete: expected {expected} bytes, but only {available} available")]
    IncompletePacket { expected: usize, available: usize },

    #[error("Serialization / Deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Packet Type Identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    /// Keep-alive ping and round-trip latency probe
    KeepAlivePing = 0x00,
    /// Live video frame payload
    VideoFrame = 0x01,
    /// Optional synchronized audio packet
    AudioFrame = 0x02,
    /// Chunk of an instant photo or scanned PDF document
    InstantCaptureChunk = 0x03,
    /// Codec configuration and parameter sets (SPS/PPS/VPS)
    CodecConfig = 0x04,
    /// Instantaneous Decoder Refresh (IDR) keyframe request
    KeyframeRequest = 0x05,
    /// Graceful session termination
    Disconnect = 0x06,
    /// Selective retransmission request for missing capture chunk
    ChunkRetransmit = 0x07,
    /// Pairing and authentication handshake initiation
    PairingAuth = 0x10,
    /// Pairing authentication response / token verification
    PairingAuthResponse = 0x11,
    /// Mutual SAS / 6-digit PIN verification confirmation
    PairingVerify = 0x12,
    /// Remote camera control RPC from PC to Phone
    CameraControl = 0x20,
    /// Acknowledgment of camera control
    CameraControlAck = 0x21,
    /// Real-time stream telemetry (FPS, bitrate, battery, thermal)
    TelemetryHeartbeat = 0x30,
}

impl TryFrom<u8> for PacketType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(PacketType::KeepAlivePing),
            0x01 => Ok(PacketType::VideoFrame),
            0x02 => Ok(PacketType::AudioFrame),
            0x03 => Ok(PacketType::InstantCaptureChunk),
            0x04 => Ok(PacketType::CodecConfig),
            0x05 => Ok(PacketType::KeyframeRequest),
            0x06 => Ok(PacketType::Disconnect),
            0x07 => Ok(PacketType::ChunkRetransmit),
            0x10 => Ok(PacketType::PairingAuth),
            0x11 => Ok(PacketType::PairingAuthResponse),
            0x12 => Ok(PacketType::PairingVerify),
            0x20 => Ok(PacketType::CameraControl),
            0x21 => Ok(PacketType::CameraControlAck),
            0x30 => Ok(PacketType::TelemetryHeartbeat),
            other => Err(ProtocolError::UnknownPacketType(other)),
        }
    }
}

/// Binary Wire Packet Header (24 bytes)
///
/// Layout:
/// - [0..4]   Magic Bytes ("LNKC")
/// - [4]      Protocol Version (1)
/// - [5]      Packet Type (0x01..0x30)
/// - [6]      Flags (bit 0: Keyframe, bit 1: Compressed, bit 2: Encrypted)
/// - [7]      Reserved (0x00)
/// - [8..12]  Sequence Number (u32 big-endian)
/// - [12..20] Timestamp in ms or PTS in microseconds (u64 big-endian)
/// - [20..24] Payload Length in bytes (u32 big-endian)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub version: u8,
    pub packet_type: PacketType,
    pub flags: u8,
    pub sequence: u32,
    pub timestamp: u64,
    pub payload_length: u32,
}

impl PacketHeader {
    pub const FLAG_KEYFRAME: u8 = 0x01;
    pub const FLAG_CONFIG: u8 = 0x02;
    pub const FLAG_END_OF_STREAM: u8 = 0x04;
    pub const FLAG_ENCRYPTED: u8 = 0x08;

    pub fn new(packet_type: PacketType, sequence: u32, timestamp: u64, payload_length: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            packet_type,
            flags: 0,
            sequence,
            timestamp,
            payload_length,
        }
    }

    pub fn is_keyframe(&self) -> bool {
        (self.flags & Self::FLAG_KEYFRAME) != 0
    }

    pub fn set_keyframe(&mut self, is_keyframe: bool) {
        if is_keyframe {
            self.flags |= Self::FLAG_KEYFRAME;
        } else {
            self.flags &= !Self::FLAG_KEYFRAME;
        }
    }

    pub fn is_config(&self) -> bool {
        (self.flags & Self::FLAG_CONFIG) != 0
    }

    pub fn set_config(&mut self, is_config: bool) {
        if is_config {
            self.flags |= Self::FLAG_CONFIG;
        } else {
            self.flags &= !Self::FLAG_CONFIG;
        }
    }

    pub fn is_encrypted(&self) -> bool {
        (self.flags & Self::FLAG_ENCRYPTED) != 0
    }

    pub fn set_encrypted(&mut self, is_encrypted: bool) {
        if is_encrypted {
            self.flags |= Self::FLAG_ENCRYPTED;
        } else {
            self.flags &= !Self::FLAG_ENCRYPTED;
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&MAGIC_BYTES)?;
        writer.write_u8(self.version)?;
        writer.write_u8(self.packet_type as u8)?;
        writer.write_u8(self.flags)?;
        writer.write_u8(0)?; // reserved
        writer.write_u32::<BigEndian>(self.sequence)?;
        writer.write_u64::<BigEndian>(self.timestamp)?;
        writer.write_u32::<BigEndian>(self.payload_length)?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, ProtocolError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC_BYTES {
            return Err(ProtocolError::InvalidMagic(magic));
        }

        let version = reader.read_u8()?;
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let type_byte = reader.read_u8()?;
        let packet_type = PacketType::try_from(type_byte)?;
        let flags = reader.read_u8()?;
        let _reserved = reader.read_u8()?;
        let sequence = reader.read_u32::<BigEndian>()?;
        let timestamp = reader.read_u64::<BigEndian>()?;
        let payload_length = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            version,
            packet_type,
            flags,
            sequence,
            timestamp,
            payload_length,
        })
    }
}

/// Raw LinkCam Frame Packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Maximum allowed payload size (10 MB for high-res photo chunks / document scans)
    pub const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;
    /// Maximum allowed video frame size (2 MB for H.264 / HEVC NAL slices)
    pub const MAX_VIDEO_FRAME_SIZE: usize = 2 * 1024 * 1024;

    pub fn new(header: PacketHeader, payload: Vec<u8>) -> Self {
        let mut p = Self { header, payload };
        p.header.payload_length = p.payload.len() as u32;
        p
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        self.header.write_to(&mut buffer)?;
        buffer.write_all(&self.payload)?;
        Ok(buffer)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ProtocolError::IncompletePacket {
                expected: HEADER_SIZE,
                available: bytes.len(),
            });
        }

        let mut cursor = Cursor::new(bytes);
        let header = PacketHeader::read_from(&mut cursor)?;

        let expected_total = HEADER_SIZE + header.payload_length as usize;
        if bytes.len() < expected_total {
            return Err(ProtocolError::IncompletePacket {
                expected: expected_total,
                available: bytes.len(),
            });
        }

        if header.payload_length as usize > Self::MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(
                header.payload_length as usize,
                Self::MAX_PAYLOAD_SIZE,
            ));
        }

        let payload = bytes[HEADER_SIZE..expected_total].to_vec();
        Ok(Self { header, payload })
    }
}

// ---------------------------------------------------------------------------
// Typed Control & Telemetry Payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    Mjpeg,
    Vp8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraLens {
    RearWide,
    RearUltraWide,
    RearTelephoto,
    Front,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
    Photo,
    DocumentScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionMode {
    Usb,
    WiFi,
    DirectHotspot,
}

/// Remote Camera Control RPC Message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum CameraControl {
    SetResolution { width: u32, height: u32, fps: u32 },
    SetCameraLens { lens: CameraLens },
    SetTorch { enabled: bool },
    SetZoom { ratio: f32 },
    SetExposure { ev: f32, auto: bool },
    SetMeetingMode { enabled: bool }, // Dim/black screen on phone
    TriggerInstantCapture { mode: CaptureMode },
}

/// Telemetry Heartbeat sent from Phone to PC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryHeartbeat {
    pub fps: f32,
    pub bitrate_kbps: u32,
    pub latency_ms: u32,
    pub battery_pct: u8,
    pub is_charging: bool,
    pub thermal_state: String,
    pub connection_mode: ConnectionMode,
}

/// Instant Capture Chunk Metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstantCaptureMeta {
    pub transfer_id: String,
    pub file_name: String,
    pub mime_type: String, // "image/jpeg" or "application/pdf"
    pub total_size: u64,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub sha256_checksum: String,
}

/// Codec Configuration Payload (PacketType::CodecConfig, 0x04)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodecConfigPayload {
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vps: Option<Vec<u8>>,
}

/// Instantaneous Decoder Refresh (IDR) Keyframe Request (PacketType::KeyframeRequest, 0x05)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeRequestPayload {
    pub stream_id: u32,
    pub reason: String,
}

/// Graceful Session Disconnect Payload (PacketType::Disconnect, 0x06)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisconnectPayload {
    pub reason_code: u16,
    pub message: String,
}

/// Selective Chunk Retransmission Request (PacketType::ChunkRetransmit, 0x07)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRetransmitPayload {
    pub transfer_id: String,
    pub chunk_index: u32,
}

/// Mutual SAS / PIN Verification Payload (PacketType::PairingVerify, 0x12)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingVerifyPayload {
    pub sas_code: String,
    pub auth_token: String,
    pub is_trusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization_roundtrip() {
        let mut header = PacketHeader::new(PacketType::VideoFrame, 1024, 1698765432000, 4096);
        header.set_keyframe(true);

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();
        assert_eq!(buffer.len(), HEADER_SIZE);

        let mut cursor = Cursor::new(&buffer);
        let parsed = PacketHeader::read_from(&mut cursor).unwrap();

        assert_eq!(header, parsed);
        assert!(parsed.is_keyframe());
    }

    #[test]
    fn test_packet_full_roundtrip() {
        let payload = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc];
        let header = PacketHeader::new(PacketType::CameraControl, 42, 500, payload.len() as u32);
        let packet = Packet::new(header, payload.clone());

        let bytes = packet.to_bytes().unwrap();
        let decoded = Packet::from_bytes(&bytes).unwrap();

        assert_eq!(packet, decoded);
        assert_eq!(decoded.payload, payload);
        assert_eq!(decoded.header.packet_type, PacketType::CameraControl);
    }

    #[test]
    fn test_invalid_magic_error() {
        let bad_bytes = vec![b'X', b'Y', b'Z', b'!', 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let res = Packet::from_bytes(&bad_bytes);
        assert!(matches!(res, Err(ProtocolError::InvalidMagic(_))));
    }

    #[test]
    fn test_control_rpc_json_roundtrip() {
        let cmd = CameraControl::SetCameraLens {
            lens: CameraLens::RearUltraWide,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: CameraControl = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_telemetry_heartbeat_roundtrip() {
        let hb = TelemetryHeartbeat {
            fps: 59.94,
            bitrate_kbps: 4850,
            latency_ms: 38,
            battery_pct: 88,
            is_charging: true,
            thermal_state: "Normal".to_string(),
            connection_mode: ConnectionMode::DirectHotspot,
        };
        let json = serde_json::to_string(&hb).unwrap();
        let deserialized: TelemetryHeartbeat = serde_json::from_str(&json).unwrap();
        assert_eq!(hb, deserialized);
    }

    #[test]
    fn test_phase1_new_packet_types_roundtrip() {
        let types = vec![
            PacketType::KeepAlivePing,
            PacketType::CodecConfig,
            PacketType::KeyframeRequest,
            PacketType::Disconnect,
            PacketType::ChunkRetransmit,
            PacketType::PairingVerify,
        ];

        for pt in types {
            let byte = pt as u8;
            let decoded = PacketType::try_from(byte).unwrap();
            assert_eq!(pt, decoded);
        }
    }

    #[test]
    fn test_codec_config_payload_roundtrip() {
        let config = CodecConfigPayload {
            codec: VideoCodec::H264,
            width: 1920,
            height: 1080,
            fps: 60,
            sps: vec![0x67, 0x42, 0x00, 0x28],
            pps: vec![0x68, 0xce, 0x38, 0x80],
            vps: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: CodecConfigPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(config, decoded);
    }

    #[test]
    fn test_pairing_verify_payload_roundtrip() {
        let verify = PairingVerifyPayload {
            sas_code: "582 391".to_string(),
            auth_token: "tok_secure_test_123".to_string(),
            is_trusted: true,
        };
        let json = serde_json::to_string(&verify).unwrap();
        let decoded: PairingVerifyPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(verify, decoded);
    }
}
