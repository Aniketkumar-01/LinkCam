use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingPayload {
    pub session_id: String,
    pub auth_token: String,
    pub verification_pin: String,
    pub port: u16,
    pub ips: Vec<String>,
    pub device_name: String,
    pub protocol_version: u8,
    pub expires_in_secs: u32,
    pub cert_fingerprint: String,
}

#[derive(Debug, Clone)]
struct IpAttemptRecord {
    failed_attempts: u32,
    first_attempt: Instant,
    blocked_until: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct PairingManager {
    current_token: String,
    session_id: String,
    verification_pin: String,
    created_at: Instant,
    token_lifetime: Duration,
    port: u16,
    device_name: String,
    cert_fingerprint: String,
    ip_records: HashMap<IpAddr, IpAttemptRecord>,
}

impl PairingManager {
    pub fn new(port: u16, device_name: String) -> Self {
        let mut mgr = Self {
            current_token: String::new(),
            session_id: String::new(),
            verification_pin: String::new(),
            created_at: Instant::now(),
            token_lifetime: Duration::from_secs(120), // 120-second token lifetime
            port,
            device_name,
            cert_fingerprint: "SHA256:4a:3b:2c:1d:linkcam:v1.1".to_string(),
            ip_records: HashMap::new(),
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

        // Generate 6-digit mutual SAS verification PIN (e.g. "582 391")
        let pin_val: u32 = rand::thread_rng().gen_range(100_000..=999_999);
        let pin_str = pin_val.to_string();
        self.verification_pin = format!("{} {}", &pin_str[..3], &pin_str[3..]);
        self.created_at = Instant::now();
    }

    /// Check if current pairing token has expired
    pub fn is_token_expired(&self) -> bool {
        self.created_at.elapsed() > self.token_lifetime
    }

    /// Check if an IP address is currently blocked due to excessive failed attempts
    pub fn is_ip_blocked(&mut self, ip: &IpAddr) -> bool {
        if let Some(record) = self.ip_records.get_mut(ip) {
            if let Some(blocked_until) = record.blocked_until {
                if Instant::now() < blocked_until {
                    return true;
                } else {
                    // Block expired, reset record
                    record.blocked_until = None;
                    record.failed_attempts = 0;
                }
            }
        }
        false
    }

    /// Record pairing attempt outcome for rate limiting
    pub fn record_attempt(&mut self, ip: IpAddr, success: bool) {
        let now = Instant::now();
        let record = self.ip_records.entry(ip).or_insert_with(|| IpAttemptRecord {
            failed_attempts: 0,
            first_attempt: now,
            blocked_until: None,
        });

        if success {
            record.failed_attempts = 0;
            record.blocked_until = None;
        } else {
            // Reset sliding window if older than 60 seconds
            if now.duration_since(record.first_attempt) > Duration::from_secs(60) {
                record.failed_attempts = 0;
                record.first_attempt = now;
            }

            record.failed_attempts += 1;

            // Block for 15 minutes after 5 failed attempts
            if record.failed_attempts >= 5 {
                record.blocked_until = Some(now + Duration::from_secs(900));
            }
        }
    }

    /// Validate an incoming client handshake token with rate limiting and expiration
    pub fn validate_token(&mut self, ip: IpAddr, token: &str, session: &str) -> Result<bool, &'static str> {
        if self.is_ip_blocked(&ip) {
            return Err("ERR_PAIRING_RATE_LIMITED: IP blocked due to repeated failed pairing attempts");
        }

        // Loopback / USB tethering (ADB reverse): local host is trusted
        if ip.is_loopback() && (token == "lnk_usb" || token == self.current_token) {
            self.record_attempt(ip, true);
            return Ok(true);
        }

        if self.is_token_expired() {
            return Err("ERR_QR_TOKEN_EXPIRED: Pairing token has expired. Please refresh the QR code.");
        }

        let is_valid = (!self.current_token.is_empty() && self.current_token == token && self.session_id == session)
            || (!self.verification_pin.is_empty() && (token == self.verification_pin || token == self.verification_pin.replace(' ', "")));

        self.record_attempt(ip, is_valid);

        if !is_valid {
            Err("ERR_AUTH_TOKEN_MISMATCH: Invalid session or authentication token")
        } else {
            Ok(true)
        }
    }

    /// Retrieve all usable local IP addresses for Wi-Fi and direct phone hotspot
    pub fn get_available_ips(&self) -> Vec<String> {
        let mut ips = Vec::new();

        if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
            for (_name, ip) in ifaces {
                if let IpAddr::V4(v4) = ip {
                    if !v4.is_loopback() && !v4.is_link_local() {
                        let ip_str = v4.to_string();
                        if !ips.contains(&ip_str) {
                            ips.push(ip_str);
                        }
                    }
                }
            }
        }

        if ips.is_empty() {
            if let Ok(ip) = local_ip_address::local_ip() {
                ips.push(ip.to_string());
            }
        }

        if !ips.contains(&"127.0.0.1".to_string()) {
            ips.push("127.0.0.1".to_string());
        }

        ips
    }

    pub fn verification_pin(&self) -> &str {
        &self.verification_pin
    }

    /// Generate the pairing payload embedded into the desktop QR code
    pub fn generate_pairing_payload(&self) -> PairingPayload {
        PairingPayload {
            session_id: self.session_id.clone(),
            auth_token: self.current_token.clone(),
            verification_pin: self.verification_pin.clone(),
            port: self.port,
            ips: self.get_available_ips(),
            device_name: self.device_name.clone(),
            protocol_version: linkcam_protocol::PROTOCOL_VERSION,
            expires_in_secs: 120,
            cert_fingerprint: self.cert_fingerprint.clone(),
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
    use std::net::Ipv4Addr;

    #[test]
    fn test_pairing_rotation_and_validation() {
        let mut mgr = PairingManager::new(8443, "Windows-PC".to_string());
        let payload = mgr.generate_pairing_payload();
        let test_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50));

        assert_eq!(payload.port, 8443);
        assert!(!payload.auth_token.is_empty());
        assert!(!payload.session_id.is_empty());
        assert_eq!(payload.expires_in_secs, 120);
        assert!(!payload.verification_pin.is_empty());

        assert!(mgr.validate_token(test_ip, &payload.auth_token, &payload.session_id).unwrap());
        assert!(mgr.validate_token(test_ip, "wrong-token", &payload.session_id).is_err());
        assert!(mgr.validate_token(test_ip, &payload.auth_token, "wrong-session").is_err());

        let old_token = payload.auth_token;
        mgr.rotate_credentials();
        assert!(mgr.validate_token(test_ip, &old_token, &payload.session_id).is_err());
    }

    #[test]
    fn test_pairing_rate_limiting() {
        let mut mgr = PairingManager::new(8443, "Windows-PC".to_string());
        let test_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 99));

        for _ in 0..5 {
            let _ = mgr.validate_token(test_ip, "bad", "bad");
        }

        assert!(mgr.is_ip_blocked(&test_ip));
        let res = mgr.validate_token(test_ip, "token", "session");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("ERR_PAIRING_RATE_LIMITED"));
    }
}
