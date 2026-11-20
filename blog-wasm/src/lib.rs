mod api;
mod auth;
mod components;
mod format;
mod models;
mod storage;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    yew::Renderer::<components::app::App>::new().render();
}
