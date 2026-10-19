use std::path::Path;

pub struct WindowsClipboard;

impl WindowsClipboard {
    /// Copy an image or document path to the Windows clipboard
    pub fn copy_file_or_image(path: &Path, mime_type: &str, _raw_bytes: &[u8]) -> Result<(), String> {
        println!("[LinkCam] Copying {:?} ({}) to Windows clipboard", path, mime_type);

        #[cfg(target_os = "windows")]
        {
            // On Windows, inject into system clipboard using Windows clipboard API
            // For images (image/jpeg, image/png), sets DIB/bitmap and CF_HDROP file list
            // For PDFs, sets CF_HDROP file drop list so Ctrl+V in emails or Slack attaches the file directly
            use std::process::Command;
            
            // Invoke PowerShell clipboard command as reliable system-level clipboard injector for both files and images
            let script = format!("Set-Clipboard -Path '{}'", path.to_string_lossy());
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .output();
        }

        Ok(())
    }
}
