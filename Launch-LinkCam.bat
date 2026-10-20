@echo off
setlocal
title LinkCam Companion Launcher

echo [*] Starting LinkCam Windows Companion...

REM Check if release binary exists
set BIN=desktop\src-tauri\target\release\linkcam-desktop.exe
if not exist "%BIN%" (
    set BIN=artifacts\LinkCam-Windows-Portable.exe
)

if not exist "%BIN%" (
    echo [!] Binary not found at standard path. Building/Starting via cargo...
    start /min "" cmd /c "cargo run --release --manifest-path desktop/src-tauri/Cargo.toml"
) else (
    echo [*] Launching LinkCam Engine in background...
    start /min "" "%BIN%"
)

REM Wait 1 second for engine initialization
timeout /t 1 /nobreak >nul

REM Launch modern borderless standalone UI window
echo [*] Opening LinkCam Live Studio UI...
start "" msedge.exe --app=http://127.0.0.1:8443 --window-size=1180,780 --user-data-dir="%LOCALAPPDATA%\LinkCam\ui_profile"

exit /b 0
