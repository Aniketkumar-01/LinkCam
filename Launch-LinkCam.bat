@echo off
setlocal
if exist "desktop\src-tauri\target\release\linkcam-desktop.exe" (
    start "" "desktop\src-tauri\target\release\linkcam-desktop.exe"
) else if exist "artifacts\LinkCam-Windows-Portable.exe" (
    start "" "artifacts\LinkCam-Windows-Portable.exe"
) else (
    echo [*] Launching LinkCam Companion Native Desktop App...
    cargo run --release --manifest-path desktop/src-tauri/Cargo.toml
)
