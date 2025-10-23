//! Assembler view - editor and output

use crate::cpu_context::use_cpu;
use serde::Deserialize;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Debug, Deserialize)]
struct AssemblyResult {
    success: bool,
    #[serde(default)]
    origin: Option<u16>,
    #[serde(rename = "entryPoint")]
    entry_point: Option<u16>,
    #[serde(rename = "codeSize")]
    code_size: Option<usize>,
    message: String,
    #[serde(default)]
    errors: Vec<String>,
}

#[function_component(AssemblerView)]
pub fn assembler_view() -> Html {
    let cpu_ctx = use_cpu();
    let sample_code = "*\n* Sample IBM 1130 Assembly Program\n*\n    ORG  /100\nSTART LDX 1 COUNT\n    LD  A+1\n    A   A\n    STO A+1\n    MDX 1 -1\n    BNZ START\n    WAIT\n\nA    DC  1\nCOUNT DC  10\n    END START\n";

    let code = use_state(|| sample_code.to_string());
    let output = use_state(|| "Ready to assemble...".to_string());
    let status = use_state(|| "Ready".to_string());
    let error_count = use_state(|| 0usize);
    let success = use_state(|| false);
    let editor_ref = use_node_ref();

    let line_count = code.lines().count();

    let on_code_change = {
        let code = code.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(textarea) = e.target_dyn_into::<HtmlTextAreaElement>() {
                code.set(textarea.value());
            }
        })
    };

    let on_assemble = {
        let code = code.clone();
        let output = output.clone();
        let status = status.clone();
        let error_count = error_count.clone();
        let success = success.clone();
        let cpu_ctx = cpu_ctx.clone();

        Callback::from(move |_: MouseEvent| {
            status.set("Assembling...".to_string());
            output.set("Assembling source code...\n".to_string());

            let mut cpu = cpu_ctx.cpu.borrow_mut();
            match cpu.assemble(&code) {
                Ok(result_value) => {
                    if let Ok(result) =
                        serde_wasm_bindgen::from_value::<AssemblyResult>(result_value)
                    {
                        if result.success {
                            success.set(true);
                            error_count.set(0);
                            status.set("Success".to_string());

                            let mut msg = format!("✓ {}\n\n", result.message);
                            if let Some(origin) = result.origin {
                                msg.push_str(&format!("Origin: 0x{:04X}\n", origin));
                            }
                            if let Some(entry) = result.entry_point {
                                msg.push_str(&format!("Entry Point: 0x{:04X}\n", entry));
                            }
                            if let Some(size) = result.code_size {
                                msg.push_str(&format!("Code Size: {} words\n", size));
                            }
                            msg.push_str("\nProgram loaded into memory and ready to execute.");
                            output.set(msg);
                        } else {
                            success.set(false);
                            error_count.set(result.errors.len());
                            status.set("Error".to_string());

                            let mut msg = format!("✗ {}\n\n", result.message);
                            for (i, error) in result.errors.iter().enumerate() {
                                msg.push_str(&format!("{}. {}\n", i + 1, error));
                            }
                            output.set(msg);
                        }
                    }
                }
                Err(e) => {
                    success.set(false);
                    error_count.set(1);
                    status.set("Error".to_string());
                    output.set(format!("✗ Assembly failed\n\n{:?}", e));
                }
            }
        })
    };

    let on_clear = {
        let code = code.clone();
        let output = output.clone();
        let status = status.clone();
        let error_count = error_count.clone();
        let success = success.clone();

        Callback::from(move |_: MouseEvent| {
            code.set(String::new());
            output.set("Ready to assemble...".to_string());
            status.set("Ready".to_string());
            error_count.set(0);
            success.set(false);
        })
    };

    let status_class = if *success {
        "success"
    } else if *error_count > 0 {
        "error"
    } else {
        ""
    };

    html! {
        <div class="view-panel assembler-view">
            <div class="assembler-editor-section">
                <div class="editor-toolbar">
                    <button class="toolbar-button primary" onclick={on_assemble}>{"Assemble"}</button>
                    <button class="toolbar-button" onclick={on_clear}>{"Clear"}</button>
                    <button class="toolbar-button" disabled={true}>{"Load"}</button>
                    <button class="toolbar-button" disabled={true}>{"Save"}</button>
                    <button class="toolbar-button" disabled={true}>{"Examples ▾"}</button>
                </div>

                <div class="editor-container">
                    <textarea
                        ref={editor_ref}
                        class="assembler-editor"
                        placeholder="Enter IBM 1130 assembly code here..."
                        value={(*code).clone()}
                        oninput={on_code_change}
                    />
                </div>
            </div>

            <div class="assembler-output-section">
                <h3 class="panel-title">{"Assembler Output"}</h3>

                <div class="output-tabs">
                    <button class="output-tab active">{"Messages"}</button>
                    <button class="output-tab" disabled={true}>{"Listing"}</button>
                    <button class="output-tab" disabled={true}>{"Symbol Table"}</button>
                </div>

                <div class="output-container">
                    <div class="output-content">
                        <pre class="output-text">{&*output}</pre>
                    </div>
                </div>

                <div class="assembler-stats">
                    <div class="stat-item">
                        <span class="stat-label">{"Status:"}</span>
                        <span class={classes!("stat-value", status_class)}>{&*status}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Lines:"}</span>
                        <span class="stat-value">{line_count}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Errors:"}</span>
                        <span class={classes!("stat-value", if *error_count > 0 { "error" } else { "" })}>
                            {*error_count}
                        </span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Code Size:"}</span>
                        <span class="stat-value">{"N/A"}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
