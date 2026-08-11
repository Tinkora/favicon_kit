# Repository Guide for AI Agents

## Project Overview

Favicon Kit v0.1.0 is a browser-local favicon asset generator. The static editor
passes browser-decoded image data to Rust/WASM and exports a fixed package for
web applications and PWAs. It does not include an application backend.

## Architecture

```text
favicon_kit/
├── crates/
│   ├── favicon_kit_core/         # Image resize, ICO encode, manifest generation, WASM bindings
│   └── favicon_kit_web/          # WASM cdylib + HTML editor
├── docs/                          # English-first product specifications
└── skills/                        # Non-runnable integration drafts and guidance
```

## Key Files for AI Context

| File | Purpose |
|------|---------|
| `crates/favicon_kit_core/src/generate.rs` | Image resize, ICO encode, manifest/HTML snippet generation |
| `crates/favicon_kit_core/src/error.rs` | Stable error type with machine-readable codes |
| `crates/favicon_kit_core/src/wasm.rs` | WASM bindings (3 JS exports) |
| `crates/favicon_kit_core/src/lib.rs` | Module declarations and re-exports |
| `crates/favicon_kit_web/src/lib.rs` | cdylib entry point, re-exports WASM functions |
| `crates/favicon_kit_web/static/index.html` | Full-featured favicon generator UI |
| `skills/favicon_kit.md` | Guidance for describing the browser application |
| `skills/mcp-tools.json` | Non-runnable draft integration schema |
| `docs/product_spec.md` | English product specification |
| `docs/product_spec.zh-CN.md` | Simplified Chinese product specification |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets -- -D warnings

# WASM compilation check
cargo check -p favicon_kit_web --target wasm32-unknown-unknown

# Build Web WASM for deployment
wasm-pack build --target web crates/favicon_kit_web

# Copy the generated package to the static editor before local serving or Pages deployment
mkdir -p crates/favicon_kit_web/static/pkg
cp crates/favicon_kit_web/pkg/* crates/favicon_kit_web/static/pkg/

# Run the static editor locally
python3 -m http.server --directory crates/favicon_kit_web/static 8080
```

## Design Principles

1. **Browser-first**: Image processing and package generation run in Rust/WASM after browser decoding
2. **No application upload path**: The editor has no application endpoint for source images or generated files
3. **ICO as PNG container**: Modern ICO format uses embedded PNG data for each resolution
4. **Padding with background**: When source image aspect ratio differs from 1:1, padding fills with configurable color

## Favicon Sizes

Standard set: 16, 32, 48, 64, 128, 180, 192, 512

- ICO contains: 16, 32, 48, 64, 128, 256
- Individual PNGs: 16, 32, 48, 64, 128, 180, 192, and 512
- Apple Touch: `apple-touch-icon.png` and `favicon-180x180.png`, both 180x180
- PWA: 192x192, 512x512
- Windows tile: `mstile-150x150.png`, 150x150

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
|------|---------|
| `IMAGE_TOO_SMALL` | Source image below minimum dimension |
| `IMAGE_TOO_LARGE` | Source image exceeds maximum dimension |
| `RESIZE_ERROR` | Image resize operation failed |
| `ICO_ERROR` | ICO encoding failed |
| `ZIP_ERROR` | ZIP creation failed |

## ZIP Export Contract

```text
favicon.ico               # Multi-resolution ICO (16, 32, 48, 64, 128, 256)
favicon-16x16.png
favicon-32x32.png
favicon-48x48.png
favicon-64x64.png
favicon-128x128.png
apple-touch-icon.png       # 180x180
favicon-180x180.png        # 180x180
favicon-192x192.png
favicon-512x512.png
mstile-150x150.png         # 150x150 Windows tile
site.webmanifest
browserconfig.xml
```

The archive contains only these top-level paths. Generated manifest, XML, and
HTML references must resolve to members of this package.

## Language and Commit Conventions

- English is the default language for public documentation. Keep the complete
  Simplified Chinese README aligned with user-facing README changes.
- Write new or modified code comments in English.
- Write commit subjects and bodies in English and follow Conventional Commits.
- This repository-level rule overrides any global preference for another commit-message language.

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440 pixel widths, including console, keyboard, accessibility, and overflow checks.
