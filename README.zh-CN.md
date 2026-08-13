# Favicon Kit

Favicon Kit v0.1.0 是一个在浏览器本地运行的 Web/PWA 图标资产生成器。输入一张图片后，
它会导出确定性的 PNG、ICO、manifest 与 HTML 元数据包。

[English](README.md)

[![在 Ko-fi 上支持 Tinkora](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/tinkora)

## 能力范围

- 生成浏览器图标、Apple Touch 图标、PWA 图标和 Windows tile 图标。
- 生成包含 16、32、48、64、128、256 像素 PNG 条目的多分辨率 `favicon.ico`。
- 导出 `site.webmanifest`、`browserconfig.xml` 和只引用包内文件的 HTML `<head>` 片段。
- 使用 Rust/WASM 和 Lanczos3 缩放，图片处理在浏览器内完成。
- 支持设置 padding 背景、padding 比例、应用名称和主题色。

本项目不是图像编辑器、AI 图片生成器、图像托管服务、CDN 或可运行的 MCP server。
`skills/` 下的 schema 只是不可运行的集成草案。

## 运行 Web 应用

环境要求：Rust 1.95 或更高版本、wasm-pack 0.15 或更高版本，以及
`wasm32-unknown-unknown` Rust target。

```bash
rustup target add wasm32-unknown-unknown
wasm-pack build --target web crates/favicon_kit_web
mkdir -p crates/favicon_kit_web/static/pkg
cp crates/favicon_kit_web/pkg/* crates/favicon_kit_web/static/pkg/
python3 -m http.server --directory crates/favicon_kit_web/static 8080
```

打开 `http://localhost:8080`。部署到 Pages 时，先填充 `pkg/` 目录，再直接发布
`crates/favicon_kit_web/static/`；仓库根目录不再保留独立营销页。

编辑器没有用于上传原始图片或生成文件的应用接口。这个边界只描述应用代码；部署主机、
浏览器扩展、浏览器遥测和浏览器崩溃报告不在该边界内。

## 下载包内容

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

256 像素图层只存在于 `favicon.ico` 中，不作为独立 PNG 导出。

## 输入边界

- 浏览器 UI 接受当前浏览器 `createImageBitmap` 能够解码的图片格式。
- 原图宽高必须在 16 到 4096 像素之间。
- Rust core 接收原始 RGBA 像素，本身不负责解码任意图片文件。

## 开发验证

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p favicon_kit_core --target wasm32-unknown-unknown --locked
cargo check -p favicon_kit_web --target wasm32-unknown-unknown --locked
```

浏览器 smoke 测试需要 Node.js 24：

```bash
cd crates/favicon_kit_web
npm ci
npx playwright install chromium
npm run test:wasm-smoke:local
```

## 文档

- [产品规格](docs/product_spec.zh-CN.md)
- [Product specification](docs/product_spec.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
- [支持](SUPPORT.md)
- [更新日志](CHANGELOG.md)

## 许可证

[MIT](LICENSE) © [Tinkora](https://github.com/Tinkora)
