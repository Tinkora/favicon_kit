# Favicon Kit 产品规格

[English](product_spec.md)

当前产品版本：v0.1.0。

## 产品一句话

开发者在浏览器中选择一张 Logo 图片；浏览器完成解码后，由 Rust/WASM 生成各平台所需的
favicon 尺寸和格式。

## 目标用户

- 需要为网站/Web 应用快速生成全套 favicon 的前端开发者。
- 不想安装任何桌面软件、不想上传图片到第三方服务的隐私敏感用户。
- 需要批量生成不同尺寸图标的 PWA 开发者。

## 核心体验

1. 用户打开页面，拖入或选择一张 Logo/图标图片。
2. 系统自动识别图片尺寸，预览原始图片。
3. 用户可调整背景填充色（用于非正方形原图的 padding 区域）和 padding 百分比。
4. 系统实时生成所有标准尺寸的预览卡片。
5. 浏览器标签页模拟预览显示实际效果。
6. 用户查看 HTML `<link>` 标签代码片段，可一键复制。
7. 用户下载单个 PNG 或一键打包 ZIP。

## 输出

### 单独 PNG 下载

每个尺寸可单独下载 PNG 文件。

### ZIP 打包下载

ZIP 包含以下固定路径成员：

```text
favicon.ico              # 多分辨率 ICO（16, 32, 48, 64, 128, 256）
favicon-16x16.png
favicon-32x32.png
favicon-48x48.png
favicon-64x64.png
favicon-128x128.png
apple-touch-icon.png      # 180×180
favicon-180x180.png       # 180×180
favicon-192x192.png
favicon-512x512.png
mstile-150x150.png       # browserconfig.xml 引用的 Windows tile 图标
site.webmanifest
browserconfig.xml
```

### HTML 代码片段

生成标准的 `<link>` 和 `<meta>` 标签，可直接粘贴到 HTML `<head>` 中：

```html
<link rel="icon" type="image/x-icon" href="/favicon.ico">
<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png">
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
<link rel="manifest" href="/site.webmanifest">
<meta name="msapplication-config" content="/browserconfig.xml">
<meta name="theme-color" content="#ffffff">
```

## 技术规格

### 输入约束

- 浏览器 UI 支持当前浏览器 `createImageBitmap` 能解码的格式；Rust core 只接收原始 RGBA 像素
- 最小尺寸：16×16 像素
- 最大尺寸：4096×4096 像素
- 建议使用正方形 Logo，非正方形将自动居中并填充背景色

### 缩放算法

- 使用 Lanczos3 算法进行高质量缩放
- 保持纵横比，居中裁剪或填充

### ICO 格式

- 使用现代 ICO 格式（PNG 内嵌），兼容所有现代浏览器和操作系统
- 单文件包含 16、32、48、64、128、256 六种分辨率

### PWA Manifest

- 生成符合 W3C 标准的 `site.webmanifest`
- 包含所有必需字段：name、icons、start_url、display
- PWA 图标使用 `purpose: "any"`，未经过专用 safe-zone 处理时不宣称支持 `maskable`

## MVP 范围

- 拖放和文件选择上传
- 所有标准尺寸生成（8 种）
- favicon.ico 多分辨率文件
- apple-touch-icon（180×180）
- PWA 图标（192×192、512×512）
- site.webmanifest 生成
- browserconfig.xml 生成
- 与 browserconfig.xml 匹配的 150×150 Windows tile PNG
- HTML 代码片段生成与复制
- 单个 PNG 下载
- ZIP 打包下载
- 背景色选择器
- Padding 百分比滑块
- 各尺寸实时预览卡片
- 浏览器标签页模拟预览
- 浏览器负责图片解码，Rust/WASM 负责图标与元数据生成

## 非目标

- 不提供 SVG favicon 生成（首版）
- 不提供 AI 图标生成
- 不支持动画 favicon
- 不支持 macOS ICNS 格式
- 不提供在线存储或 CDN
- 不提供可运行的 MCP transport

## 隐私边界

- 图片由浏览器解码为 RGBA 后交给 Rust/WASM 处理；编辑器没有上传原图或生成文件的应用接口。
- 该边界不覆盖部署主机、浏览器扩展、浏览器遥测或浏览器崩溃报告。

## 验收标准

- 拖入 512×512 PNG Logo，能生成所有标准尺寸
- favicon.ico 可被浏览器正确解析并显示多分辨率
- ZIP 下载包结构完整，所有文件可正常打开
- manifest、browserconfig.xml 与 HTML 片段中的每个本地引用都能在 ZIP 中找到
- 非正方形原图 padding 填充色正确
- HTML 代码片段可直接复制使用
- 各尺寸预览与下载文件一致
- WASM 大小 < 2MB（优化后）
- 页面在 Chrome、Firefox、Safari 上正常工作
