# Implementation Plan: LinkCam (Continuity Companion for Android & Windows)

## Overview
LinkCam is an Apple Continuity Camera companion for Android and Windows. It provides zero-configuration, ultra-low-latency phone-to-PC webcam streaming (via USB, local Wi-Fi, or zero-router Direct Mobile Hotspot) and one-tap instant capture/document scanning with automatic edge-detection and PDF conversion delivered directly into the Windows clipboard and system tray.

## Architecture Decisions
1. **Tauri v2 + Rust Desktop Engine**:
   - Extremely low memory footprint (<40MB RAM vs 300MB+ for Electron), critical when running alongside resource-heavy meeting tools (Teams, Zoom, Google Meet) and games/streaming software (OBS).
   - Rust handles high-throughput UDP/TCP packet parsing, multi-threaded video frame decoding, Windows Media Foundation virtual camera streaming, and native Windows clipboard/tray integration.
2. **Windows 11 Fluent 2 Interface (Frontend)**:
   - Designed to the highest Impeccable standards: authentic Mica and Acrylic materials, Segoe UI Variable typography, crisp vector SVG iconography (zero emoji slop, zero repetitive icon cards), and tactile micro-interactions.
   - Live stream telemetry HUD: real-time FPS, bitrate, end-to-end latency, phone battery level, thermal state, and connection mode.
3. **Native Android Client (Kotlin + Jetpack Compose + CameraX)**:
   - Built specifically for modern ARM64-v8a Android 10+ devices.
   - CameraX `ImageAnalysis` + hardware `MediaCodec` H.264 / low-latency MJPEG encoder for smooth 1080p60 streaming.
   - Foreground service with power-saving black screen mode ("Meeting Mode") to prevent OLED burn-in and thermal throttling during long calls.
   - ML Kit Document Scanner / Vision edge detection for auto-perspective cropping and PDF generation.
4. **Zero-Router Direct Hotspot & Local Pairing**:
   - Works anywhere: Android can create a Mobile Hotspot, Windows PC connects to it, and LinkCam streams directly over local Wi-Fi without needing a 3rd device, external router, or internet access.
   - QR code pairing transfers ephemeral session keys for local AES-256-GCM authentication.

## Task List

### Phase 1: Protocol & Network Foundation
- [ ] Task 1: Wire Protocol & Serialization Specification
- [ ] Task 2: Network Server Engine (TCP/WebSocket & Pairing Handshake in Rust)
- [ ] Task 3: Pairing Handshake & Hotspot Discovery Protocol

### Checkpoint: Protocol Foundation
- [ ] Protocol packets serialize and deserialize cleanly with unit tests
- [ ] Handshake and token verification pass

### Phase 2: Windows Companion App & Virtual Camera Bridge
- [ ] Task 4: Tauri Project Scaffold & Native Rust Modules
- [ ] Task 5: Windows Virtual Camera Pipeline (Media Foundation / DirectShow bridge)
- [ ] Task 6: Instant Capture Receiver, Clipboard Injector & Notification Engine
- [ ] Task 7: Impeccable Windows 11 Fluent UI (Live Studio, Stream HUD, Camera Controls)
- [ ] Task 8: Pairing Wizard & Interactive Quick-Capture Tray

### Checkpoint: Desktop Companion
- [ ] Desktop app builds and launches with sleek Windows 11 Fluent 2 interface
- [ ] Virtual camera feed sink renders video smoothly
- [ ] Instant capture banner, tray, and clipboard injection verified

### Phase 3: Android Native Client
- [ ] Task 9: Android Project Scaffold (Kotlin, Jetpack Compose, CameraX)
- [ ] Task 10: CameraX Video Streamer & MediaCodec Hardware Pipeline
- [ ] Task 11: Transport Layer (Hotspot, Wi-Fi, USB Socket Client)
- [ ] Task 12: Instant Capture & Document Auto-Crop / PDF Engine
- [ ] Task 13: Impeccable Material 3 Android UI & Meeting Power-Save Mode

### Checkpoint: Android Client & End-to-End Link
- [ ] Android app captures camera frames, encodes, and transmits to PC
- [ ] Auto-crop document scan produces clean PDF and delivers to Windows clipboard in <1.5s
- [ ] Hotspot pairing works with zero router dependencies

### Phase 4: Verification, Polish & Performance Hardening
- [x] Task 14: End-to-End Latency & Stream Performance Optimization
- [x] Task 15: Impeccable Design Audit & Polish (Zero AI Slop verification)

### Phase 5: DroidCam Comparative Enhancements
- [x] Task 16: Granular Hardware Camera Controls (Zoom, EV, Torch, Lens)
- [x] Task 17: Low-Latency Phone Audio Streaming & Desktop VU Telemetry
- [x] Task 18: Windows 11 Native Media Foundation User-Mode Virtual Camera

## Risks and Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Windows Virtual Camera Driver permissions / signing | High | Utilize Windows Media Foundation `MFCreateVirtualCamera` API (Windows 11 / Windows 10 21H2+) which runs in user-mode without kernel driver signing, with DirectShow filter fallback. |
| Wi-Fi interference / latency spikes | Med | Support automatic bitrate / resolution throttling, buffer drop on keyframe, and seamless fallback to high-speed USB socket. |
| Phone heating during 1080p60 streaming | Med | Implement Android "OLED Meeting Mode" (dims or blanks screen while running camera analysis in foreground service) and hardware `MediaCodec` encoding. |
| Network isolation on guest/enterprise Wi-Fi | Low | Direct Mobile Hotspot mode bypasses corporate Wi-Fi isolation entirely; direct USB connection provides zero-network fallback. |
