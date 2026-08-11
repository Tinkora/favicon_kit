pub mod error;
pub mod generate;
pub mod wasm;

pub use error::CoreError;
pub use generate::{
    FAVICON_SIZES, FaviconMetadata, FaviconOptions, FaviconOutput, FaviconSizes, SourceImage,
    generate_browserconfig, generate_favicons, generate_html_snippet, generate_manifest,
    generate_zip,
};
pub use wasm::{do_generate, do_generate_zip, parse_hex_color};
