# Task List: LinkCam

## Phase 1: Protocol & Network Foundation

### Task 1: Wire Protocol & Serialization Specification
**Description:** Define the binary frame format, packet headers, control RPC schema, and payload envelopes for video frames, camera control commands, and instant capture files.
**Acceptance criteria:**
- [x] Binary packet layout defined with magic header `LNKC`, 1-byte packet type, 8-byte timestamp, 4-byte payload length, and payload.
- [x] Packet types defined for Video Frame (`0x01`), Audio Frame (`0x02`), Instant Capture Chunk (`0x03`), Pairing Auth (`0x10`), Camera Control (`0x20`), and Telemetry Heartbeat (`0x30`).
- [x] Control RPC schemas defined for resolution, camera lens, torch, zoom, and capture trigger.
**Verification:**
- [x] Unit tests for packet serialization and parsing pass.
**Dependencies:** None
**Files likely touched:**
- `protocol/src/packet.rs`
- `protocol/schemas/protocol.json`
- `protocol/README.md`
**Estimated scope:** S (2 files)

---

### Task 2: Network Server Engine (Rust Core)
**Description:** Implement high-performance async TCP/WebSocket server in Rust using Tokio, handling multiple stream channels, client authentication, and bounded frame queues.
**Acceptance criteria:**
- [x] Async server listens on configurable port (default `8443`) supporting both WebSocket and raw TCP socket streaming.
- [x] Handles client connection, session token validation, and graceful disconnect cleanup.
- [x] Non-blocking frame dispatcher with zero memory leaks and backpressure control.
**Verification:**
- [x] `cargo test` passes for network server module.
**Dependencies:** Task 1
**Files likely touched:**
- `desktop/src-tauri/src/network/server.rs`
- `desktop/src-tauri/src/network/session.rs`
**Estimated scope:** M (3 files)

---

### Task 3: Pairing Handshake & Hotspot Discovery Protocol
**Description:** Implement the pairing workflow including QR code generation with cryptographic session tokens and local mDNS discovery (`_linkcam._tcp`).
**Acceptance criteria:**
- [x] Generates dynamic QR code payload with local IP addresses (including direct hotspot IP `192.168.43.x`), port, and ephemeral AES session key.
- [x] Handshake verifies mutual authentication before video frames or capture files are accepted.
- [x] Supports auto-reconnect if Wi-Fi network drops or switches to hotspot.
**Verification:**
- [x] Test client handshake passes with valid and invalid tokens.
**Dependencies:** Task 2
**Files likely touched:**
- `desktop/src-tauri/src/network/pairing.rs`
- `desktop/src-tauri/src/network/discovery.rs`
**Estimated scope:** S (2 files)

---

## Checkpoint 1: Protocol & Network Foundation
- [x] All protocol unit tests pass.
- [x] Handshake and network frame throughput verified.

---

## Phase 2: Windows Companion App & Virtual Camera Bridge

### Task 4: Tauri Project Scaffold & Native Rust Modules
**Description:** Initialize the Tauri v2 desktop companion project with Rust backend, Tauri IPC commands, and modern web frontend build configuration.
**Acceptance criteria:**
- [x] Tauri v2 project structure established with `Cargo.toml`, Tauri config, and Vite React frontend.
- [x] Tauri IPC commands exposed for pairing state, camera controls, capture history, and system settings.
**Verification:**
- [x] `npm run tauri build` or project check compiles without errors.
**Dependencies:** Task 1, Task 2, Task 3
**Files likely touched:**
- `desktop/package.json`
- `desktop/src-tauri/Cargo.toml`
- `desktop/src-tauri/tauri.conf.json`
- `desktop/src-tauri/src/main.rs`
**Estimated scope:** M (4 files)

---

### Task 5: Windows Virtual Camera Pipeline
**Description:** Implement the Windows Virtual Camera driver bridge using Windows Media Foundation (`MFCreateVirtualCamera`) and DirectShow filter pipeline to register LinkCam as a system-recognized webcam.
**Acceptance criteria:**
- [x] Virtual camera registered in Windows Media Foundation device catalog with friendly name "LinkCam Virtual Camera".
- [x] Frame sink receives decoded video frames (NV12 / YUY2 / RGB32) and feeds them into the active media stream for Zoom, Teams, Google Meet, and OBS.
- [x] Clean teardown when companion app closes or streaming pauses.
**Verification:**
- [x] Device enum test verifies "LinkCam Virtual Camera" presence.
**Dependencies:** Task 4
**Files likely touched:**
- `desktop/src-tauri/src/vcam/mod.rs`
- `desktop/src-tauri/src/vcam/mf_virtual_camera.rs`
- `desktop/src-tauri/src/vcam/frame_sink.rs`
**Estimated scope:** M (3 files)

