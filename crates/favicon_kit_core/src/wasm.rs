use crate::error::CoreError;
use crate::generate::{
    FaviconMetadata, FaviconOptions, FaviconOutput, SourceImage, generate_favicons, generate_zip,
};

/// Parse a hex color string like "#RRGGBB" or "#RRGGBBAA" into [u8; 4].
/// Returns None if the string is empty or not a valid hex color.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 4]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.is_empty() {
        return None;
    }
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some([r, g, b, 255])
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some([r, g, b, a])
    } else {
        None
    }
}

/// Generate favicons from raw RGBA pixels. Returns the full FaviconOutput.
///
/// This is the core function that the WASM bridge wraps.
pub fn do_generate(
    source_image: SourceImage<'_>,
    options: FaviconOptions,
    metadata: FaviconMetadata<'_>,
) -> Result<FaviconOutput, CoreError> {
    generate_favicons(source_image, options, metadata)
}

/// Generate favicon ZIP from raw RGBA pixels.
pub fn do_generate_zip(
    source_image: SourceImage<'_>,
    options: FaviconOptions,
    metadata: FaviconMetadata<'_>,
) -> Result<Vec<u8>, CoreError> {
    let output = do_generate(source_image, options, metadata)?;
    generate_zip(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff0000"), Some([255, 0, 0, 255]));
        assert_eq!(parse_hex_color("#00FF00"), Some([0, 255, 0, 255]));
        assert_eq!(parse_hex_color("#12345678"), Some([0x12, 0x34, 0x56, 0x78]));
        assert_eq!(parse_hex_color("ff0000"), Some([255, 0, 0, 255]));
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#xyz"), None);
        assert_eq!(parse_hex_color("  #ffffff  "), Some([255, 255, 255, 255]));
    }
}
