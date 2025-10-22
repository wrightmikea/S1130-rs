//! Compact header component

use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="app-header">
            <div class="header-content">
                <h1 class="header-title">{ "IBM 1130 Emulator" }</h1>
                <span class="header-subtitle">{ "Rust + WebAssembly" }</span>
            </div>
        </header>
    }
}
