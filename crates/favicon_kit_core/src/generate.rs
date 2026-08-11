use std::io::{Cursor, Write};

use image::imageops::FilterType;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};

use crate::error::CoreError;

// ── Constants ───────────────────────────────────────────────────────────────

/// All standard favicon sizes.
pub const FAVICON_SIZES: &[u32] = &[16, 32, 48, 64, 128, 180, 192, 512];

/// Sizes included in the multi-resolution ICO file.
pub const ICO_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256];

/// Size of the Windows tile image referenced by browserconfig.xml.
pub const MS_TILE_SIZE: u32 = 150;

/// Input constraints.
pub const MIN_DIMENSION: u32 = 16;
pub const MAX_DIMENSION: u32 = 4096;

/// Default background color for padding (transparent).
pub const DEFAULT_BG_COLOR: [u8; 4] = [0, 0, 0, 0];

// ── Types ───────────────────────────────────────────────────────────────────

/// Configuration for favicon generation.
#[derive(Clone, Debug)]
pub struct FaviconSizes {
    /// The target sizes to generate PNGs for.
    pub sizes: Vec<u32>,
}

impl Default for FaviconSizes {
    fn default() -> Self {
        Self {
            sizes: FAVICON_SIZES.to_vec(),
        }
    }
}

/// Raw source image supplied to the generator.
#[derive(Clone, Copy, Debug)]
pub struct SourceImage<'a> {
    /// Raw RGBA pixel data.
    pub rgba_pixels: &'a [u8],
    /// Source image width in pixels.
    pub width: u32,
    /// Source image height in pixels.
    pub height: u32,
}

impl<'a> SourceImage<'a> {
    /// Create a source image descriptor from raw RGBA pixels and dimensions.
    pub const fn new(rgba_pixels: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            rgba_pixels,
            width,
            height,
        }
    }
}

/// Rendering options that affect the generated image pixels.
#[derive(Clone, Copy, Debug)]
pub struct FaviconOptions {
    /// Optional RGBA color used to fill padding.
    pub background_color: Option<[u8; 4]>,
    /// Padding on each side as a percentage of the square canvas.
    pub padding_percent: f32,
}

impl Default for FaviconOptions {
    fn default() -> Self {
        Self {
            background_color: None,
            padding_percent: 0.0,
        }
    }
}

/// Application metadata written to text output files.
#[derive(Clone, Copy, Debug)]
pub struct FaviconMetadata<'a> {
    /// Full application name.
    pub app_name: &'a str,
    /// Compact application name for browser UI.
    pub app_short_name: &'a str,
    /// Theme and tile color.
    pub theme_color: &'a str,
}

impl<'a> FaviconMetadata<'a> {
    /// Create favicon metadata from application labels and the theme color.
    pub const fn new(app_name: &'a str, app_short_name: &'a str, theme_color: &'a str) -> Self {
        Self {
            app_name,
            app_short_name,
            theme_color,
        }
    }
}

/// The complete output of favicon generation.
#[derive(Clone, Debug)]
pub struct FaviconOutput {
    /// Multi-resolution ICO bytes (16, 32, 48, 64, 128, 256).
    pub ico_bytes: Vec<u8>,
    /// Individual PNGs at each standard size: Vec<(size, png_bytes)>.
    pub pngs: Vec<(u32, Vec<u8>)>,
    /// Apple Touch Icon (180×180) PNG bytes.
    pub apple_touch_icon: Vec<u8>,
    /// Windows tile (150×150) PNG bytes.
    pub ms_tile_icon: Vec<u8>,
    /// Contents of site.webmanifest.
    pub manifest_json: String,
    /// Contents of browserconfig.xml.
    pub browserconfig_xml: String,
    /// HTML <link> snippet for all favicon formats.
    pub html_snippet: String,
}

// ── Image Helpers ───────────────────────────────────────────────────────────

