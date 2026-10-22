use super::pairing::PairingManager;
use super::session::{ClientSession, SessionState};
use futures_util::{SinkExt, StreamExt};
use linkcam_protocol::{CameraControl, Packet, PacketHeader, PacketType, TelemetryHeartbeat, HEADER_SIZE};
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub struct NetworkServer {
    port: u16,
    pairing_mgr: Arc<RwLock<PairingManager>>,
    session: Arc<RwLock<Option<ClientSession>>>,
    video_frame_tx: broadcast::Sender<Packet>,
    capture_chunk_tx: mpsc::Sender<Packet>,
    control_tx: mpsc::Sender<CameraControl>,
    control_rx: Arc<Mutex<mpsc::Receiver<CameraControl>>>,
}

impl NetworkServer {
    pub fn new(
        port: u16,
        device_name: String,
        capture_chunk_tx: mpsc::Sender<Packet>,
    ) -> (Self, broadcast::Receiver<Packet>) {
        let (video_frame_tx, video_frame_rx) = broadcast::channel(64);
        let (control_tx, control_rx) = mpsc::channel(32);

        let server = Self {
            port,
            pairing_mgr: Arc::new(RwLock::new(PairingManager::new(port, device_name))),
            session: Arc::new(RwLock::new(None)),
            video_frame_tx,
            capture_chunk_tx,
            control_tx,
            control_rx: Arc::new(Mutex::new(control_rx)),
        };

        (server, video_frame_rx)
    }

    pub fn get_pairing_manager(&self) -> Arc<RwLock<PairingManager>> {
        self.pairing_mgr.clone()
    }

    pub fn get_session(&self) -> Arc<RwLock<Option<ClientSession>>> {
        self.session.clone()
    }

    pub fn get_video_frame_tx(&self) -> broadcast::Sender<Packet> {
        self.video_frame_tx.clone()
    }

    pub async fn send_camera_control(&self, cmd: CameraControl) -> Result<(), mpsc::error::SendError<CameraControl>> {
        self.control_tx.send(cmd).await
    }

    /// Start listening on 0.0.0.0:[port] for both Wi-Fi, Hotspot, and USB connections
    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await?;

