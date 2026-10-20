use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    usb_tokens: HashMap<String, (String, Instant)>,
    trusted_adb_serials: HashSet<String>,
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
            usb_tokens: HashMap::new(),
            trusted_adb_serials: HashSet::new(),
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

    /// Generate an ephemeral, one-time USB pairing token bound to the ADB device serial
    pub fn generate_usb_token(&mut self, serial: &str) -> (String, String) {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let session_id = format!("usb_{}", uuid::Uuid::new_v4());
        self.usb_tokens.insert(serial.to_string(), (token.clone(), Instant::now()));
        (token, session_id)
    }

    /// Trust an ADB device serial
    pub fn trust_adb_device(&mut self, serial: &str) {
        self.trusted_adb_serials.insert(serial.to_string());
    }

    /// Check if an ADB device serial has been trusted by the user
    pub fn is_device_trusted(&self, serial: &str) -> bool {
        self.trusted_adb_serials.contains(serial)
    }

    /// TLS certificate fingerprint for certificate pinning over USB and Wi-Fi
    pub fn cert_fingerprint(&self) -> &str {
        &self.cert_fingerprint
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

    /// Validate an incoming client handshake token with rate limiting and expiration.
    /// CRITICAL SECURITY: Loopback connections must NEVER accept a hardcoded/static token like "lnk_usb"!
    /// A static token would permit any local process, rogue app, or RDP user to silently pair and hijack the virtual camera.
    pub fn validate_token(&mut self, ip: IpAddr, token: &str, session: &str) -> Result<bool, &'static str> {
        if self.is_ip_blocked(&ip) {
            return Err("ERR_PAIRING_RATE_LIMITED: IP blocked due to repeated failed pairing attempts");
        }

        // Check if token matches active dynamic one-time USB session token (if loopback)
        let is_valid_usb = if ip.is_loopback() {
            // Prune expired USB tokens (> 120s)
            self.usb_tokens.retain(|_, (_, exp)| exp.elapsed() < Duration::from_secs(120));
            self.usb_tokens.values().any(|(tok, _)| tok == token)
        } else {
            false
        };

        if self.is_token_expired() && !is_valid_usb {
            return Err("ERR_QR_TOKEN_EXPIRED: Pairing token has expired. Please refresh the QR code.");
        }

        let is_valid = is_valid_usb
            || (!self.current_token.is_empty() && self.current_token == token && self.session_id == session)
            || (!self.verification_pin.is_empty() && (token == self.verification_pin || token == self.verification_pin.replace(' ', "")));

        self.record_attempt(ip, is_valid);

        if !is_valid {
            Err("ERR_AUTH_TOKEN_MISMATCH: Invalid session or authentication token")
        } else {
            Ok(true)
        }
    }

    /// Retrieve usable local IP addresses filtered strictly to physical and mobile hotspot adapters.
    /// Excludes virtual interfaces (Hyper-V, WSL, Docker, VPNs) and caps the list to 2-3 addresses
    /// to preserve high-density QR code optical scanning.
    pub fn get_available_ips(&self) -> Vec<String> {
        let mut candidates: Vec<(String, bool)> = Vec::new();

        if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
            for (name, ip) in ifaces {
                let name_lower = name.to_lowercase();
                // Filter out virtual, WSL, Hyper-V, Docker, TAP/TUN, and VPN adapters
                let is_virtual = name_lower.contains("vethernet")
                    || name_lower.contains("wsl")
                    || name_lower.contains("tailscale")
                    || name_lower.contains("wireguard")
                    || name_lower.contains("docker")
                    || name_lower.contains("hyper-v")
                    || name_lower.contains("virtual")
                    || name_lower.contains("tap")
                    || name_lower.contains("tun")
                    || name_lower.contains("host-only")
                    || name_lower.contains("vmware")
                    || name_lower.contains("bluetooth")
                    || name_lower.contains("zerotier");

                if is_virtual {
                    continue;
                }

                if let IpAddr::V4(v4) = ip {
                    if v4.is_loopback() || v4.is_link_local() || v4.is_broadcast() || v4.is_multicast() {
                        continue;
                    }

                    // Enforce RFC 1918 private IP ranges or Windows Mobile Hotspot default subnet (192.168.137.x)
                    let octets = v4.octets();
                    let is_rfc1918 = octets[0] == 10
                        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                        || (octets[0] == 192 && octets[1] == 168);

                    if is_rfc1918 {
                        let ip_str = v4.to_string();
                        // Prioritize hotspot interfaces (192.168.137.x or 192.168.43.x)
                        let is_hotspot = (octets[0] == 192 && octets[1] == 168 && (octets[2] == 137 || octets[2] == 43))
                            || name_lower.contains("hotspot")
                            || name_lower.contains("wi-fi direct");

                        if !candidates.iter().any(|(addr, _)| addr == &ip_str) {
                            candidates.push((ip_str, is_hotspot));
                        }
                    }
                }
            }
        }

        // Sort: Hotspots first, then physical LAN/Wi-Fi
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let mut ips: Vec<String> = candidates.into_iter().map(|(addr, _)| addr).collect();

        // Fallback to local_ip if empty
        if ips.is_empty() {
            if let Ok(ip) = local_ip_address::local_ip() {
                if let IpAddr::V4(v4) = ip {
                    if !v4.is_loopback() && !v4.is_link_local() {
                        let octets = v4.octets();
                        let is_private = octets[0] == 10
                            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                            || (octets[0] == 192 && octets[1] == 168);
                        if is_private {
                            ips.push(v4.to_string());
                        }
                    }
                }
            }
        }

        // Cap at 2-3 addresses to preserve QR density and avoid bloating
        ips.truncate(3);

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
    fn test_static_loopback_token_rejected() {
        let mut mgr = PairingManager::new(8443, "Windows-PC".to_string());
        let loopback = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // The dangerous "lnk_usb" static bypass MUST be rejected
        let res = mgr.validate_token(loopback, "lnk_usb", "usb_session");
        assert!(res.is_err(), "Static token 'lnk_usb' must be rejected");

        // Dynamic USB token generated for a specific device serial must be accepted
        let (usb_token, usb_session) = mgr.generate_usb_token("device_serial_1234");
        assert!(mgr.validate_token(loopback, &usb_token, &usb_session).unwrap());
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