---

### Task 6: Instant Capture Receiver, Clipboard Injector & Notification Engine
**Description:** Implement desktop file receiver for instant captures and document scans, with automatic Windows clipboard injection, file organization, and interactive Windows desktop toasts.
**Acceptance criteria:**
- [x] Reassembles received image and PDF chunks with SHA-256 validation.
- [x] Automatically places captured image onto Windows system clipboard (allowing instant Ctrl+V into Slack, Teams, Word, Discord).
- [x] Saves timestamped file into `~/Pictures/LinkCam/` or `~/Documents/LinkCam/`.
- [x] Emits desktop event with preview thumbnail and quick actions.
**Verification:**
- [x] Mock capture payload triggers clipboard copy and file write successfully.
**Dependencies:** Task 4
**Files likely touched:**
- `desktop/src-tauri/src/capture/receiver.rs`
- `desktop/src-tauri/src/capture/clipboard.rs`
- `desktop/src-tauri/src/capture/notifications.rs`
**Estimated scope:** M (3 files)

---

### Task 7: Impeccable Windows 11 Fluent UI (Studio, Stream HUD & Controls)
**Description:** Build the Windows 11 desktop frontend following Impeccable standards with Mica/Acrylic materials, Segoe UI Variable typography, live video canvas, telemetry HUD, and tactile camera control panels.
**Acceptance criteria:**
- [x] Zero AI slop: no generic icon cards, no gradient text, authentic Fluent 2 styling with translucent Mica/Acrylic depth and subtle borders.
- [x] Real-time video player canvas with aspect ratio locking and low-latency rendering.
- [x] Telemetry HUD: live FPS, bitrate (Mbps), latency (ms), phone battery percentage, connection badge (USB ⚡ / Wi-Fi 📶 / Hotspot 📱).
- [x] Tactile camera controls: lens switcher (Wide / Ultra-wide / Front), resolution selector (1080p60 / 1080p30 / 4K), torch toggle, zoom slider, exposure slider, mirror/flip.
**Verification:**
- [x] Frontend builds cleanly with zero lint errors and responsive layout across desktop window sizes.
**Dependencies:** Task 4, Task 5
**Files likely touched:**
- `desktop/src/App.tsx`
- `desktop/src/components/LiveStudio.tsx`
- `desktop/src/components/CameraControls.tsx`
- `desktop/src/components/StreamHUD.tsx`
- `desktop/src/styles/fluent.css`
**Estimated scope:** M (5 files)

---

### Task 8: Pairing Wizard & Interactive Quick-Capture Tray
**Description:** Build the desktop pairing interface (QR code scanner, Hotspot step-by-step guidance, USB auto-detect) and interactive Quick-Capture Tray with drag-and-drop preview.
**Acceptance criteria:**
- [x] Pairing wizard displays high-contrast crisp QR code and zero-router Hotspot instructions.
- [x] Quick-Capture Tray shows recent captures with actions: "Copy", "Open PDF", "Show in Folder", "Drag to active app".
- [x] System tray icon with quick pause/resume streaming and disconnect toggles.
**Verification:**
- [x] UI interaction tests verify tray actions and QR code generation.
**Dependencies:** Task 6, Task 7
**Files likely touched:**
- `desktop/src/components/PairingWizard.tsx`
- `desktop/src/components/QuickCaptureTray.tsx`
- `desktop/src/components/Sidebar.tsx`
**Estimated scope:** M (3 files)

---

## Checkpoint 2: Desktop Companion
- [x] Desktop app launches with full Fluent 2 UI.
- [x] Virtual camera bridge, preview renderer, and instant capture receiver operational.

---

## Phase 3: Android Native Client

### Task 9: Android Project Scaffold
**Description:** Set up native Android project targeting modern ARM64-v8a devices (Android 10+, API 29+) with Kotlin, Jetpack Compose, CameraX, and Gradle Kotlin DSL.
**Acceptance criteria:**
- [x] Android build configuration targeting API 29-35, ARM64-v8a ABI filter, Kotlin 1.9+, Compose Compiler.
- [x] Permissions defined in AndroidManifest for Camera, Wi-Fi / Hotspot state, Foreground Service, and Notifications.
**Verification:**
- [x] Gradle build configuration resolves without errors.
**Dependencies:** Task 1
**Files likely touched:**
- `android/build.gradle.kts`
- `android/app/build.gradle.kts`
- `android/app/src/main/AndroidManifest.xml`
**Estimated scope:** S (3 files)

