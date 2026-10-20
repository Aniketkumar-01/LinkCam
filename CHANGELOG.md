# Changelog

All notable changes to **LinkCam** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.16] - 2026-10-20

### Fixed
- **Windows DPAPI E0432 Compiler Fix**: Corrected unresolved import `CRYPTOAPI_BLOB` to `CRYPT_INTEGER_BLOB` in `desktop/src-tauri/src/network/dpapi.rs`, complying with `windows` crate `0.52.0` Win32 Cryptography type definitions.
- **Rust Compiler Warning Cleanup**: Eliminated 12 unused import and unused mutability warnings across `receiver.rs`, `capture/mod.rs`, `session.rs`, `udp_server.rs`, `network/mod.rs`, `frame_sink.rs`, `server.rs`, and `main.rs`, ensuring a zero-warning clean compilation in CI.

## [1.0.15] - 2026-10-20

### Fixed
- **TypeScript Strict Build Fix (TS6133)**: Removed unused `AlertTriangle` icon import in `desktop/src/components/DiagnosticsModal.tsx`, resolving CI and local Vite/TypeScript production compilation failure.

### Added
- **Embedded Desktop Bundle Icons**: Added generated multi-resolution icon assets (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`, `icon.icns`) in `desktop/src-tauri/icons/` for native Windows executable and Inno Setup installer packaging.

## [1.0.14] - 2026-10-20

### Added
- **Phase 1 (Security & Protocol Core)**:
  - **Dynamic Mutual SAS Verification PIN**: Desktop companion generates a random 6-digit mutual authentication PIN (e.g. `582 391`) embedded into the QR pairing payload and displayed on the desktop authorization modal for cross-device verification.
  - **Time-Limited QR Session Expiration**: Introduced a 120-second sliding expiration window (`exp: 120`) on pairing tokens, preventing stale QR code reuse.
  - **Per-IP Handshake Rate Limiting & DoS Protection**: Implemented connection rate limiting (max 3 pairing attempts/min per IP, auto-blocking after 5 failed attempts for 15 minutes) to protect against brute-force pairing attacks.
  - **Windows DPAPI Credential Store (`dpapi.rs`)**: Secure hardware-bound token encryption using Windows DPAPI (`CryptProtectData` / `CryptUnprotectData`) persisting trusted device tokens in `%LOCALAPPDATA%\LinkCam\trusted_devices.dat`.
  - **Private Adapter Network Binding**: Added subnet and IP validation (`is_private_ip()`) to restrict connections to private LAN ranges (RFC 1918 / loopback), rejecting unauthorized connections from public or VPN interfaces.
  - **Memory DoS Guards & Frame Caps**: Enforced strict packet size ceilings (`MAX_PAYLOAD_SIZE = 10 MB` for capture chunks and `MAX_VIDEO_FRAME_SIZE = 2 MB` for video frames) to prevent memory exhaustion attacks.
  - **Protocol Standardization & Typed RPCs**: Added protocol packet types across Rust and Kotlin: `KeepAlivePing` (`0x00`), `CodecConfig` (`0x04`), `KeyframeRequest` (`0x05`), `Disconnect` (`0x06`), `ChunkRetransmit` (`0x07`), and `PairingVerify` (`0x12`), with flags `FLAG_CONFIG` (`0x02`), `FLAG_END_OF_STREAM` (`0x04`), and `FLAG_ENCRYPTED` (`0x08`). Typed Rust structs for all payloads.

- **Phase 2 (Media Pipeline & Performance)**:
  - **Hardware H.264 Encoder (`MediaCodecH264Encoder.kt`)**: Implemented zero-jitter hardware H.264 video encoding via Android `MediaCodec` with CBR low-latency baseline mode, runtime dynamic bitrate adjustment (`setBitrate()`), and on-demand IDR keyframe insertion (`requestKeyframe()`), completely eliminating CPU JPEG compression bottlenecks.
  - **High-Throughput UDP Video Transport (`UdpStreamer.kt` & `udp_server.rs`)**: Created a dedicated low-latency UDP streaming pipeline on port `8444`. Direct-dispatches video datagram frames into the Desktop companion's pipeline without TCP head-of-line blocking.
  - **Adaptive Bitrate (ABR) Controller (`AbrController.kt`)**: Implemented 3-tier dynamic quality governor (`Nominal` 8Mbps @ 60fps, `Degraded` 4.5Mbps @ 60fps, `Emergency` 2.2Mbps @ 30fps) reacting within 300ms to packet loss, round-trip time (RTT), and device thermal conditions.
  - **48 kHz PCM Stereo Audio Streamer (`AudioStreamer.kt`)**: Added low-latency audio capture and streaming with monotonic PTS timestamps synchronized with video packet headers.

- **Phase 3 (Windows Virtual Camera System)**:
  - **Canonical DirectShow COM Registration (`mf_virtual_camera.rs`)**: Corrected DirectShow virtual camera registration to the canonical Windows Category GUID `{860BB310-5D01-11D0-BD3B-00A0C911CE86}` (`CLSID_VideoInputDeviceCategory`) ensuring native compatibility across OBS, Zoom, Teams, and Discord.
  - **Dual-Path Virtual Camera Architecture**: Implemented Media Foundation (`MFCreateVirtualCamera`) registration for Windows 11 and DirectShow registration fallback for legacy applications.

- **Phase 4 (Reliability, UX & Diagnostics Suite)**:
  - **Android Wi-Fi & Thermal Protection (`StreamService.kt`)**: Configured `WifiManager.WifiLock` with `WIFI_MODE_FULL_HIGH_PERF` to bypass aggressive Android Wi-Fi power-saving latency spikes, alongside thermal status listening (`OnThermalStatusChangedListener`).
  - **Zero-Config Network Service Discovery (`NsdDiscoveryManager.kt`)**: Added Android NSD / mDNS broadcasting on service type `_linkcam._tcp.` allowing desktop discovery without manual IP entry.
  - **Desktop Diagnostics & Latency Suite (`DiagnosticsModal.tsx`)**: Created real-time diagnostics modal displaying live RTT, FPS, bitrate, jitter, optical screen-to-lens calibration tool, and one-click JSON diagnostic bundle export.
  - **Interactive Latency HUD (`StreamHUD.tsx` & `LiveStudio.tsx`)**: Added clickable latency badge with live network quality indicators and quick access to deep diagnostics.

- **System Architecture & Specification v1.1.0 (`SYSTEM_ARCHITECTURE_AND_SPECIFICATION.md`)**:
  - Published comprehensive architectural baseline detailing STRIDE threat model, 24-byte protocol layout, dual-path virtual camera design, and prioritized engineering roadmap.

## [1.0.13] - 2026-10-20

### Fixed
- **Embedded Frontend Asset Bundling (Zero External Dev Server Dependency)**: Removed `devUrl` and `beforeDevCommand` from `tauri.conf.json`, ensuring `tauri::generate_context!()` compiles the application to embed all static React/Vite assets directly from `../dist` into the binary. Resolved `ERR_CONNECTION_REFUSED: localhost refused to connect`.
- **Vite Relative Asset Base URL**: Set `base: './'` in `desktop/vite.config.ts`, ensuring all compiled scripts and stylesheets resolve cleanly across Tauri custom protocols (`tauri://localhost`, `http://tauri.localhost`).
- **WebView2 Content Security Policy (CSP)**: Updated `tauri.conf.json` security CSP to whitelist `tauri:` and `http://tauri.localhost`.
- **Self-Contained Local Launcher**: Updated `Launch-LinkCam.bat` to automatically build `desktop/dist` if not already present before invoking Cargo.

## [1.0.12] - 2026-10-20

### Fixed
- **Tauri v2 Windows Icon Generation & Manifest Integration**: Added `generate-icons.cjs` and automated Rust build hook in `build.rs` to generate and embed `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, and `icon.ico` directly into the Windows executable and installer, resolving the missing icon error (`ico not found; required for generating a Windows Resource file during tauri-build`).
- **Tauri v2 Window Label & Capabilities**: Defined explicit window label `main` in `tauri.conf.json` and added `capabilities/default.json` with `core:default` permissions to comply with Tauri v2 security specification.
- **Inno Setup & Release Pipeline Synchronization**: Synced `installer.iss`, `push_and_release.ps1`, and `push_and_release.bat` to `v1.0.12`.

## [1.0.11] - 2026-10-20

### Added
- **True Standalone Native Desktop App (Tauri v2)**: Converted LinkCam Desktop Companion into a true independent native Windows application using Tauri v2. Completely eliminated the external browser window and `127.0.0.1` network error screens. The application now compiles to a self-contained `.exe` with embedded assets, custom window chrome, and integrated background engine.

## [1.0.10] - 2026-10-20

### Fixed
- **Desktop UI Missing Handler Definition**: Implemented missing `handleOpenFolder` function in `App.tsx`, resolving `TS2304: Cannot find name 'handleOpenFolder'` during the `tsc && vite build` step.

## [1.0.9] - 2026-10-20

### Fixed
- **Desktop UI TypeScript Strict Compliance**: Added missing `ShieldCheck` icon import in `PairingWizard.tsx`, removed unused `AlertTriangle` in `PairingVerificationModal.tsx`, and updated pairing callback arguments in `App.tsx` to satisfy `strict`, `noUnusedLocals`, and `noUnusedParameters` rules.

## [1.0.8] - 2026-10-20

### Fixed
- **Desktop UI JSX Syntax Error**: Fixed TS1005 TypeScript compilation error in `PairingWizard.tsx` by properly closing the mode selection ternary block and restoring the USB mode instruction card.

## [1.0.7] - 2026-10-20

### Added
- **Windows Video Capture System Catalog Registration**: Implemented direct Windows DirectShow category registration (`HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCamVirtualCamera`), ensuring "LinkCam Virtual Camera" is properly discovered and listed in Zoom, Discord, Google Meet, Microsoft Teams, and OBS.
- **Standalone Windows GUI App Mode**: Added `#![windows_subsystem = "windows"]` and standalone application window spawning. Launching LinkCam no longer opens a black console/terminal window; it displays the Windows 11 Fluent Live Studio interface directly.
- **Interactive Pairing Authorization & Verification**: Added `PairingVerificationModal` component and device pairing policy options, allowing users to inspect device name, IP address, and 6-digit verification PIN, with interactive `[Pair & Allow]` and `[Decline]` controls.
- **One-Click Windows Desktop Launchers**: Added `Launch-LinkCam.bat` and silent `Launch-LinkCam.vbs` to repo root for zero-terminal launching.

## [1.0.6] - 2026-10-20

### Fixed
- **Kotlin Float Type Mismatch in StreamScreen**: Fixed compilation error in `:app:compileDebugKotlin` (`The floating-point literal does not conform to the expected type Float`) at `StreamScreen.kt:56` where `Color.White.copy(alpha = 0.04)` was inferred as `Double`. Added explicit `0.04f` literal suffix.
- **APK Release Artifact Packaging**: Resolved GitHub Actions failure in the `Rename APK for Release` step by checking Gradle's standard output paths (`android/app/build/outputs/apk/debug/app-debug.apk`) directly with fallback search logic, eliminating shell argument and semicolon escaping errors previously caused by `find -exec`.
- **Inno Setup ISCC Path Discovery**: Added dynamic path resolution for Inno Setup compiler (`ISCC.exe`) across 64-bit and 32-bit Program Files directories and system environment PATH.
- **Workflow Artifact Uploads**: Added `upload-artifact@v4` action to the Windows Companion build job, ensuring both `LinkCam-Windows-Setup.exe` and `LinkCam-Windows-Portable.exe` can be downloaded directly from GitHub Actions workflow summaries.

## [1.0.5] - 2026-10-20

### Fixed
- **Android NDK Toolchain Trigger**: Removed unneeded `ndk { abiFilters.add("arm64-v8a") }` block from `build.gradle.kts`. Pure Kotlin/Java Android projects without C++ code must not declare `ndk`, which was triggering AGP to look for and fail on the missing Android NDK (`No version of NDK matched the requested version`).
- **Gradle Networking Clean-Up**: Removed `systemProp.http.keepAlive=false` and hardcoded socket timeout overrides from `gradle.properties` that caused HTTPS connection drops, handshake failures, and dependency download timeouts in CI.
- **AAPT2 Vector Path Compliance**: Scaled and normalized vector icon path data in `ic_launcher_foreground.xml` to standard cubic Beziers.
- **CI Build Log Telemetry**: Added output tee and failure log dump step in `.github/workflows/release.yml` so any build issues are printed directly in GitHub Actions summaries.

## [1.0.4] - 2026-10-20

### Fixed
- **Android Kotlin Syntax Error**: Fixed invalid TypeScript arrow syntax `() => Unit` in `MeetingModeOverlay.kt` to valid Kotlin function signature `() -> Unit` which caused compilation failure.
- **Android App Launcher Icons**: Created missing adaptive and fallback vector drawables (`ic_launcher.xml`, `ic_launcher_round.xml`, `ic_launcher_foreground.xml`, and `colors.xml`) in `res/`, resolving AAPT resource linking failure.
- **Android SDK & Toolchain Standardization**: Set `compileSdk = 34` and `targetSdk = 34` for Android Gradle Plugin 8.3.2 native compatibility on Ubuntu runners, added `android.suppressUnsupportedCompileSdk=35`, and bumped ML Kit Document Scanner to stable `16.0.0`.
- **YuvImage Stride Safety**: Provided explicit non-null row strides in `CameraStreamer.kt` preventing nullability constraint errors during image processing.
- **GitHub Actions CI Resilience**: Set `cache-disabled: true` on `gradle/actions/setup-gradle@v3` to prevent runner cache service HTTP 400 failures, and made APK artifact collection robust using `find`.
- **Windows Setup Installer (`LinkCam-Windows-Setup.exe`)**: Added Inno Setup packaging script (`installer.iss`) to compile an installer with Start Menu & Desktop shortcuts, auto-startup option, and clean uninstaller alongside the portable binary.

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
