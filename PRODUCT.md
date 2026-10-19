# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack
Tauri (Rust core & virtual camera pipeline + modern TypeScript/Web frontend) for Windows 11/10 companion; Native Android (Kotlin + Jetpack Compose + CameraX + ML Kit Document Scanner) for mobile client.

## Users
Remote professionals, hybrid workers, software engineers, streamers, educators, and creators using Windows PCs who want Apple Continuity Camera-level video fidelity and instant document capture from their Android phones without subscriptions, paywalls, or cloud dependencies.

## Product Purpose
Bring an Apple Continuity Camera-grade companion experience to Android and Windows: zero-configuration phone-to-webcam video streaming (via USB and Wi-Fi / direct Hotspot), combined with instant mobile photo and auto-cropped document scanning directly into the Windows workspace.

## Positioning
Unlike legacy webcam apps (DroidCam, Iriun, Camo with subscriptions and clunky UIs), LinkCam feels like a first-party Windows + Android ecosystem feature. It provides native Windows Virtual Camera (DirectShow / Media Foundation) driver integration, high-resolution low-latency streaming, zero-router direct Hotspot pairing, and one-tap document edge detection delivered straight to the Windows clipboard and desktop tray.

## Operating Context
- Workflows: Video conferencing (Microsoft Teams, Zoom, Google Meet, Slack, Discord), streaming and recording (OBS Studio, Camtasia), and office productivity (scanning receipts, notes, IDs, contracts directly into emails, chats, or documents).
- Physical setup: Phone docked on laptop mount or desk tripod, connected via USB cable or wireless Wi-Fi / phone hotspot.
- Environment: Windows 11 / Windows 10 (x64) and Android 10+ (ARM64-v8a modern devices).

## Capabilities and Constraints
- **Camera Link**: Real-time video streaming with configurable resolution (720p, 1080p, 4K) and framerate (30fps, 60fps), camera selection (main sensor, ultra-wide, telephoto, selfie), exposure/white balance adjustments, zoom slider, and torch toggle controlled from PC.
- **Windows Virtual Camera**: System-wide webcam driver recognized natively by any Windows app without requiring screen capture or third-party virtual cameras.
- **Instant Capture**: Document scan with automatic edge detection, perspective correction, filter enhancement (Clean Document / Color / Grayscale), and multi-page PDF generation, plus full-resolution photo capture.
- **Desktop Delivery**: Floating notification banner and interactive system tray quick-actions (Copy to Clipboard, Save as PDF/PNG to LinkCam folder, Drag & Drop directly into active applications).
- **Pairing & Connectivity**: QR code fast handshake, zero-router direct Hotspot pairing, local network mDNS discovery, and high-speed USB socket communication.
- **Privacy & Security**: End-to-end encrypted local socket transport (TLS/AES-GCM), 100% offline-capable, zero analytics or cloud video streaming.

## Brand Commitments
- Name: LinkCam
- Design Aesthetic: Impeccable craftsmanship, anti-slop, clean Windows 11 Fluent Design (Mica / Acrylic depth, Segoe UI Variable, subtle elevation, responsive side navigation) and Android Material 3 (Material You dynamic theming, edge-to-edge insets, fluid tactile gestures).

## Evidence on Hand
- Product brief and architecture decisions captured from design interview (Adaptive platform, Tauri + Rust + Web on Windows, Kotlin + Compose on Android, DirectShow/MediaFoundation virtual camera, interactive tray/banner delivery).

## Product Principles
1. **Zero-Latency Feel**: Video frame delivery and capture triggers must feel instant, tactile, and rock-solid without dropped frames or audio-video drift.
2. **Zero-Router Freedom**: Work anywhere—office enterprise Wi-Fi, home network, coffee shop direct hotspot, or USB tethering—with zero external router dependencies.
3. **Native System Respect**: Live harmoniously inside Windows 11 (Fluent, Mica, system tray, toast notifications) and Android (Material 3, edge-to-edge, CameraX) rather than forcing an alien generic web container.
4. **Frictionless Continuity**: No sign-ins, no subscriptions, no watermarks, no intrusive ads. One QR scan to pair, one tap to stream or capture.
