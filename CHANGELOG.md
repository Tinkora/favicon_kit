# Changelog

## [0.1.0] - 2026-08-06

### Added

- `favicon_kit_core`: favicon generation engine with image resize, ICO encoding,
  manifest generation, and ZIP export.
- `favicon_kit_web`: Rust/WASM bridge and static editor with live previews.
- Browser, Apple Touch, PWA, and Windows tile output assets.
- English-first product documentation and a complete Simplified Chinese README.

### Features

- Multi-resolution ICO generation (16, 32, 48, 64, 128, 256)
- Individual PNG export for 16, 32, 48, 64, 128, 180, 192, and 512 pixels
- 180 by 180 Apple Touch assets and a 150 by 150 Windows tile asset
- `site.webmanifest`, `browserconfig.xml`, and HTML `<head>` snippet generation
- Configurable padding background, padding percentage, application names, and
  theme color
- Drag-and-drop image selection, live previews, and ZIP download