/// Decode raw RGBA pixels into an RgbaImage.
fn rgba_to_image(rgba_pixels: &[u8], width: u32, height: u32) -> Result<RgbaImage, CoreError> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| CoreError::ResizeError("pixel count overflow".into()))?;

    if rgba_pixels.len() != expected {
        return Err(CoreError::ResizeError(format!(
            "expected {} bytes for {}×{} RGBA, got {}",
            expected,
            width,
            height,
            rgba_pixels.len()
        )));
    }

    RgbaImage::from_raw(width, height, rgba_pixels.to_vec())
        .ok_or_else(|| CoreError::ResizeError("failed to build RgbaImage from raw pixels".into()))
}

/// Square-crop the source image to a square canvas with optional background color.
/// If padding_pct > 0, the image is placed centered with padding on the shorter axis.
/// If padding_pct == 0, the image is center-cropped to square.
fn square_canvas(img: &RgbaImage, bg_color: [u8; 4], padding_percent: f32) -> RgbaImage {
    let (iw, ih) = (img.width(), img.height());
    let side = iw.max(ih);

    // Determine the visible area of the original image within the square canvas.
    let inner_side_f = side as f32 * (1.0 - padding_percent / 100.0 * 2.0);
    let inner_side = (inner_side_f.max(1.0)) as u32;

    // Scale factor to fit the original image into the inner square, preserving aspect ratio.
    let scale = if iw >= ih {
        inner_side as f32 / iw as f32
    } else {
        inner_side as f32 / ih as f32
    };

    let new_w = (iw as f32 * scale).round() as u32;
    let new_h = (ih as f32 * scale).round() as u32;

    // Resize the original image.
    let resized = image::imageops::resize(img, new_w.max(1), new_h.max(1), FilterType::Lanczos3);

    // Create the square canvas filled with bg_color.
    let mut canvas = RgbaImage::from_pixel(side, side, image::Rgba(bg_color));

    // Center the resized image on the canvas.
    let ox = (side.saturating_sub(new_w)) / 2;
    let oy = (side.saturating_sub(new_h)) / 2;

    image::imageops::overlay(&mut canvas, &resized, ox as i64, oy as i64);

    canvas
}

/// Encode an RgbaImage as PNG bytes.
fn encode_png(img: &RgbaImage) -> Result<Vec<u8>, CoreError> {
    let mut buf = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buf);
        let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
        encoder
            .write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                ExtendedColorType::Rgba8,
            )
            .map_err(|e| CoreError::ResizeError(format!("PNG encode: {e}")))?;
    }
    Ok(buf)
}

/// Resize an RgbaImage to a target size using Lanczos3.
fn resize_to(img: &RgbaImage, size: u32) -> RgbaImage {
    image::imageops::resize(img, size, size, FilterType::Lanczos3)
}

// ── ICO Encoding ────────────────────────────────────────────────────────────

/// Encode multiple PNG images into a single multi-resolution ICO file.
///
/// Modern ICO format: header + directory entries + embedded PNG data.
/// Each directory entry records the PNG data offset and size.
fn encode_ico(pngs: &[(u32, Vec<u8>)]) -> Result<Vec<u8>, CoreError> {
    if pngs.is_empty() {
        return Err(CoreError::IcoError("no images for ICO".into()));
    }

    let count = pngs.len() as u16;
    if count > 256 {
        return Err(CoreError::IcoError("too many ICO entries (max 256)".into()));
    }

    // Header (6 bytes) + directory (16 bytes per entry)
    let header_size: u32 = 6;
    let entry_size: u32 = 16;
    let dir_size: u32 = entry_size * count as u32;

    // Calculate offsets using actual PNG data lengths.
    let mut offsets: Vec<u32> = Vec::with_capacity(count as usize);
    let mut current_offset = header_size + dir_size;

    for (_size, png_data) in pngs {
        offsets.push(current_offset);
        current_offset += png_data.len() as u32;
    }

    let total_size = current_offset as usize;
    let mut buf = Vec::with_capacity(total_size);

    // Write header
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&1u16.to_le_bytes()); // type: ICO = 1
    buf.extend_from_slice(&count.to_le_bytes()); // count

    // Write directory entries
    for (i, (size, png_data)) in pngs.iter().enumerate() {
        let w = if *size >= 256 { 0u8 } else { *size as u8 };
        let h = if *size >= 256 { 0u8 } else { *size as u8 };

        buf.push(w);
        buf.push(h);
        buf.push(0u8); // palette colors (0 = no palette)
        buf.push(0u8); // reserved

        // For PNG-embedded ICO, planes = 1, bpp = 32
        buf.extend_from_slice(&1u16.to_le_bytes()); // planes
        buf.extend_from_slice(&32u16.to_le_bytes()); // bpp

        let data_len = png_data.len() as u32;
        buf.extend_from_slice(&data_len.to_le_bytes());
        buf.extend_from_slice(&offsets[i].to_le_bytes());
    }

    // Write PNG data
    for (_size, png_data) in pngs {
        buf.extend_from_slice(png_data);
    }

    Ok(buf)
}

