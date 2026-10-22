@echo off
setlocal
cd /d "%~dp0"

REM Ensure DirectShow Virtual Camera DLL is built and registered for OBS/Zoom/Meet
if not exist "desktop\vcam-dshow\LinkCamDShow64.dll" (
    echo [*] DirectShow virtual camera DLL not found. Building and registering...
    call "desktop\vcam-dshow\build.bat"
)

if exist "artifacts\LinkCam-Windows-Portable.exe" (
    start "" "artifacts\LinkCam-Windows-Portable.exe"
    exit /b 0
)

if exist "desktop\src-tauri\target\release\linkcam-desktop.exe" (
    start "" "desktop\src-tauri\target\release\linkcam-desktop.exe"
    exit /b 0
)

echo [*] LinkCam desktop binary not found. Preparing build...
if not exist "desktop\dist\index.html" (
    echo [*] Building web frontend...
    cd desktop
    call npm install
    call npm run build
    cd ..
)

echo [*] Launching LinkCam Companion Native Desktop App...
cargo run --release --manifest-path desktop/src-tauri/Cargo.toml