        println!("[LinkCam] Network server listening on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            println!("[LinkCam] Incoming connection from {}", peer_addr);

            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream, peer_addr).await {
                    eprintln!("[LinkCam] Connection error with {}: {:?}", peer_addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Enforce private subnet only (LAN, Hotspot, Loopback)
        let is_private = match peer_addr.ip() {
            std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
            std::net::IpAddr::V6(v6) => v6.is_loopback(),
        };

        if !is_private {
            eprintln!("[LinkCam Security] Dropped connection from non-private IP: {}", peer_addr);
            return Ok(());
        }

        // Peek first 4 bytes to check if client is connecting via raw TCP ("LNKC") or WebSocket ("GET ")
        let mut peek_buf = [0u8; 4];
        stream.peek(&mut peek_buf).await?;

        if &peek_buf == b"GET " {
            // WebSocket upgrade
            let ws_stream = accept_async(stream).await?;
            self.handle_websocket(ws_stream, peer_addr).await?;
        } else {
            // Raw binary TCP stream (lowest latency)
            self.handle_raw_tcp(stream, peer_addr).await?;
        }

        Ok(())
    }

    async fn handle_raw_tcp(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (mut reader, mut writer) = stream.into_split();
        let mut header_buf = [0u8; HEADER_SIZE];

        // Read handshake packet
        reader.read_exact(&mut header_buf).await?;
        let header = PacketHeader::read_from(&mut Cursor::new(&header_buf))?;

        if header.packet_type != PacketType::PairingAuth {
            eprintln!("[LinkCam] Expected PairingAuth, received {:?}", header.packet_type);
            return Ok(());
        }

        if header.payload_length as usize > Packet::MAX_PAYLOAD_SIZE {
            eprintln!("[LinkCam] PairingAuth payload exceeds MAX_PAYLOAD_SIZE: {}", header.payload_length);
            return Ok(());
        }

        let mut payload = vec![0u8; header.payload_length as usize];
        reader.read_exact(&mut payload).await?;

        // Authenticate token with IP rate-limiting and expiration
        let auth_str = String::from_utf8_lossy(&payload);
        let parts: Vec<&str> = auth_str.split(':').collect();
        if parts.len() < 2 {
            return Ok(());
        }

        let auth_res = {
            let mut pairing = self.pairing_mgr.write().await;
            pairing.validate_token(peer_addr.ip(), parts[0], parts[1])
        };

        match auth_res {
            Ok(true) => {}
            Err(err_msg) => {
                eprintln!("[LinkCam] Authentication failed for {}: {}", peer_addr, err_msg);
                let resp_header = PacketHeader::new(PacketType::PairingAuthResponse, 1, 0, err_msg.len() as u32);
                let mut resp_buf = Vec::new();
                resp_header.write_to(&mut resp_buf)?;
                resp_buf.extend_from_slice(err_msg.as_bytes());
                let _ = writer.write_all(&resp_buf).await;
                return Ok(());
            }
            _ => return Ok(()),
        }

        // Send PairingAuthResponse success
        let resp_header = PacketHeader::new(PacketType::PairingAuthResponse, 1, 0, 2);
        let mut resp_buf = Vec::new();
        resp_header.write_to(&mut resp_buf)?;
        resp_buf.extend_from_slice(b"OK");
        writer.write_all(&resp_buf).await?;

        // Derive 32-byte transport session key from mutual pairing credentials
        let session_key = crate::network::crypto::derive_session_key(parts[0], parts[1]);

        // Outbound packet transmission queue & task
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<Vec<u8>>(32);
        let write_task = tokio::spawn(async move {
            while let Some(bytes) = outbound_rx.recv().await {
                if writer.write_all(&bytes).await.is_err() {
                    break;
                }
            }
        });

        // Forward remote camera control RPCs from desktop IPC to phone over TCP
        let control_rx_arc = self.control_rx.clone();
        let outbound_tx_ctrl = outbound_tx.clone();
        let control_task = tokio::spawn(async move {
            let mut rx = control_rx_arc.lock().await;
            let mut seq = 100u32;
            while let Some(cmd) = rx.recv().await {
                if let Ok(json_bytes) = serde_json::to_vec(&cmd) {
                    seq = seq.wrapping_add(1);
                    let hdr = PacketHeader::new(PacketType::CameraControl, seq, 0, json_bytes.len() as u32);
                    let mut pkt_buf = Vec::with_capacity(HEADER_SIZE + json_bytes.len());
                    if hdr.write_to(&mut pkt_buf).is_ok() {
                        pkt_buf.extend_from_slice(&json_bytes);
                        if outbound_tx_ctrl.send(pkt_buf).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Update session
        {
            let mut session_guard = self.session.write().await;
            let mut session = ClientSession::new(self.control_tx.clone());
            session.set_state(SessionState::Streaming);
            *session_guard = Some(session);
        }

        // Main frame receive loop
        loop {
            if let Err(_) = reader.read_exact(&mut header_buf).await {
                break;
            }

            let hdr = match PacketHeader::read_from(&mut Cursor::new(&header_buf)) {
                Ok(h) => h,
                Err(_) => break,
            };

            if hdr.payload_length as usize > Packet::MAX_PAYLOAD_SIZE {
                eprintln!("[LinkCam] Frame payload exceeds MAX_PAYLOAD_SIZE: {}", hdr.payload_length);
                break;
            }

            let mut body = vec![0u8; hdr.payload_length as usize];
            if let Err(_) = reader.read_exact(&mut body).await {
                break;
            }

            // Decrypt encrypted packets in-place using verified session key
            if (hdr.flags & PacketHeader::FLAG_ENCRYPTED) != 0 {
                let nonce = crate::network::crypto::construct_nonce(hdr.sequence, hdr.timestamp);
                crate::network::crypto::chacha20_crypt(&mut body, &session_key, &nonce, 1);
            }

            let packet = Packet::new(hdr.clone(), body);

            match hdr.packet_type {
                PacketType::KeepAlivePing => {
                    let pong_header = PacketHeader::new(PacketType::KeepAlivePing, hdr.sequence, hdr.timestamp, 0);
                    let mut pong_buf = Vec::new();
                    if pong_header.write_to(&mut pong_buf).is_ok() {
                        let _ = outbound_tx.send(pong_buf).await;
                    }
                }
                PacketType::Disconnect => {
                    eprintln!("[LinkCam] Graceful disconnect received from {}", peer_addr);
                    break;
                }
                PacketType::VideoFrame => {
                    let _ = self.video_frame_tx.send(packet);
                }
                PacketType::AudioFrame => {
                    // Calculate RMS amplitude from 48 kHz 16-bit PCM stereo samples
                    let mut sum_sq = 0.0f32;
                    let mut count = 0usize;
                    for chunk in packet.payload.chunks_exact(2) {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
                        sum_sq += sample * sample;
                        count += 1;
                    }
                    let rms = if count > 0 { (sum_sq / count as f32).sqrt().min(1.0) } else { 0.0 };
                    let mut sess = self.session.write().await;
                    if let Some(s) = sess.as_mut() {
                        s.update_audio_level(rms);
                    }
                }
                PacketType::InstantCaptureChunk => {
                    let _ = self.capture_chunk_tx.send(packet).await;
                }
                PacketType::TelemetryHeartbeat => {
                    if let Ok(hb) = serde_json::from_slice::<TelemetryHeartbeat>(&packet.payload) {
                        let mut sess = self.session.write().await;
                        if let Some(s) = sess.as_mut() {
                            s.update_telemetry(hb);
                        }
                    }
                }
                _ => {}
            }
        }

        // Abort background sender tasks upon disconnect
        write_task.abort();
        control_task.abort();

        // Cleanup session on disconnect
        {
            let mut session_guard = self.session.write().await;
            *session_guard = None;
        }

        Ok(())
    }

    async fn handle_websocket(
        &self,
        ws: tokio_tungstenite::WebSocketStream<TcpStream>,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Security Hardening: WebSocket endpoints are strictly reserved for local frontend UI preview.
        // Remote streaming from phones must use authenticated raw TCP protocol with PairingAuth tokens.
        if !peer_addr.ip().is_loopback() {
            eprintln!("[LinkCam Security] Dropped unauthorized non-loopback WebSocket attempt from {}", peer_addr);
            return Ok(());
        }

        let (mut ws_sender, mut ws_receiver) = ws.split();

        let mut video_rx = self.video_frame_tx.subscribe();
        let send_task = tokio::spawn(async move {
            loop {
                match video_rx.recv().await {
                    Ok(packet) => {
                        if let Ok(bytes) = packet.to_bytes() {
                            if ws_sender.send(Message::Binary(bytes)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("[LinkCam WebSocket] Loopback video stream lagged by {} frames; resuming", skipped);
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        while let Some(msg_res) = ws_receiver.next().await {
            if let Ok(msg) = msg_res {
                if msg.is_close() {
                    break;
                }
            } else {
                break;
            }
        }
        send_task.abort();
        Ok(())
    }
}
