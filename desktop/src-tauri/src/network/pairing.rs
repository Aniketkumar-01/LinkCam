use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingPayload {
    pub session_id: String,
    pub auth_token: String,
    pub port: u16,
    pub ips: Vec<String>,
    pub device_name: String,
    pub protocol_version: u8,
}

#[derive(Debug, Clone)]
pub struct PairingManager {
    current_token: String,
    session_id: String,
    port: u16,
    device_name: String,
}

impl PairingManager {
    pub fn new(port: u16, device_name: String) -> Self {
        let mut mgr = Self {
            current_token: String::new(),
            session_id: String::new(),
            port,
            device_name,
        };
        mgr.rotate_credentials();
        mgr
    }

    /// Rotate session credentials for a fresh pairing session
    pub fn rotate_credentials(&mut self) {
        self.session_id = uuid::Uuid::new_v4().to_string();
        self.current_token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
    }

    /// Validate an incoming client handshake token
    pub fn validate_token(&self, token: &str, session: &str) -> bool {
        !self.current_token.is_empty()
            && self.current_token == token
            && self.session_id == session
    }

    /// Retrieve all usable local IP addresses for Wi-Fi and direct phone hotspot
    pub fn get_available_ips(&self) -> Vec<String> {
        let mut ips = Vec::new();

        // Check if there is an active default hotspot gateway (typically 192.168.43.x or 192.168.137.x)
        if let Ok(ip) = local_ip_address::local_ip() {
            ips.push(ip.to_string());
        }

        // Always provide localhost fallback
        if !ips.contains(&"127.0.0.1".to_string()) {
            ips.push("127.0.0.1".to_string());
        }

        ips
    }

    /// Generate the pairing payload embedded into the desktop QR code
    pub fn generate_pairing_payload(&self) -> PairingPayload {
        PairingPayload {
            session_id: self.session_id.clone(),
            auth_token: self.current_token.clone(),
            port: self.port,
            ips: self.get_available_ips(),
            device_name: self.device_name.clone(),
            protocol_version: linkcam_protocol::PROTOCOL_VERSION,
        }
    }

    /// Render pairing payload as JSON string for QR code generation
    pub fn generate_qr_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.generate_pairing_payload())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_rotation_and_validation() {
        let mut mgr = PairingManager::new(8443, "Windows-PC".to_string());
        let payload = mgr.generate_pairing_payload();

        assert_eq!(payload.port, 8443);
        assert!(!payload.auth_token.is_empty());
        assert!(!payload.session_id.is_empty());

        assert!(mgr.validate_token(&payload.auth_token, &payload.session_id));
        assert!(!mgr.validate_token("wrong-token", &payload.session_id));
        assert!(!mgr.validate_token(&payload.auth_token, "wrong-session"));

        let old_token = payload.auth_token;
        mgr.rotate_credentials();
        assert!(!mgr.validate_token(&old_token, &payload.session_id));
    }
}
