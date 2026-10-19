# LinkCam

> **Continuity Camera Companion for Android & Windows**  
> High-fidelity, low-latency phone-to-PC webcam streaming and instant one-tap document auto-crop & photo delivery straight to the Windows clipboard.

---

## Architecture Overview

LinkCam consists of three core components:

1. **`protocol/`**: High-performance binary wire protocol (`LNKC` 24-byte header), control RPC schemas, and telemetry formats shared between desktop and mobile.
2. **`desktop/`**: Windows companion application built with **Tauri v2 + Rust** and a **Windows 11 Fluent 2** frontend (Mica/Acrylic materials, telemetry HUD, camera control dock, quick-capture tray, and Windows Virtual Camera bridge).
3. **`android/`**: Native Android app written in **Kotlin & Jetpack Compose** targeting modern **ARM64-v8a** devices (Android 10+ / API 29+) using **CameraX**, **ML Kit Document Scanner**, and an **OLED Meeting Power-Save Mode**.

---

## Running the Applications

### 1. Windows Companion App (`desktop/`)

#### Option A: Quick Frontend Preview (Vite Dev Server)
To preview and interact with the Windows 11 Fluent 2 companion dashboard in your browser:

```bash
cd desktop
npm install
npm run dev
```
Open [http://localhost:3000](http://localhost:3000) to view:
- **Live Studio**: Camera viewfinder, telemetry HUD (FPS, bitrate, latency, battery), and tactile controls.
- **Pair Devices**: Dynamic QR code generator and Zero-Router Hotspot guide.
- **Instant Captures**: Auto-crop document gallery and clipboard manager.

#### Option B: Full Desktop Companion (Tauri Native)
To run with native Windows Virtual Camera registration, network listener, and system tray:

```bash
cd desktop
npm install
npm run tauri dev
```

---

### 2. Android Mobile App (`android/`)

#### Open in Android Studio
1. Open **Android Studio** (Hedgehog, Iguana, Jellyfish, or newer).
2. Select **Open an Existing Project** and navigate to `d:\AG\LinkCam\android`.
3. Allow Gradle to sync dependencies.
4. Connect an Android phone (Android 10+, ARM64-v8a) with USB Debugging enabled.
5. Click **Run** (`Shift + F10`) or install via command line:

```bash
cd android
./gradlew installDebug
```

---

## Connection Modes

1. **Direct Hotspot (Zero Router Needed)**:
   - Turn on Mobile Hotspot on your phone.
   - Connect your Windows PC's Wi-Fi to your phone's hotspot.
   - Open LinkCam on phone and scan the QR code on the desktop app.
   - Video frames stream directly over the local subnet (`192.168.43.1`) without using internet data!
2. **Local Wi-Fi**:
   - Both devices connected to the same 5GHz Wi-Fi network.
   - Scan desktop QR code for instant pairing.
3. **USB Cable (Ultra Low Latency)**:
   - Connect phone to PC with a USB-C cable.
   - Enables sub-40ms latency and keeps the phone charged during long calls.

---

## Features
- **Zero AI Slop**: Designed to Impeccable craftsmanship standards. No generic repetitive cards, no gradient text, no emoji icons, and authentic Fluent 2 / Material 3 type scales.
- **Windows Virtual Camera**: Appears in Zoom, Microsoft Teams, Google Meet, and OBS as `"LinkCam Virtual Camera"`.
- **Instant Capture**: Auto-crops documents, straightens perspective, compiles to clean PDF, and delivers straight into the Windows clipboard ready for `Ctrl+V`.
- **OLED Meeting Mode**: Dims/blanks the phone display during long meetings to prevent thermal throttling and save battery while streaming in the background.