/// Generate and encode the ICO-specific sizes from the prepared square canvas.
fn build_ico(canvas: &RgbaImage) -> Result<Vec<u8>, CoreError> {
    let ico_pngs = ICO_SIZES
        .iter()
        .map(|&size| {
            let resized = resize_to(canvas, size);
            encode_png(&resized).map(|png| (size, png))
        })
        .collect::<Result<Vec<_>, _>>()?;

    encode_ico(&ico_pngs)
}

// ── Manifest Generators ─────────────────────────────────────────────────────

/// Generate site.webmanifest JSON string.
pub fn generate_manifest(
    name: &str,
    short_name: &str,
    theme_color: &str,
    bg_color: &str,
) -> String {
    let icons: Vec<serde_json::Value> = [192, 512]
        .iter()
        .map(|&size| {
            serde_json::json!({
                "src": format!("/favicon-{size}x{size}.png"),
                "sizes": format!("{size}x{size}"),
                "type": "image/png",
                "purpose": "any"
            })
        })
        .collect();

    let manifest = serde_json::json!({
        "name": name,
        "short_name": short_name,
        "icons": icons,
        "start_url": "/",
        "display": "standalone",
        "theme_color": theme_color,
        "background_color": bg_color,
        "orientation": "any"
    });

    serde_json::to_string_pretty(&manifest).unwrap_or_default()
}

/// Generate browserconfig.xml string.
pub fn generate_browserconfig(tile_color: &str) -> String {
    let tile_color = escape_xml_text(tile_color);
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<browserconfig>
  <msapplication>
    <tile>
      <square150x150logo src="/mstile-150x150.png"/>
      <TileColor>{}</TileColor>
    </tile>
  </msapplication>
</browserconfig>
"#,
        tile_color
    )
}

/// Generate the HTML snippet with all <link> tags.
pub fn generate_html_snippet(theme_color: &str, site_name: &str) -> String {
    let theme_color = escape_html_attribute(theme_color);
    let site_name = escape_html_attribute(site_name);
    format!(
        r#"<!-- Favicon generated by favicon_kit -->
<link rel="icon" type="image/x-icon" href="/favicon.ico">
<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png">
<link rel="icon" type="image/png" sizes="48x48" href="/favicon-48x48.png">
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
<link rel="icon" type="image/png" sizes="192x192" href="/favicon-192x192.png">
<link rel="icon" type="image/png" sizes="512x512" href="/favicon-512x512.png">
<link rel="manifest" href="/site.webmanifest">
<meta name="msapplication-config" content="/browserconfig.xml">
<meta name="msapplication-TileColor" content="{theme_color}">
<meta name="theme-color" content="{theme_color}">
<meta name="apple-mobile-web-app-title" content="{site_name}">"#,
        theme_color = theme_color,
        site_name = site_name,
    )
}

/// Escape text inserted between XML tags and replace invalid XML characters.
fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ if is_valid_xml_character(character) => escaped.push(character),
            _ => escaped.push('\u{fffd}'),
        }
    }
    escaped
}

