//! Bundled browser UI HTML. The `build.rs` script materialises this
//! into `OUT_DIR` either by running `npm run build` (dev checkout) or
//! by copying `prebuilt/ui.html` (crates.io install).

pub(super) const HTML_TEMPLATE: &str = include_str!(concat!(env!("OUT_DIR"), "/ui.html"));
