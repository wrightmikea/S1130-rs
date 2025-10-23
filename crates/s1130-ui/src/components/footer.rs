//! Footer component with build information

use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    let git_sha = env!("GIT_SHA");
    let timestamp = env!("BUILD_TIMESTAMP");
    let hostname = env!("BUILD_HOSTNAME");

    html! {
        <footer class="app-footer">
            <div class="footer-content">
                <span class="build-info">
                    { format!("Build: {} @ {} | {}", git_sha, hostname, timestamp) }
                </span>
                <span class="phase-info">
                    { "Development: UI Complete, Integrating WASM • 161 tests passing" }
                </span>
            </div>
        </footer>
    }
}
