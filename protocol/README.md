# LinkCam Wire Protocol Specification

Version: 1.0.0  
Transport: TCP Socket / WebSocket (Local Network, Direct Hotspot, USB Loopback)  
Byte Order: Network Big-Endian (`BE`)

## Binary Packet Layout

Every transmission packet begins with a fixed 24-byte header followed by a variable-length payload:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Magic "LNKC" (4 bytes)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    Version    |  Packet Type  |     Flags     |   Reserved    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Sequence Number (32 bits)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                    Timestamp / PTS (64 bits)                  +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Payload Length (32 bits)                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Bytes...                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Packet Types
| Value | Identifier | Description |
|---|---|---|
| `0x01` | `VideoFrame` | Encoded video stream frame (H.264 NAL unit or MJPEG frame) |
| `0x02` | `AudioFrame` | Synchronized AAC or Opus audio frame |
| `0x03` | `InstantCaptureChunk` | Document PDF or JPEG photo file chunk |
| `0x10` | `PairingAuth` | Initial cryptographic token handshake |
| `0x11` | `PairingAuthResponse`| Token verification and session approval |
| `0x20` | `CameraControl` | PC to Phone remote camera control RPC |
| `0x21` | `CameraControlAck` | Confirmation of camera setting applied |
| `0x30` | `TelemetryHeartbeat` | Phone-to-PC stats (FPS, bitrate, latency, battery, thermal) |

### Header Flags
- `0x01`: Keyframe / IDR frame (essential for instant recovery if packets drop).
- `0x02`: Compressed chunk.
- `0x04`: Encrypted with session AES-256-GCM.
