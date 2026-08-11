use wasm_bindgen::prelude::*;

/// Initialization called automatically by wasm-pack. Sets up panic hook.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Converts a CoreError into a JsValue error carrying a stable `code` field.
fn core_err(e: favicon_kit_core::CoreError) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"code".into(), &e.code().into()).ok();
    js_sys::Reflect::set(&obj, &"message".into(), &e.to_string().into()).ok();
    obj.into()
}

/// Generate favicons from raw RGBA pixels. Returns a JS object with all output data.
///
/// Returns an object with:
/// - `ico_bytes`: Uint8Array
/// - `pngs`: Array<{size: number, data: Uint8Array}>
/// - `apple_touch_icon`: Uint8Array
/// - `manifest_json`: string
/// - `browserconfig_xml`: string
/// - `html_snippet`: string
#[wasm_bindgen]
#[expect(
    clippy::too_many_arguments,
    reason = "The browser ABI preserves the existing JavaScript function signature"
)]
pub fn wasm_generate_favicons(
    rgba_pixels: &[u8],
    width: u32,
    height: u32,
    bg_color_hex: &str,
    padding_percent: f32,
    app_name: &str,
    app_short_name: &str,
    theme_color: &str,
) -> Result<JsValue, JsValue> {
    let output = favicon_kit_core::generate_favicons(
        favicon_kit_core::SourceImage::new(rgba_pixels, width, height),
        favicon_kit_core::FaviconOptions {
            background_color: favicon_kit_core::parse_hex_color(bg_color_hex),
            padding_percent,
        },
        favicon_kit_core::FaviconMetadata::new(app_name, app_short_name, theme_color),
    )
    .map_err(core_err)?;

    // Build a JS object manually.
    let obj = js_sys::Object::new();

    // ICO bytes as Uint8Array
    let ico_arr = js_sys::Uint8Array::new_with_length(output.ico_bytes.len() as u32);
    ico_arr.copy_from(&output.ico_bytes);
    js_sys::Reflect::set(&obj, &"ico_bytes".into(), &ico_arr).ok();

    // PNGs as array of {size, data}
    let pngs_arr = js_sys::Array::new();
    for (size, data) in &output.pngs {
        let entry = js_sys::Object::new();
        js_sys::Reflect::set(&entry, &"size".into(), &(*size).into()).ok();
        let data_arr = js_sys::Uint8Array::new_with_length(data.len() as u32);
        data_arr.copy_from(data);
        js_sys::Reflect::set(&entry, &"data".into(), &data_arr).ok();
        pngs_arr.push(&entry);
    }
    js_sys::Reflect::set(&obj, &"pngs".into(), &pngs_arr).ok();

    // Apple touch icon
    let at_arr = js_sys::Uint8Array::new_with_length(output.apple_touch_icon.len() as u32);
    at_arr.copy_from(&output.apple_touch_icon);
    js_sys::Reflect::set(&obj, &"apple_touch_icon".into(), &at_arr).ok();

    // Strings
    js_sys::Reflect::set(
        &obj,
        &"manifest_json".into(),
        &output.manifest_json.as_str().into(),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"browserconfig_xml".into(),
        &output.browserconfig_xml.as_str().into(),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"html_snippet".into(),
        &output.html_snippet.as_str().into(),
    )
    .ok();

    Ok(obj.into())
}

/// Generate favicon ZIP from raw RGBA pixels. Returns the ZIP as a Uint8Array in JS.
#[wasm_bindgen]
#[expect(
    clippy::too_many_arguments,
    reason = "The browser ABI preserves the existing JavaScript function signature"
)]
pub fn wasm_generate_favicon_zip(
    rgba_pixels: &[u8],
    width: u32,
    height: u32,
    bg_color_hex: &str,
    padding_percent: f32,
    app_name: &str,
    app_short_name: &str,
    theme_color: &str,
) -> Result<Vec<u8>, JsValue> {
    let output = favicon_kit_core::generate_favicons(
        favicon_kit_core::SourceImage::new(rgba_pixels, width, height),
        favicon_kit_core::FaviconOptions {
            background_color: favicon_kit_core::parse_hex_color(bg_color_hex),
            padding_percent,
        },
        favicon_kit_core::FaviconMetadata::new(app_name, app_short_name, theme_color),
    )
    .map_err(core_err)?;

    favicon_kit_core::generate_zip(&output).map_err(core_err)
}

/// Return the default HTML snippet with link tags for favicons.
#[wasm_bindgen]
pub fn wasm_html_snippet(theme_color: &str, site_name: &str) -> String {
    favicon_kit_core::generate_html_snippet(theme_color, site_name)
}

/// Return the list of standard favicon sizes.
#[wasm_bindgen]
pub fn wasm_get_sizes() -> Box<[u32]> {
    favicon_kit_core::FAVICON_SIZES.to_vec().into_boxed_slice()
}
