// Die verschachtelten Leptos-View-Typen sprengen im Release-Build das Standardlimit.
#![recursion_limit = "256"]

pub mod app;
pub mod backend;
pub mod components;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
