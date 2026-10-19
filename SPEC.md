# Spec: LinkCam (Continuity Camera Companion for Android & Windows)

## Objective
LinkCam provides an Apple Continuity Camera-tier companion experience for Android and Windows. It allows users to use their modern Android phone's rear or front camera as a low-latency, high-resolution physical webcam in Windows (Zoom, Teams, Google Meet, OBS) over USB or Wi-Fi/Direct Hotspot, alongside instant one-tap document and photo capture with auto-crop delivered straight to the Windows clipboard and desktop tray.

## Tech Stack
- **Windows Companion Core**: Tauri (v2.x) with Rust backend (`tokio`, `tungstenite` / local TCP socket, `windows` crate / DirectShow & MediaFoundation Virtual Camera integration).
- **Windows Companion Frontend**: Modern TypeScript, React, Vite, Tailwind CSS (or scoped CSS modules matching Fluent Design 2 / Windows 11 Mica & Acrylic aesthetics), Lucide SVG icons.
- **Android Client**: Native Kotlin, Jetpack Compose, CameraX (v1.3+), ML Kit Document Scanner API / Vision Edge Detection, Android Coroutines + Ktor/OkHttp WebSocket, Android 10+ (API 29+, targeting ARM64-v8a).
- **Protocol & Network Transport**: High-performance local binary protocol over TCP/WebSocket, H.264 / MJPEG packet streaming with timestamps, mDNS discovery (`_linkcam._tcp`), QR code session encryption (AES-256-GCM).

## Commands
```bash
# Windows Desktop App (Tauri)
Dev: npm run tauri dev
Build: npm run tauri build
Test: npm test
Lint: npm run lint

# Android App (Gradle)
Build: ./gradlew assembleDebug
Test: ./gradlew test
Install on device: ./gradlew installDebug
```

## Project Structure
```
d:/AG/LinkCam/
├── CAPABILITY_MAP.md
├── PRODUCT.md
├── SPEC.md
├── .impeccable/
│   └── config.json
├── protocol/                     # Shared protocol definitions & packet specifications
│   ├── schemas/                  # JSON/binary protocol schemas (control, status, frames)
│   └── docs/                     # Wire protocol documentation
├── desktop/                      # Windows Companion App (Tauri + React + Rust)
│   ├── src-tauri/                # Rust backend: network server, virtual cam bridge, tray
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── network/          # TCP/WebSocket server, QR pairing, hotspot listener
│   │       ├── vcam/             # Windows Virtual Camera / DirectShow pipe
│   │       └── capture/          # Instant capture receiver, clipboard & file manager
│   ├── src/                      # Windows 11 Fluent UI frontend
│   │   ├── components/           # Camera preview, stream stats, device drawer, quick tray
│   │   ├── styles/               # Fluent Mica, Acrylic, typography, custom scrollbars
│   │   └── App.tsx
│   ├── package.json
│   └── vite.config.ts
└── android/                      # Native Android Client
    ├── app/
    │   ├── build.gradle.kts
    │   └── src/main/
    │       ├── AndroidManifest.xml
    │       └── java/com/linkcam/app/
    │           ├── camera/       # CameraX streamer, resolution & sensor controller
    │           ├── scanner/      # Document auto-crop & PDF generator
    │           ├── transport/    # USB / Wi-Fi / Hotspot socket client
    │           └── ui/           # Jetpack Compose Material 3 screens & tactile controls
    └── build.gradle.kts
```

## Code Style & Impeccable Design Standards
- **Anti-Slop Guarantee**:
  - No generic repetitive icon-cards with boilerplate text.
  - No fake gradient text, neon halos, or unmotivated animations.
  - No unicode emoji icons — use crisp, dedicated SVGs with consistent stroke weights.
  - Windows 11 typography: `Segoe UI Variable`, `Segoe UI`, or clean system sans with tabular numerals for FPS, bitrate, and latency stats.
  - Depth: Windows 11 Mica & Acrylic materials, subtle borders (`rgba(255, 255, 255, 0.08)` in dark mode), natural soft box shadows.
  - Android UI: Strictly follows Material 3 (Material You dynamic theming, edge-to-edge insets, 48x48dp minimum touch targets, proper predictive back support).
- **TypeScript**: Strict null checks, explicit types for IPC and network events, no `any`.
- **Rust**: Idiomatic Rust with clean error handling (`thiserror`, `anyhow`), zero panics in background loops, bounded packet channels.
- **Kotlin**: Jetpack Compose idiom, Kotlin Coroutines & Flows, single-source-of-truth ViewModels, clean separation between camera hardware and UI layers.

## Testing Strategy
- **Protocol**: Unit tests for packet framing, serialization, handshake verification, and chunk reassembly.
- **Desktop Companion**: Component unit tests for preview UI and connection state machine; end-to-end simulation of mock video stream and capture reception.
- **Android Client**: Unit tests for capture conversion, state machines, and connection fallbacks (Hotspot vs. Wi-Fi vs. USB).
- **End-to-End**: Verification of round-trip frame latency (<60ms over USB, <90ms over 5GHz Wi-Fi/Hotspot) and instant capture clipboard injection.

## Boundaries
- **Always**:
  - Validate packet payloads and authenticate pairing tokens before accepting video streams or files.
  - Clean up camera hardware resources and background services immediately upon disconnect.
  - Ensure zero external network/cloud dependencies: LinkCam must function entirely offline.
  - Provide immediate visual feedback for connection states (Connected, Streaming, Hotspot Ready, Reconnecting).
- **Ask First**:
  - Changing the video compression codec (H.264 vs MJPEG vs VP8/AV1).
  - Altering the Windows Virtual Camera driver registration mechanism.
- **Never**:
  - Send user video, audio, or scanned documents to external cloud servers or analytics.
  - Overwrite user files in the LinkCam capture folder without unique timestamped naming.
  - Force nag screens, paywalls, or artificial resolution watermarks.

## Success Criteria
1. **Webcam Streaming**: Android camera streams 1080p video at 30/60 fps to Windows companion with sub-100ms latency.
2. **Virtual Camera Registration**: LinkCam appears as a selectable camera device in Windows apps (Zoom, Teams, OBS, Google Meet).
3. **Instant Capture**: User taps "Capture Document" on phone → phone detects edges, auto-crops, converts to PDF/JPEG → sends to PC → PC displays interactive quick-toast and places file onto Windows clipboard and designated directory within 1.5 seconds.
4. **Zero-Router Direct Hotspot Pairing**: PC connects directly to Android phone's hotspot without requiring any local router or internet connection, automatically establishes handshake, and begins streaming.
5. **Craft Quality**: Interface passes Impeccable audit with high visual fidelity, fluid feedback, and zero generic AI slop.
