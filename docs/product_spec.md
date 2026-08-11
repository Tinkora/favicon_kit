# Favicon Kit Product Specification

[简体中文](product_spec.zh-CN.md)

Current product version: v0.1.0.

## Purpose

Favicon Kit creates a validated, browser-local favicon asset package from one
source image. It is for developers who need a reproducible set of browser and
PWA assets without sending branding material to an application service.

## User workflow

1. Choose or drop an image the current browser can decode.
2. Set padding, padding background, application name, short name, and theme
   color.
3. Inspect the generated sizes and browser-tab preview.
4. Download individual PNGs, `favicon.ico`, the HTML snippet, or the complete
   ZIP package.

## Output contract

The complete ZIP contains exactly these top-level paths:

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

`favicon.ico` contains 16, 32, 48, 64, 128, and 256 pixel PNG entries.
`apple-touch-icon.png` and `favicon-180x180.png` are both 180 by 180 pixels.
`browserconfig.xml` references `mstile-150x150.png`. Every reference in
generated manifest, XML, and HTML output must resolve within the ZIP.

The PWA manifest includes 192 and 512 pixel PNG icons with `purpose: "any"`.
It does not claim maskable safe-zone processing.

## Boundaries

- Processing runs in Rust/WASM after the browser decodes an image into RGBA.
- The editor has no application endpoint for source images or generated files.
- Source dimensions are limited to 16 through 4096 pixels on both axes.
- Generated metadata encodes application names and colors for their JSON and
  HTML/XML contexts; the browser UI supplies hexadecimal CSS colors.
- Favicon Kit does not offer AI generation, image hosting, a CDN, or a runnable
  MCP transport.
- This boundary does not cover a deployment host, browser extension, browser
  telemetry, or browser crash reporting.

## Non-goals

- Editing SVG or macOS ICNS assets.
- Claiming that generated icons are maskable-safe without a dedicated safe-zone
  mode.
- Guaranteeing deployment-host, browser-extension, or browser telemetry
  behavior.

## Acceptance criteria

- A valid source image produces every documented ZIP member.
- PNG dimensions, ICO directory entries, ZIP members, manifest references, and
  HTML/XML references are verified by automated tests.
- Invalid dimensions and metadata fail with stable machine-readable errors.
- Native, WASM, browser, documentation, supply-chain, and release checks pass
  before a public tag is created.