---

### Task 10: CameraX Video Streamer & MediaCodec Pipeline
**Description:** Implement low-latency camera streaming using CameraX `ImageAnalysis` and hardware `MediaCodec` H.264 / MJPEG encoder in a persistent Android Foreground Service.
**Acceptance criteria:**
- [x] CameraX binds to lifecycle with configurable resolution (1080p/4K) and target framerate (30/60 fps).
- [x] Multi-lens support: queries and toggles between rear primary, ultra-wide, telephoto, and selfie cameras.
- [x] Hardware-accelerated frame encoding with sub-30ms frame encoding latency.
- [x] Remote control listener: executes PC commands for torch, zoom, exposure, and lens switching.
**Verification:**
- [x] Camera streamer captures frames and produces encoded packets.
**Dependencies:** Task 9
**Files likely touched:**
- `android/app/src/main/java/com/linkcam/app/camera/CameraStreamer.kt`
- `android/app/src/main/java/com/linkcam/app/camera/MediaCodecEncoder.kt`
- `android/app/src/main/java/com/linkcam/app/service/StreamService.kt`
**Estimated scope:** M (3 files)

---

### Task 11: Transport Layer (Hotspot, Wi-Fi, USB Socket Client)
**Description:** Implement Android network transport supporting automatic Hotspot IP detection, Wi-Fi connection, and high-speed USB socket client.
**Acceptance criteria:**
- [x] Detects active connection type (Direct Hotspot `192.168.43.1`, local Wi-Fi, or USB loopback/tethering).
- [x] Connects to PC server using async non-blocking coroutines, performs AES-GCM token handshake, and streams video packets with automatic reconnection.
**Verification:**
- [x] Transport unit tests pass for packet dispatching and connection state transitions.
**Dependencies:** Task 9, Task 1
**Files likely touched:**
- `android/app/src/main/java/com/linkcam/app/transport/SocketTransport.kt`
- `android/app/src/main/java/com/linkcam/app/transport/ConnectionManager.kt`
**Estimated scope:** S (2 files)

---

### Task 12: Instant Capture & Document Auto-Crop / PDF Engine
**Description:** Build the mobile document scanner and high-res capture module with automatic edge detection, perspective transformation, and multi-page PDF generation.
**Acceptance criteria:**
- [x] Document scanner detects paper boundaries, straightens perspective, and applies readability filters (Clean Document / Color / Grayscale).
- [x] High-resolution photo capture option using CameraX `ImageCapture`.
- [x] Converts scanned pages to optimized PDF document and sends directly to PC companion over active socket.
**Verification:**
- [x] Document capture test generates valid PDF and transmits chunked payload.
**Dependencies:** Task 10, Task 11
**Files likely touched:**
- `android/app/src/main/java/com/linkcam/app/scanner/DocumentScanner.kt`
- `android/app/src/main/java/com/linkcam/app/scanner/PdfGenerator.kt`
**Estimated scope:** S (2 files)

---

### Task 13: Impeccable Material 3 Android UI & Meeting Power-Save Mode
**Description:** Create the Android Jetpack Compose user interface following Material Design 3 guidelines with edge-to-edge layout, QR scanner, tactile streaming controls, and an OLED Meeting Power-Save Mode.
**Acceptance criteria:**
- [x] Material 3 design: dynamic theming, edge-to-edge window insets, 48x48dp minimum touch targets.
- [x] Integrated QR code scanner for 1-tap PC pairing.
- [x] Tactile live controls: stream status pill, torch button, camera flip, and prominent "Scan & Send to PC" action.
- [x] "Meeting Mode": screen dims / turns black while continuing background camera stream to prevent screen burn-in and save battery.
**Verification:**
- [x] Compose UI previews and unit tests verify navigation and meeting mode states.
**Dependencies:** Task 10, Task 12
**Files likely touched:**
- `android/app/src/main/java/com/linkcam/app/ui/MainActivity.kt`
- `android/app/src/main/java/com/linkcam/app/ui/StreamScreen.kt`
- `android/app/src/main/java/com/linkcam/app/ui/ScanScreen.kt`
- `android/app/src/main/java/com/linkcam/app/ui/theme/Theme.kt`
**Estimated scope:** M (4 files)

---

## Checkpoint 3: Android Client & End-to-End Link
- [x] Mobile app pairs with PC over Hotspot, Wi-Fi, or USB.
- [x] Video stream flows smoothly from phone to Windows companion.
- [x] Document scan delivers PDF to PC clipboard and folder.

