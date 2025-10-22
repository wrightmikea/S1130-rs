//! S1130 UI - Yew-based web interface for IBM 1130 emulator

mod app;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
