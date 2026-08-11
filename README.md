# Favicon Kit

Favicon Kit v0.1.0 generates browser-local favicon packages for web
applications and PWAs. Start with one image and export a deterministic set of
PNG, ICO, manifest, and HTML metadata files.

[完整简体中文 README](README.zh-CN.md)

## What it does

- Generates browser, Apple touch, PWA, and Windows tile icon assets.
- Creates a multi-resolution `favicon.ico` containing 16, 32, 48, 64, 128,
  and 256 pixel PNG entries.
- Exports `site.webmanifest`, `browserconfig.xml`, and an HTML `<head>`
  snippet that refer only to files in the downloaded package.
- Resizes in Rust/WASM with Lanczos3 and keeps image processing in the browser.
- Lets users set a padding background, padding amount, application names, and
  theme color before downloading the result.

Favicon Kit is deliberately not an icon editor, an AI image generator, an
image host, a CDN, or a runnable MCP server. The schemas under `skills/` are
non-runnable integration drafts.

## Use the web app

Requirements: Rust 1.95 or later, wasm-pack 0.15 or later, and the
`wasm32-unknown-unknown` Rust target.

Build the WASM package, copy it beside the static editor, and serve that
directory:

```bash
rustup target add wasm32-unknown-unknown
wasm-pack build --target web crates/favicon_kit_web
mkdir -p crates/favicon_kit_web/static/pkg
cp crates/favicon_kit_web/pkg/* crates/favicon_kit_web/static/pkg/
python3 -m http.server --directory crates/favicon_kit_web/static 8080
```

Open `http://localhost:8080`. For Pages deployment, publish
`crates/favicon_kit_web/static/` after its `pkg/` directory is populated; the
repository root has no separate marketing page.

The editor has no application endpoint for uploading source images or generated
files. This describes the application code only: a deployment host, browser
extensions, browser telemetry, and browser crash reporting remain outside that
boundary.

## Download contents

The ZIP uses these fixed top-level paths:

```text
favicon.ico
favicon-16x16.png
favicon-32x32.png
favicon-48x48.png
favicon-64x64.png
favicon-128x128.png
apple-touch-icon.png
favicon-180x180.png
favicon-192x192.png
favicon-512x512.png
mstile-150x150.png
site.webmanifest
browserconfig.xml
```

`favicon.ico` is the only file that contains the 256 pixel representation.

## Input limits

- The browser UI accepts image formats its `createImageBitmap` implementation
  can decode.
- Each source dimension must be within the inclusive range of 16 to 4096
  pixels.
- The Rust core accepts raw RGBA pixels; it does not decode arbitrary image
  files itself.

## Development

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p favicon_kit_core --target wasm32-unknown-unknown --locked
cargo check -p favicon_kit_web --target wasm32-unknown-unknown --locked
```

## Documentation

- [Product specification](docs/product_spec.md)
- [中文产品规格](docs/product_spec.zh-CN.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [Changelog](CHANGELOG.md)

## License

[MIT](LICENSE) © [Tinkora](https://github.com/Tinkora)
