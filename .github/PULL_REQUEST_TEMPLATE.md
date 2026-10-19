## Summary of Changes
<!-- Provide a brief description of the problem solved, feature added, or improvement made. -->

## Related Issues
<!-- Link relevant issue numbers, e.g., Closes #12, Fixes #34 -->

## Type of Change
- [ ] 🐛 Bug fix (non-breaking change fixing an issue)
- [ ] ✨ New feature (non-breaking change adding functionality)
- [ ] 🔒 Security hardening / vulnerability patch
- [ ] ⚡ Performance optimization
- [ ] 📝 Documentation update
- [ ] 🎨 UI/UX styling improvement

## Subsystem Impacted
- [ ] Windows Desktop Companion (Tauri / React / Rust)
- [ ] Android Mobile Client (Kotlin / Compose / CameraX)
- [ ] Binary Protocol (`protocol/`)
- [ ] Windows Media Foundation Virtual Camera Driver (`vcam/`)
- [ ] CI/CD & Build Workflows

## Pre-Merge Checklist
- [ ] Code compiles cleanly locally (`npm run build` in `desktop/`, `./gradlew assembleDebug` in `android/`, `cargo check` in `desktop/src-tauri/`)
- [ ] No unused locals or TypeScript type errors (`tsc --noEmit` passes)
- [ ] Tested on actual hardware (Windows 11 / Android device)
- [ ] Zero AI Slop: adhering to Fluent 2 / Material 3 design tokens without unnecessary dependencies
- [ ] Added or updated relevant documentation or changelog entries
