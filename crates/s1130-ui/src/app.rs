//! Main application component

use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app-container">
            <header class="app-header">
                <h1>{ "IBM 1130 Emulator" }</h1>
                <p>{ "Rust + Yew + WebAssembly" }</p>
            </header>
            <main class="app-main">
                <div class="placeholder">
                    <p>{ "Emulator UI will be implemented in Phase 7" }</p>
                    <p>{ "Current: Phase 0 - Project scaffolding complete" }</p>
                </div>
            </main>
        </div>
    }
}
