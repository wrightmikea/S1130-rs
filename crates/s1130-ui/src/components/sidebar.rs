//! Sidebar component with controls and status

use yew::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <aside class="app-sidebar">
            <section class="sidebar-section">
                <h3 class="sidebar-title">{ "Status" }</h3>
                <div class="status-indicators">
                    <div class="status-item">
                        <span class="status-label">{ "CPU:" }</span>
                        <span class="status-value ready">{ "Ready" }</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{ "Memory:" }</span>
                        <span class="status-value ready">{ "32K words" }</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{ "Keyboard:" }</span>
                        <span class="status-value ready">{ "Ready" }</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{ "Printer:" }</span>
                        <span class="status-value ready">{ "Ready" }</span>
                    </div>
                </div>
            </section>

            <section class="sidebar-section">
                <h3 class="sidebar-title">{ "Controls" }</h3>
                <div class="control-buttons">
                    <button class="control-btn" disabled=true>
                        { "Load Program" }
                    </button>
                    <button class="control-btn" disabled=true>
                        { "Run" }
                    </button>
                    <button class="control-btn" disabled=true>
                        { "Step" }
                    </button>
                    <button class="control-btn" disabled=true>
                        { "Reset" }
                    </button>
                </div>
                <p class="sidebar-note">{ "Controls coming in Phase 7" }</p>
            </section>

            <section class="sidebar-section">
                <h3 class="sidebar-title">{ "Quick Info" }</h3>
                <div class="info-list">
                    <div class="info-item">{ "✓ Assembler (2-pass)" }</div>
                    <div class="info-item">{ "✓ Console I/O" }</div>
                    <div class="info-item">{ "✓ XIO instruction" }</div>
                    <div class="info-item">{ "✓ 28 opcodes" }</div>
                </div>
            </section>
        </aside>
    }
}
