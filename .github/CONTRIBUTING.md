# Contributing to LinkCam

Thank you for your interest in contributing to **LinkCam**! 

LinkCam is an open-source Apple Continuity Camera alternative bridging modern Android hardware (CameraX, ML Kit) with Windows 11 (Windows Media Foundation Virtual Camera, Fluent 2).

---

## 🛠️ Development Setup

### Prerequisites
- **Node.js**: v20+ with npm
- **Rust**: Stable toolchain (1.75+)
- **Android Studio / Android SDK**: API 29+ (compileSdk 35) with JDK 17
- **Windows 11**: For testing the Windows Media Foundation Virtual Camera bridge

### Subsystem Repositories

#### 1. Windows Companion (`desktop/`)
```bash
cd desktop
npm install
npm run dev # Launches Vite frontend at http://localhost:3000

# To run the Tauri desktop engine:
cargo run --manifest-path src-tauri/Cargo.toml
```

#### 2. Android App (`android/`)
```bash
cd android
./gradlew assembleDebug
# Installs to connected Android device via ADB:
./gradlew installDebug
```

#### 3. Binary Wire Protocol (`protocol/`)
```bash
cd protocol
cargo test
```

---

## 🎨 Craftsmanship & Quality Standards

- **Zero AI Slop**: Maintain authentic Fluent 2 / Material 3 type hierarchies and color tokens. Do not introduce generic bloated utility libraries when standard components suffice.
- **Strict Typing**: TypeScript `tsc --noEmit` must pass with zero errors and no unused imports.
- **Security First**: All external inputs, network packets, and filenames must be strictly bounded and sanitized before memory allocation or filesystem writes.

---

## 📜 Pull Request Process

1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Verify local builds and tests:
   ```bash
   cd desktop && npm run build
   cd ../android && ./gradlew assembleDebug
   cd ../protocol && cargo test
   ```
3. Commit with [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(...)`: new feature
   - `fix(...)`: bug fix
   - `perf(...)`: performance improvement
   - `docs(...)`: documentation changes
4. Submit a Pull Request targeting `main`.
