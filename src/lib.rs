#![recursion_limit = "256"]

pub mod app;
pub mod backend;
pub mod convert;
pub mod frontend;
pub mod models;
pub mod routes;
pub mod shared;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
