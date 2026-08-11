# Favicon Kit Guidance

Favicon Kit v0.1.0 is a browser-local favicon generation workbench. A user
selects an image in the static web editor and exports a Web/PWA asset package
generated with Rust/WASM.

`skills/mcp-tools.json` is a non-runnable integration draft. It is not a
manifest for an available MCP server, transport, Base64 adapter, or callable
agent tool.

## Workflow

1. Open the static Favicon Kit editor locally or from a deployment of
   `crates/favicon_kit_web/static/`.
2. Select a logo or icon image that the current browser can decode.
3. Adjust the padding fill color, padding percentage, application names, and
   theme color.
4. Generate and inspect the size previews and browser-tab preview.
5. Download individual PNGs, `favicon.ico`, the HTML snippet, or the complete
   ZIP package.

## Output Package

The complete ZIP has these fixed top-level paths:

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

The ICO contains 16, 32, 48, 64, 128, and 256 pixel entries.
`apple-touch-icon.png` and `favicon-180x180.png` are 180 by 180 pixels.

## Communication Rules

- Do not describe the draft schema as callable or as a runnable MCP integration.
- Describe the browser application as browser-local image processing, without
  making absolute claims about its deployment host, extensions, or browser
  telemetry.
- The generated HTML snippet assumes website-root paths. Users deploying to a
  subdirectory must update the generated `href` and `src` paths.
- For non-square source images, padding uses the configured background color.

## Privacy & Security

- The editor has no application endpoint for uploading image pixels or generated
  files.
- Generated files download through the browser. Deployment hosts, browser
  extensions, browser telemetry, and browser crash reporting are outside this
  application-code boundary.