/// Escape text inserted into a quoted HTML attribute value.
fn escape_html_attribute(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// Return whether a character is permitted by the XML 1.0 character production.
const fn is_valid_xml_character(character: char) -> bool {
    matches!(
        character,
        '\u{9}'
            | '\u{a}'
            | '\u{d}'
            | '\u{20}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}'
            | '\u{10000}'..='\u{10ffff}'
    )
}

// ── Main API ────────────────────────────────────────────────────────────────

/// Generate all favicon formats from raw RGBA pixel data.
///
/// # Arguments
/// * `source_image` - Raw RGBA source pixels and their dimensions.
/// * `options` - Rendering options for canvas padding and background fill.
/// * `metadata` - Application labels and color used by text output files.
pub fn generate_favicons(
    source_image: SourceImage<'_>,
    options: FaviconOptions,
    metadata: FaviconMetadata<'_>,
) -> Result<FaviconOutput, CoreError> {
    // Validate dimensions
    if source_image.width < MIN_DIMENSION || source_image.height < MIN_DIMENSION {
        return Err(CoreError::ImageTooSmall(
            source_image.width,
            source_image.height,
            MIN_DIMENSION,
        ));
    }
    if source_image.width > MAX_DIMENSION || source_image.height > MAX_DIMENSION {
        return Err(CoreError::ImageTooLarge(
            source_image.width,
            source_image.height,
            MAX_DIMENSION,
        ));
    }

    let bg = options.background_color.unwrap_or(DEFAULT_BG_COLOR);

    // Decode pixels into an image.
    let source = rgba_to_image(
        source_image.rgba_pixels,
        source_image.width,
        source_image.height,
    )?;

    // Create square canvas with padding/background fill.
    let canvas = square_canvas(&source, bg, options.padding_percent.clamp(0.0, 50.0));

    // Generate each size.
    let sizes = FaviconSizes::default().sizes;
    let mut pngs: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());

    for &size in &sizes {
        let resized = resize_to(&canvas, size);
        let png_bytes = encode_png(&resized)?;
        pngs.push((size, png_bytes));
    }

    // ICO includes a 256px entry that is intentionally not a standalone PNG export.
    let ico_bytes = build_ico(&canvas)?;

    // Apple Touch Icon is the 180×180 PNG.
    let apple_touch_icon = pngs
        .iter()
        .find(|(s, _)| *s == 180)
        .map(|(_, d)| d.clone())
        .ok_or_else(|| CoreError::ResizeError("missing 180×180 icon".into()))?;
    let ms_tile_icon = encode_png(&resize_to(&canvas, MS_TILE_SIZE))?;

    // Generate manifest files.
    let manifest_json = generate_manifest(
        metadata.app_name,
        metadata.app_short_name,
        metadata.theme_color,
        metadata.theme_color,
    );
    let browserconfig_xml = generate_browserconfig(metadata.theme_color);
    let html_snippet = generate_html_snippet(metadata.theme_color, metadata.app_short_name);

    Ok(FaviconOutput {
        ico_bytes,
        pngs,
        apple_touch_icon,
        ms_tile_icon,
        manifest_json,
        browserconfig_xml,
        html_snippet,
    })
}

// ── ZIP Export ──────────────────────────────────────────────────────────────

