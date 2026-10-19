# Design System: LinkCam

<!-- impeccable:design-schema 1 -->

## Design Philosophy: Anti-Slop & Ecosystem Native
LinkCam is designed as an ecosystem-grade companion experience that feels like a native first-party feature built by Microsoft and Google working together—not an AI-generated template or a third-party ad-ridden utility.

### Prohibited Patterns ("The Slop Test")
- **No repetitive identical cards**: Do not use grids of identical cards with `[icon] + [bold title] + [generic 2-line description]`.
- **No gradient text**: Headings earn emphasis through weight, scale, and spatial hierarchy, never rainbow/neon text fills.
- **No fake glows or halos**: Zero-blur neon halos around containers are forbidden. Elevation is conveyed through authentic Windows 11 Mica/Acrylic translucency and subtle layered depth.
- **No kicker/eyebrow labels**: Do not put tiny uppercase tags (e.g., "FEATURE OVERVIEW") above main titles. Let headings speak for themselves.
- **No emoji or unicode icons**: Every icon is a drawn, crisp SVG with unified 1.5px/2px stroke weight.
- **No fake monospace**: Monospace is reserved exclusively for real live data (bitrate, frame times, timestamps, memory stats, IP addresses).

---

## Windows Desktop Surface (Windows 11 Fluent 2)

### Materials & Depth
- **Window Base**: Deep Slate Acrylic (`#0f1115` base with background blur `backdrop-filter: blur(28px) saturate(125%)`).
- **Surface Elevation 1 (Sidebar & Navigation)**: Semi-transparent tinted background (`rgba(22, 27, 34, 0.75)` with subtle 1px border `rgba(255, 255, 255, 0.08)`).
- **Surface Elevation 2 (Controls & Cards)**: Layered acrylic plates (`rgba(30, 36, 46, 0.65)` with hover elevation `rgba(40, 48, 62, 0.8)`).
- **Accent**: Windows 11 Fluent Cobalt (`#0078d4`, focus rings `#2b88d8`, text accent `#60a5fa`).
- **Borders**: Hairline 1px border (`rgba(255, 255, 255, 0.08)`) framing interactive surfaces.

### Typography (Fluent Type Hierarchy)
- **Primary Typeface**: `Segoe UI Variable`, `Segoe UI`, system-ui, sans-serif.
- **Display / Title**: Segoe UI Variable Display, Semibold (20px - 24px), tracking `-0.01em`.
- **Body**: Segoe UI Variable Text, Regular (13px - 14px), line-height `20px`.
- **Telemetry & Numbers**: `Segoe UI Variable Text` with `font-variant-numeric: tabular-nums` or `Cascadia Code` (11px - 12px) for live stream stats.

### Core Screens & Layouts
1. **Live Studio (Primary)**:
   - Full-bleed 16:9 / 4:3 responsive video viewfinder with low-latency hardware canvas rendering.
   - Floating HUD overlay (top right): Pill-shaped glass badge showing Connection Type (`⚡ USB` / `📶 5GHz Wi-Fi` / `📱 Direct Hotspot`), `1080p60`, `4.8 Mbps`, `42ms Latency`.
   - Floating Status Pill (top left): Phone model name, battery level (`🔋 84%`), thermal state (`Normal`).
   - Tactile Control Bar (docked bottom): Lens selector (Rear 1x / Ultra-wide 0.6x / Front), Torch toggle, Zoom slider, Exposure lock, Flip horizontal, and Quick Capture trigger.
2. **Instant Capture Quick-Tray**:
   - Floating slide-over drawer / toast banner triggered when phone delivers a photo or auto-cropped document.
   - Actions: `Copy to Clipboard` (default primary), `Open PDF`, `Show in Explorer`, `Drag into Active App`.
3. **Pairing & Zero-Router Hotspot Wizard**:
   - Clean dynamic QR code rendered with SVG paths.
   - Interactive tab for "Direct Hotspot (No Router)" with step-by-step auto-detection.

---

## Android Mobile Surface (Material Design 3)

### Principles
- **Edge-to-Edge**: Full screen utilization with proper window insets (`statusBars`, `navigationBars`, display cutouts).
- **Material You Dynamic Theming**: Primary and secondary tonal palettes derived from system dynamic colors, with a dedicated Dark Theme optimized for OLED displays.
- **Touch Targets**: Minimum 48×48 dp for every control with generous touch padding.
- **Predictive Back**: Clean transition animations respecting Android 14/15 predictive back gestures.

### Signature Mobile Features
1. **OLED Meeting Mode (Power-Save)**:
   - During long meetings (e.g. 45-minute video call), keeping the phone screen on causes heat buildup and battery drain.
   - One-tap "Meeting Mode" dims or turns off screen pixels while the CameraX background service continues streaming at 60fps with camera sensors active.
   - A single tap anywhere on screen instantly wakes the UI.
2. **Ergonomic Capture FAB**:
   - Extended Floating Action Button at the bottom center: "Scan Document" with one-touch auto-crop and send.
3. **Connection Mode Pill**:
   - Prominently displays current transmission channel (`USB Connected` / `Direct Hotspot Active` / `Local Wi-Fi`) with signal strength.