---

## Phase 4: Verification, Polish & Performance Hardening

### Task 14: End-to-End Latency & Stream Performance Optimization
**Description:** Benchmark and optimize end-to-end video pipeline latency, packet jitter buffers, and frame synchronization.
**Acceptance criteria:**
- [x] End-to-end latency verified <80ms over 5GHz Wi-Fi / Hotspot and <50ms over USB.
- [x] Automatic frame rate / quality adaptation on network congestion.
**Verification:**
- [x] Latency benchmark test passes target thresholds.
**Dependencies:** Task 5, Task 10, Task 11
**Files likely touched:**
- `desktop/src-tauri/src/vcam/frame_sink.rs`
- `android/app/src/main/java/com/linkcam/app/transport/SocketTransport.rs`
**Estimated scope:** S (2 files)

---

### Task 15: Impeccable Design Audit & Polish
**Description:** Conduct comprehensive design audit against `craft-floor.md` and `android.md` standards to verify zero AI slop, perfect contrast, smooth transitions, and premium tactile feel across both platforms.
**Acceptance criteria:**
- [x] Impeccable quality checklist verified: contrast ratios, typography scale, custom scrollbars, cohesive vector icons, zero generic AI slop.
- [x] Windows 11 Fluent 2 materials and Android Material 3 tactile interactions confirmed.
**Verification:**
- [x] Visual and UI audit passes with zero defects.
**Dependencies:** Task 7, Task 8, Task 13
**Files likely touched:**
- `desktop/src/styles/fluent.css`
- `desktop/src/components/LiveStudio.tsx`
- `android/app/src/main/java/com/linkcam/app/ui/theme/Theme.kt`
**Estimated scope:** S (2 files)

---

## Phase 5: DroidCam Comparative Enhancements

### Task 16: Granular Hardware Camera Controls (Zoom, EV, Torch, Lens)
**Description:** Implement end-to-end remote camera hardware controls allowing Windows companion to adjust camera zoom ratio, exposure compensation index, torch state, and lens selection with zero latency.
**Acceptance criteria:**
- [x] Android `CameraStreamer.kt` implements Camera2 `CONTROL_ZOOM_RATIO` (with crop region fallback) and `CONTROL_AE_EXPOSURE_COMPENSATION` without recreating session.
- [x] Android `MainActivity.kt` processes incoming CameraControl RPCs for zoom, EV, torch, and lens switching.
- [x] Desktop Rust server (`server.rs`) wires `control_rx` to outbound TCP socket stream so commands are dispatched immediately to phone.
- [x] Desktop Tauri commands expose `send_camera_control` for zoom, EV, torch, lens, and meeting mode.
- [x] Desktop UI (`LiveStudio.tsx`, `CameraControls.tsx`) dispatches debounced control updates to Tauri backend.
**Verification:**
- [x] Code inspection and unit tests verify packet transmission and range boundary handling.
**Dependencies:** Task 1, Task 2, Task 7, Task 10
**Files likely touched:**
- `android/app/src/main/java/com/linkcam/app/camera/CameraStreamer.kt`
- `android/app/src/main/java/com/linkcam/app/ui/MainActivity.kt`
- `desktop/src-tauri/src/network/server.rs`
- `desktop/src-tauri/src/main.rs`
- `desktop/src/components/LiveStudio.tsx`
**Estimated scope:** M (5 files)

---

### Task 17: Low-Latency Phone Audio Streaming & Desktop VU Telemetry
**Description:** Stream synchronized 48kHz 16-bit PCM microphone audio from Android phone over existing TLS connection into Desktop companion, providing real-time audio input level (VU meter) in Desktop HUD.
**Acceptance criteria:**
- [x] Android `AudioStreamer.kt` and `MainActivity.kt` stream tagged `AudioFrame` (`0x02`) packets over active connection.
- [x] Desktop Rust server receives and buffers `AudioFrame` packets without stalling video frames.
- [x] Desktop companion calculates real-time RMS audio level and exposes via session status / telemetry for live UI VU meter.
- [x] Desktop HUD displays live audio status pill and volume indicator.
**Verification:**
- [x] Audio frame packet serialization and RMS level calculations pass tests.
**Dependencies:** Task 1, Task 2, Task 7, Task 10
**Files likely touched:**
- `desktop/src-tauri/src/network/server.rs`
- `desktop/src-tauri/src/main.rs`
- `desktop/src/components/StreamHUD.tsx`
- `desktop/src/components/LiveStudio.tsx`
**Estimated scope:** S (4 files)

---

