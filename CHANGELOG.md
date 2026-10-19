# Changelog

All notable changes to **LinkCam** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3] - 2026-10-19

### Fixed
- **Desktop Companion Type Checking**: Fixed all TS6133 (`noUnusedLocals`) compiler errors across `CameraControls.tsx`, `CapturesGallery.tsx`, `LiveStudio.tsx`, `PairingWizard.tsx`, `QuickCaptureTray.tsx`, `SettingsView.tsx`, and `StreamHUD.tsx`. Integrated hardware-accelerated Direct3D 11 decoding toggle into Settings UI.
- **Android Gradle Dependency Resolution**: Resolved `kotlin-stdlib:1.9.23` resolution failure by declaring `kotlin-bom:1.9.23`, adding `gradlePluginPortal()` fallback mirror in `settings.gradle.kts`, and disabling flaky HTTP keep-alive connection pooling in CI via `gradle.properties`.

## [1.0.2] - 2026-10-19

### Fixed
- **Android CI/CD SDK Setup**: Removed deprecated `android-actions/setup-android@v3` which failed on missing legacy `tools` package on modern Ubuntu runners. Configured native GitHub Actions Android SDK environment with direct license acceptance, platform preparation, and Gradle 8.4 via `gradle/actions/setup-gradle@v3`.
- **Automatic SDK Component Resolution**: Enabled `android.builder.sdkDownload=true` in `gradle.properties`.

## [1.0.1] - 2026-10-19

### Security & Hardening
- **Windows Clipboard Injection Protection**: Eliminated PowerShell script string interpolation in `clipboard.rs`; now pipes literal path via stdin to `[Console]::In.ReadLine()`.
- **Path Traversal & Device Name Defense**: Added strict character allowlisting and Windows DOS device name rejection in `storage.rs`, with boundary assertions preventing writes outside `~/Pictures/LinkCam/`.
- **Packet Memory Bounds & DoS Prevention**: Enforced `MAX_PAYLOAD_SIZE` (16MB) validation on all incoming packet headers across both Rust server and Android Kotlin client.
- **Capture Receiver TTL & Rate Limiting**: Added 5-minute TTL expiration for in-flight chunk transfers and capped concurrent active transfers to 50 with max 10,000 chunks.
- **TCP Stream Framing Safety**: Implemented `readFully` in Kotlin `SocketTransport` to prevent truncated buffer reads and framing corruption.
- **Tauri Content Security Policy (CSP)**: Replaced disabled `csp: null` with strict policy restricting script/style/font/socket connections to verified origins.
- **Android Release Minification**: Added `proguard-rules.pro` with keep rules for CameraX, Kotlin Coroutines, Gson models, and ML Kit.
- **CI/CD Build Pipeline**: Updated `.github/workflows/release.yml` with `gradle/actions/setup-gradle@v3` and direct `cargo build --release` to ensure both Android APK and Windows x64 binary are compiled and published to GitHub Releases.
- **Repository Licensing**: Added official MIT License.

## [1.0.0] - 2026-10-19

### Added
- **Core Binary Protocol**: 24-byte packet framing (`LNKC`), control RPC schema, telemetry heartbeat, and chunked file transfer envelopes.
- **Windows Companion Engine (Tauri v2 + Rust)**:
  - Low-memory async TCP/WebSocket server on port `8443`.
  - Windows Media Foundation Virtual Camera integration (`MFCreateVirtualCamera`) recognized natively by Zoom, Microsoft Teams, Google Meet, and OBS.
  - Automatic Windows clipboard injector and timestamped local file storage (`~/Pictures/LinkCam/`).
- **Windows 11 Fluent 2 Interface**:
  - Live Studio viewfinder with composition grid and interactive camera control dock.
  - Real-time telemetry HUD displaying live FPS, bitrate (Mbps), frame latency (ms), phone battery %, and virtual camera status.
  - Multi-lens camera controls (Rear Wide 1x, Ultra-Wide 0.6x, Telephoto 3x, Front), resolution switcher, torch toggle, zoom slider, and exposure compensation.
  - Pairing Wizard with dynamic QR code generation, step-by-step Zero-Router Hotspot guide, and USB wired detection.
  - Quick-Capture Tray with floating slide-over notifications and direct clipboard feedback.
- **Native Android Client (Kotlin + Jetpack Compose)**:
  - Targeted for modern 64-bit devices (`ARM64-v8a`, Android 10+ / API 29+).
  - CameraX video streaming pipeline with hardware-accelerated frame analysis.
  - Dual transport manager supporting Direct Mobile Hotspot (`192.168.43.1`), local Wi-Fi, and USB loopback.
  - Instant Document Scanner with automatic edge detection, perspective transformation, and multi-page PDF generation.
  - OLED Meeting Power-Save Mode that blanks display pixels during long calls to prevent thermal throttling and save battery while streaming in the background.
  - Persistent Foreground Service with wake-lock to prevent OS task killing.
- **Zero AI Slop Guarantee**: Full compliance with Impeccable craftsmanship standards (no repetitive generic cards, no gradient text, no emoji icons, and authentic Fluent 2 / Material 3 type scales).
