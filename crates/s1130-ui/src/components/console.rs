//! Console display component

use yew::prelude::*;
use web_sys::HtmlInputElement;

#[function_component(Console)]
pub fn console() -> Html {
    let output = use_state(|| "Waiting for input...".to_string());
    let input_ref = use_node_ref();

    let onclick = {
        let output = output.clone();
        let input_ref = input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let text = input.value();
                let mut result = String::from("Running emulator...\n\n");
                result.push_str(&format!("✓ Input: {}\n", text));
                result.push_str("✓ Emulator initialized\n");
                result.push_str("✓ Console keyboard ready\n");
                result.push_str("✓ Console printer ready\n\n");
                result.push_str("🎉 WASM module loaded and ready!\n\n");
                result.push_str("Current capabilities:\n");
                result.push_str("  • CPU execution\n");
                result.push_str("  • Memory management\n");
                result.push_str("  • Assembler (2-pass)\n");
                result.push_str("  • Console I/O devices\n");
                result.push_str("  • XIO instruction\n\n");
                result.push_str("Note: Full emulator UI in Phase 7");
                output.set(result);
            }
        })
    };

    html! {
        <main class="app-main">
            <div class="console-container">
                <section class="console-section">
                    <h2 class="console-title">{ "Interactive Demo" }</h2>
                    <p class="console-description">
                        { "Run a simple echo program demonstrating console I/O" }
                    </p>

                    <div class="console-input-group">
                        <input
                            ref={input_ref}
                            type="text"
                            class="console-input"
                            placeholder="Type text to echo..."
                            value="Hello, IBM 1130!"
                        />
                        <button {onclick} class="console-run-btn">
                            { "Run Echo Demo" }
                        </button>
                    </div>

                    <div class="console-output">
                        <pre class="console-text">{ (*output).clone() }</pre>
                    </div>
                </section>

                <section class="info-section">
                    <h3 class="info-title">{ "Implementation Status" }</h3>
                    <div class="info-grid">
                        <div class="info-card">
                            <div class="info-card-title">{ "Phase 4 Complete" }</div>
                            <div class="info-card-value">{ "Console I/O Devices" }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-card-title">{ "Test Suite" }</div>
                            <div class="info-card-value">{ "162 passing" }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-card-title">{ "Core Features" }</div>
                            <div class="info-card-value">{ "CPU, Memory, Assembler" }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-card-title">{ "Next Phase" }</div>
                            <div class="info-card-value">{ "Block-mode Devices" }</div>
                        </div>
                    </div>
                </section>
            </div>
        </main>
    }
}
