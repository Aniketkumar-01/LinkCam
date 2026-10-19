# Capability Map: LinkCam

| Module id | Responsibility | Depends on |
|---|---|---|
| protocol | Shared network wire protocol, pairing handshake (QR / crypto tokens), RPC commands, and frame streaming format | — |
| virtual-cam | Windows Virtual Camera integration (DirectShow / Windows Media Foundation Virtual Camera API) and video frame sink | protocol |
| desktop | Windows desktop companion app (Tauri + Rust core + Windows 11 Fluent UI): pairing dashboard, low-latency receiver, camera controls, instant capture banner/tray | protocol, virtual-cam |
| android | Modern Android native companion app (Kotlin, Jetpack Compose, CameraX, ML Kit Scanner): background stream, USB/Wi-Fi/Hotspot direct engine, instant capture | protocol |

Build order: protocol → virtual-cam → desktop → android
