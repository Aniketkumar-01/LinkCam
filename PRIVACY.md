# Privacy Policy for LinkCam

**Last Updated:** October 2026  
**Product:** LinkCam (Desktop Companion & Android App)  
**License:** Open Source (MIT)

---

### 1. Overview
LinkCam is an open-source, peer-to-peer webcam companion application designed to turn your Android phone into an ultra-low-latency, high-fidelity webcam and document scanner for Windows. 

**LinkCam does NOT collect, store, sell, or transmit your personal data to any external server or cloud provider.** All operations run exclusively on your local hardware and private network.

---

### 2. Device Permissions & Usage

#### A. Camera (`android.permission.CAMERA`)
- **Purpose:** Used strictly to capture real-time video frames from your phone's camera sensors and stream them directly to your companion PC.
- **Scope:** Video frames are encoded on-device via hardware MediaCodec (H.264) and sent over your local network (Wi-Fi or USB ADB reverse) directly to LinkCam Desktop. Video data is never uploaded to the internet or recorded remotely.

#### B. Microphone (`android.permission.RECORD_AUDIO`)
- **Purpose:** Used to capture 48 kHz stereo audio to synchronize microphone input with your video feed.
- **Scope:** Audio packets are streamed point-to-point to your local PC for virtual microphone bridge injection. Audio is never stored or transmitted outside your private network.

#### C. Local Network & Wi-Fi (`android.permission.INTERNET`, `ACCESS_NETWORK_STATE`, `ACCESS_WIFI_STATE`)
- **Purpose:** Enables point-to-point communication between your Android device and PC.
- **Scope:** Traffic flows over local private subnets (e.g. `192.168.x.x`, `10.x.x.x`, or `127.0.0.1` over USB cable). Zero internet bandwidth is consumed when using USB cable or Zero-Router Hotspot mode.

---

### 3. File Storage & Instant Captures
When you trigger an instant photo or document scan in LinkCam:
- The captured document/photo is processed on-device (via offline edge cropping / ML Kit).
- The file is delivered directly to your personal PC and saved locally in your Pictures directory:
  ```
  %USERPROFILE%\Pictures\LinkCam\
  ```
- The file path is copied to your local Windows clipboard for immediate `Ctrl+V` pasting.
- No documents or images are ever uploaded to any cloud service.

---

### 4. Children’s Privacy
LinkCam does not collect personally identifiable information from any user, including children under 13/16/18.

---

### 5. Third-Party Services
LinkCam does not integrate third-party advertising SDKs, cloud analytics trackers, or user profiling services.

---

### 6. Contact & Source Code
LinkCam is open-source software distributed under the MIT license.  
- **Source Code & Issue Tracker:** [https://github.com/Aniketkumar-01/LinkCam](https://github.com/Aniketkumar-01/LinkCam)
