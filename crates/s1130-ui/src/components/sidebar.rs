//! Sidebar component with controls and status

use crate::cpu_context::use_cpu;
use gloo::console;
use yew::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let cpu_ctx = use_cpu();

    let on_step = {
        let cpu_context = (*cpu_ctx).clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Sidebar] Step button clicked");
            let mut cpu = cpu_context.cpu.borrow_mut();
            match cpu.step() {
                Ok(_) => {
                    console::log!("[Sidebar] Step executed successfully");
                }
                Err(e) => {
                    console::log!(format!("[Sidebar] Step error: {:?}", e));
                }
            }
        })
    };

    let on_run = {
        let cpu_context = (*cpu_ctx).clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Sidebar] Run button clicked");
            let mut cpu = cpu_context.cpu.borrow_mut();
            match cpu.run(100) {  // Run 100 instructions
                Ok(_) => {
                    console::log!("[Sidebar] Run completed successfully");
                }
                Err(e) => {
                    console::log!(format!("[Sidebar] Run error: {:?}", e));
                }
            }
        })
    };

    let on_reset = {
        let cpu_context = (*cpu_ctx).clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Sidebar] Reset button clicked");
            let mut cpu = cpu_context.cpu.borrow_mut();
            cpu.reset();
            console::log!("[Sidebar] CPU reset");
        })
    };

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
                    <button class="control-btn" onclick={on_run}>
                        { "Run" }
                    </button>
                    <button class="control-btn" onclick={on_step}>
                        { "Step" }
                    </button>
                    <button class="control-btn" onclick={on_reset}>
                        { "Reset" }
                    </button>
                </div>
                <p class="sidebar-note">{ "Step, Run, and Reset now functional!" }</p>
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