### Task 18: Windows 11 Native Media Foundation User-Mode Virtual Camera
**Description:** Implement Windows 11 `MFCreateVirtualCamera` API registration wrapper in `mf_virtual_camera.rs`, providing user-mode virtual camera registration with zero kernel driver overhead.
**Acceptance criteria:**
- [x] `mf_virtual_camera.rs` dynamically checks for `MFCreateVirtualCamera` in `mfplat.dll` on Windows 11.
- [x] Invokes `MFCreateVirtualCamera` with `MFVirtualCameraType_SoftwareCameraSource` and user lifetime.
- [x] Falls back seamlessly to DirectShow COM filter registration on Windows 10 or when MF virtual camera is unavailable.
**Verification:**
- [x] Rust registration module compiles cleanly and verifies fallback paths.
**Dependencies:** Task 5
**Files likely touched:**
- `desktop/src-tauri/src/vcam/mf_virtual_camera.rs`
**Estimated scope:** S (1 file)

---

## Phase 6: Universal Virtual Camera Compatibility (Zoom, Meet, WhatsApp)

### Task 19: DirectShow Filter Registration & FilterData Auto-Generation
**Description:** Ensure DirectShow's `CreateClassEnumerator` immediately discovers "LinkCam Virtual Camera" in Zoom, OBS, and Google Meet without administrator privileges by writing the canonical binary `FilterData` blob to both `HKCU` and `HKLM`.
**Acceptance criteria:**
- [x] `LinkCamDShow.cpp` generates standard binary `FilterData` in `RegisterCategoryKeys()` for `HKEY_CURRENT_USER` and `HKEY_LOCAL_MACHINE`.
- [x] `mf_virtual_camera.rs` writes `FilterData` in `ensure_device_path()` whenever desktop companion starts.
- [x] Inno Setup installer script (`installer.iss`) updated to version 1.2.4 with admin privileges and clean COM filter registration.
**Verification:**
- [x] Registry verification shows `FilterData` binary value present under `CLSID_VideoInputDeviceCategory\Instance\{D37E5526-92F7-4E61-A76F-B590E8F88E29}`.
**Dependencies:** Task 18
**Files likely touched:**
- `desktop/vcam-dshow/LinkCamDShow.cpp`
- `desktop/src-tauri/src/vcam/mf_virtual_camera.rs`
- `desktop/installer.iss`
**Estimated scope:** S (3 files)

---

### Task 20: CleanPoint Keyframe Tagging & Media Foundation MFT Decoder Robustness
**Description:** Fix H.264 decoding in `LinkCamDShow.cpp` by tagging keyframes with `MFSampleExtension_CleanPoint = 1` and continuously draining output buffers so reference-frame buffering and B-frame delays never cause frame drops or black standby screens.
**Acceptance criteria:**
- [x] `MftH264Decoder::Decode()` accepts `bool isKeyframe` parameter.
- [x] Sets `MFSampleExtension_CleanPoint = 1` on keyframe `IMFSample` before calling `ProcessInput`.
- [x] Continuously drains output queue across consecutive `ThreadProc` iterations, preventing decoder pipeline starvation.
**Verification:**
- [x] Code inspection and compilation verify clean point attribute and drain logic.
**Dependencies:** Task 19
**Files likely touched:**
- `desktop/vcam-dshow/LinkCamDShow.cpp`
**Estimated scope:** S (1 file)

---

### Task 21: Windows 11 Virtual Camera Activation & Desktop Raw NV12 Frame Streaming Support
**Description:** Complete the Windows 11 user-mode virtual camera integration by calling `pVirtualCamera->Start(nullptr)` and keeping the COM interface alive, and enable direct uncompressed NV12 frame delivery in shared memory for zero-decode host performance.
**Acceptance criteria:**
- [x] `mf_virtual_camera.rs` calls `->Start(nullptr)` on `IMFVirtualCamera` and retains the COM pointer while registered.
- [x] Shared memory supports direct raw NV12 / RGB32 payloads (`format: 1`) allowing zero-decode DirectShow rendering.
- [x] DirectShow filter cleanly handles both H.264 (`format: 0`) and raw NV12 (`format: 1`).
**Verification:**
- [x] Rust and C++ virtual camera components compile cleanly.
**Dependencies:** Task 19, Task 20
**Files likely touched:**
- `desktop/src-tauri/src/vcam/mf_virtual_camera.rs`
- `desktop/src-tauri/src/vcam/frame_sink.rs`
- `desktop/vcam-dshow/LinkCamDShow.cpp`
**Estimated scope:** M (3 files)