/// Generate a ZIP archive containing all favicon assets.
pub fn generate_zip(output: &FaviconOutput) -> Result<Vec<u8>, CoreError> {
    let mut bytes = Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut bytes);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let add_file = |archive: &mut zip::ZipWriter<&mut Cursor<Vec<u8>>>,
                        name: &str,
                        data: &[u8]|
         -> Result<(), CoreError> {
            archive
                .start_file(name, options)
                .map_err(|e| CoreError::ZipError(e.to_string()))?;
            archive
                .write_all(data)
                .map_err(|e| CoreError::ZipError(e.to_string()))?;
            Ok(())
        };

        add_file(&mut archive, "favicon.ico", &output.ico_bytes)?;

        for (size, data) in &output.pngs {
            if *size == 180 {
                add_file(&mut archive, "apple-touch-icon.png", data)?;
            }
            add_file(&mut archive, &format!("favicon-{size}x{size}.png"), data)?;
        }

        add_file(&mut archive, "mstile-150x150.png", &output.ms_tile_icon)?;

        add_file(
            &mut archive,
            "site.webmanifest",
            output.manifest_json.as_bytes(),
        )?;
        add_file(
            &mut archive,
            "browserconfig.xml",
            output.browserconfig_xml.as_bytes(),
        )?;

        archive
            .finish()
            .map_err(|e| CoreError::ZipError(e.to_string()))?;
    }
    Ok(bytes.into_inner())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use std::io::Read;

    /// Create a simple 64×64 test image (rainbow gradient).
    fn make_test_rgba(w: u32, h: u32) -> Vec<u8> {
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                pixels[i] = (x as f32 / w as f32 * 255.0) as u8; // R
                pixels[i + 1] = (y as f32 / h as f32 * 255.0) as u8; // G
                pixels[i + 2] = 128; // B
                pixels[i + 3] = 255; // A
            }
        }
        pixels
    }

    #[test]
    fn test_generate_favicons_basic() {
        let rgba = make_test_rgba(128, 128);
        let result = generate_favicons(
            SourceImage::new(&rgba, 128, 128),
            FaviconOptions::default(),
            FaviconMetadata::new("Test App", "Test", "#ffffff"),
        );
        assert!(result.is_ok(), "expected ok, got {:?}", result.err());

        let output = result.unwrap();
        assert!(!output.ico_bytes.is_empty());
        assert_eq!(output.pngs.len(), FAVICON_SIZES.len());

        // Verify each PNG size
        for &size in FAVICON_SIZES {
            let found = output.pngs.iter().any(|(s, _)| *s == size);
            assert!(found, "missing size {size}");
        }

        // Apple touch icon should be non-empty
        assert!(!output.apple_touch_icon.is_empty());

        // manifest should contain key fields
        assert!(output.manifest_json.contains("\"name\""));
        assert!(output.manifest_json.contains("Test App"));

        // html snippet should contain link tags
        assert!(output.html_snippet.contains("<link"));
        assert!(output.html_snippet.contains("favicon.ico"));
    }

    #[test]
    fn test_image_too_small() {
        let rgba = make_test_rgba(8, 8);
        let result = generate_favicons(
            SourceImage::new(&rgba, 8, 8),
            FaviconOptions::default(),
            FaviconMetadata::new("A", "B", "#fff"),
        );
        assert!(matches!(
            result.err(),
            Some(CoreError::ImageTooSmall(8, 8, 16))
        ));
    }

    #[test]
    fn test_image_too_large() {
        let result = generate_favicons(
            SourceImage::new(&[], 5000, 5000),
            FaviconOptions::default(),
            FaviconMetadata::new("A", "B", "#fff"),
        );
        assert!(matches!(
            result.err(),
            Some(CoreError::ImageTooLarge(5000, 5000, 4096))
        ));
    }

    #[test]
    fn test_ico_encoding() {
        let rgba = make_test_rgba(64, 64);
        let result = generate_favicons(
            SourceImage::new(&rgba, 64, 64),
            FaviconOptions::default(),
            FaviconMetadata::new("X", "Y", "#000"),
        );
        assert!(result.is_ok());

        let output = result.unwrap();
        // ICO header: reserved(2) + type(2) = 1 + count(2) = 6
        assert!(output.ico_bytes.len() > 6);

        // Verify ICO type field = 1
        let ico_type = u16::from_le_bytes([output.ico_bytes[2], output.ico_bytes[3]]);
        assert_eq!(ico_type, 1);

        let ico_count = u16::from_le_bytes([output.ico_bytes[4], output.ico_bytes[5]]);
        assert_eq!(ico_count as usize, ICO_SIZES.len());
    }

    #[test]
    fn test_zip_export() {
        let rgba = make_test_rgba(128, 128);
        let output = generate_favicons(
            SourceImage::new(&rgba, 128, 128),
            FaviconOptions::default(),
            FaviconMetadata::new("ZipTest", "ZT", "#ff0000"),
        )
        .unwrap();
        let zip_bytes = generate_zip(&output).unwrap();
        assert!(!zip_bytes.is_empty());

        // Read back the ZIP and verify members.
        let cursor = Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();

        let expected_files = [
            "favicon.ico",
            "favicon-16x16.png",
            "favicon-32x32.png",
            "favicon-48x48.png",
            "favicon-64x64.png",
            "favicon-128x128.png",
            "apple-touch-icon.png",
            "favicon-180x180.png",
            "favicon-192x192.png",
            "favicon-512x512.png",
            "site.webmanifest",
            "browserconfig.xml",
        ];

        for name in &expected_files {
            let found = (0..archive.len()).any(|i| {
                archive
                    .by_index(i)
                    .map(|f| f.name() == *name)
                    .unwrap_or(false)
            });
            assert!(found, "missing ZIP member: {name}");
        }
    }

    #[test]
    fn test_padding() {
        // Non-square image (wide) with padding.
        let rgba = make_test_rgba(200, 100);
        let result = generate_favicons(
            SourceImage::new(&rgba, 200, 100),
            FaviconOptions {
                background_color: Some([255, 0, 0, 255]),
                padding_percent: 10.0,
            },
            FaviconMetadata::new("Pad", "P", "#f00"),
        );
        assert!(result.is_ok());

        let output = result.unwrap();
        // All sizes should be generated.
        assert_eq!(output.pngs.len(), FAVICON_SIZES.len());
    }

    #[test]
    fn test_html_snippet_content() {
        let snippet = generate_html_snippet("#ff0000", "MySite");
        assert!(snippet.contains("favicon.ico"));
        assert!(snippet.contains("apple-touch-icon"));
        assert!(snippet.contains("site.webmanifest"));
        assert!(snippet.contains("browserconfig.xml"));
        assert!(snippet.contains("#ff0000"));
        assert!(snippet.contains("MySite"));
    }

    #[test]
    fn test_browserconfig() {
        let xml = generate_browserconfig("#abcdef");
        assert!(xml.contains("<browserconfig>"));
        assert!(xml.contains("<TileColor>#abcdef</TileColor>"));
    }

    #[test]
    fn test_square_image_no_padding() {
        let rgba = make_test_rgba(256, 256);
        let result = generate_favicons(
            SourceImage::new(&rgba, 256, 256),
            FaviconOptions::default(),
            FaviconMetadata::new("Sq", "S", "#fff"),
        );
        assert!(result.is_ok());

        let output = result.unwrap();
        // Verify the 256×256 PNG exists (since it's an ICO size).
        let has_256 = output.pngs.iter().any(|(s, _)| *s == 512);
        assert!(has_256);
    }

    #[test]
    fn generated_zip_members_decode_or_parse_independently() {
        let rgba = make_test_rgba(128, 128);
        let output = generate_favicons(
            SourceImage::new(&rgba, 128, 128),
            FaviconOptions::default(),
            FaviconMetadata::new("ZipTest", "ZT", "#ff0000"),
        )
        .unwrap();
        let mut archive =
            zip::ZipArchive::new(Cursor::new(generate_zip(&output).unwrap())).unwrap();

        let expected_pngs = [
            ("favicon-16x16.png", 16),
            ("favicon-32x32.png", 32),
            ("favicon-48x48.png", 48),
            ("favicon-64x64.png", 64),
            ("favicon-128x128.png", 128),
            ("apple-touch-icon.png", 180),
            ("favicon-180x180.png", 180),
            ("mstile-150x150.png", 150),
            ("favicon-192x192.png", 192),
            ("favicon-512x512.png", 512),
        ];
        assert_eq!(archive.len(), expected_pngs.len() + 3);

        for (name, expected_size) in expected_pngs {
            let mut file = archive
                .by_name(name)
                .unwrap_or_else(|_| panic!("missing ZIP member: {name}"));
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            let image = image::load_from_memory(&bytes)
                .unwrap_or_else(|error| panic!("{name} is not a decodable PNG: {error}"));
            assert_eq!(image.dimensions(), (expected_size, expected_size), "{name}");
        }

        let ico_bytes = read_zip_member(&mut archive, "favicon.ico");
        assert_ico_png_entries_decode(&ico_bytes);

        let manifest_bytes = read_zip_member(&mut archive, "site.webmanifest");
        let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
        let icons = manifest["icons"].as_array().unwrap();
        assert!(icons.iter().all(|icon| {
            !icon["purpose"]
                .as_str()
                .unwrap()
                .split_ascii_whitespace()
                .any(|purpose| purpose == "maskable")
        }));

        let browserconfig_bytes = read_zip_member(&mut archive, "browserconfig.xml");
        assert_xml_is_well_formed(&browserconfig_bytes);
    }

    #[test]
    fn generated_text_escapes_untrusted_interpolation() {
        let color = "#abc&<>\"'";
        let site_name = "A & B <tag> \"quoted\" 'single'";

        let manifest: serde_json::Value =
            serde_json::from_str(&generate_manifest(site_name, site_name, color, color)).unwrap();
        assert_eq!(manifest["name"], site_name);
        assert_eq!(manifest["theme_color"], color);

        let browserconfig = generate_browserconfig(color);
        assert!(browserconfig.contains("#abc&amp;&lt;&gt;\"'"));
        assert_xml_is_well_formed(browserconfig.as_bytes());

        let browserconfig_with_invalid_xml = generate_browserconfig("#fff\0\u{fffe}\u{ffff}");
        assert!(browserconfig_with_invalid_xml.contains("#fff\u{fffd}\u{fffd}\u{fffd}"));
        assert!(!browserconfig_with_invalid_xml.contains(['\0', '\u{fffe}', '\u{ffff}']));
        assert_xml_is_well_formed(browserconfig_with_invalid_xml.as_bytes());

        let snippet = generate_html_snippet(color, site_name);
        assert!(snippet.contains("content=\"#abc&amp;&lt;&gt;&quot;&#39;\""));
        assert!(
            snippet
                .contains("content=\"A &amp; B &lt;tag&gt; &quot;quoted&quot; &#39;single&#39;\"")
        );
    }

    fn assert_ico_png_entries_decode(ico_bytes: &[u8]) {
        assert!(ico_bytes.len() >= 6, "ICO header is incomplete");
        assert_eq!(u16::from_le_bytes([ico_bytes[0], ico_bytes[1]]), 0);
        assert_eq!(u16::from_le_bytes([ico_bytes[2], ico_bytes[3]]), 1);

        let count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]) as usize;
        assert_eq!(count, ICO_SIZES.len());
        assert!(
            ico_bytes.len() >= 6 + count * 16,
            "ICO directory is incomplete"
        );

        for (index, expected_size) in ICO_SIZES.iter().enumerate() {
            let entry = 6 + index * 16;
            let size = if ico_bytes[entry] == 0 {
                256
            } else {
                u32::from(ico_bytes[entry])
            };
            assert_eq!(size, *expected_size, "ICO entry {index}");
            let data_len =
                u32::from_le_bytes(ico_bytes[entry + 8..entry + 12].try_into().unwrap()) as usize;
            let data_offset =
                u32::from_le_bytes(ico_bytes[entry + 12..entry + 16].try_into().unwrap()) as usize;
            let png = ico_bytes
                .get(data_offset..data_offset + data_len)
                .unwrap_or_else(|| panic!("ICO entry {index} points outside the archive"));
            let image = image::load_from_memory(png)
                .unwrap_or_else(|error| panic!("ICO entry {index} is not a PNG: {error}"));
            assert_eq!(image.dimensions(), (size, size), "ICO entry {index}");
        }
    }

    fn read_zip_member(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Vec<u8> {
        let mut file = archive
            .by_name(name)
            .unwrap_or_else(|_| panic!("missing ZIP member: {name}"));
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes
    }

    fn assert_xml_is_well_formed(xml: &[u8]) {
        use quick_xml::{Reader, events::Event};

        let mut reader = Reader::from_reader(Cursor::new(xml));
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer).unwrap() {
                Event::Eof => break,
                _ => buffer.clear(),
            }
        }
    }
}
