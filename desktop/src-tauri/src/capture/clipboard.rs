use std::path::Path;

pub struct WindowsClipboard;

impl WindowsClipboard {
    /// Copy an image or document path to the Windows clipboard
    pub fn copy_file_or_image(path: &Path, mime_type: &str, _raw_bytes: &[u8]) -> Result<(), String> {
        println!("[LinkCam] Copying {:?} ({}) to Windows clipboard", path, mime_type);

        if !path.exists() || !path.is_file() {
            return Err(format!("File does not exist or is not a regular file: {:?}", path));
        }

        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            let path_str = path.to_string_lossy();
            if path_str.contains('\r') || path_str.contains('\n') {
                return Err("Path contains invalid newline characters".to_string());
            }

            // Security Hardening:
            // Do not use string interpolation in powershell command scripts.
            // Pipe the raw literal path through stdin and read via [Console]::In.ReadLine().
            // This prevents command injection via crafted file names or paths.
            let child = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Set-Clipboard -LiteralPath ([Console]::In.ReadLine())",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match child {
                Ok(mut proc) => {
                    if let Some(mut stdin) = proc.stdin.take() {
                        let _ = writeln!(stdin, "{}", path_str);
                    }
                    let _ = proc.wait();
                }
                Err(e) => {
                    eprintln!("[LinkCam] Failed to launch PowerShell clipboard injector: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
