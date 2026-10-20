use linkcam_protocol::{Packet, PacketType, HEADER_SIZE};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;

/// High-throughput UDP video frame receiver for ultra-low latency Wi-Fi streaming
pub struct UdpVideoServer {
    port: u16,
    video_tx: broadcast::Sender<Packet>,
}

impl UdpVideoServer {
    pub fn new(port: u16, video_tx: broadcast::Sender<Packet>) -> Self {
        Self { port, video_tx }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("[LinkCam UDP] Low-latency video datagram receiver listening on {}", bind_addr);

        let mut buf = vec![0u8; 65536]; // Max UDP packet buffer

        loop {
            let (len, peer_addr) = match socket.recv_from(&mut buf).await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("[LinkCam UDP] Receive error: {:?}", e);
                    continue;
                }
            };

            // Enforce private subnet only
            let is_private = match peer_addr.ip() {
                std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
                std::net::IpAddr::V6(v6) => v6.is_loopback(),
            };

            if !is_private {
                continue;
            }

            if len < HEADER_SIZE {
                continue;
            }

            if let Ok(packet) = Packet::from_bytes(&buf[..len]) {
                if packet.header.packet_type == PacketType::VideoFrame {
                    let _ = self.video_tx.send(packet);
                }
            }
        }
    }
}
