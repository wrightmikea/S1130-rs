//! S1130 UI - Yew-based web interface for IBM 1130 emulator

mod app;
mod components;

use app::App;

fn main() {
    // Enable panic hooks for better debugging
    console_error_panic_hook::set_once();

    // Render the Yew app
    yew::Renderer::<App>::new().render();
}
