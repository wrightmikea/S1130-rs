//! Assembler view - editor and output

use yew::prelude::*;

#[function_component(AssemblerView)]
pub fn assembler_view() -> Html {
    let sample_code = "*\n* Sample IBM 1130 Assembly Program\n*\n    ORG  /100\nSTART LDX 1 COUNT\n    LD  A+1\n    A   A\n    STO A+1\n    MDX 1 -1\n    BNZ START\n    WAIT\n\nA    DC  1\nCOUNT DC  10\n    END START\n";

    html! {
        <div class="view-panel assembler-view">
            <div class="assembler-editor-section">
                <div class="editor-toolbar">
                    <button class="toolbar-button primary">{"Assemble"}</button>
                    <button class="toolbar-button">{"Load"}</button>
                    <button class="toolbar-button">{"Save"}</button>
                    <button class="toolbar-button">{"Clear"}</button>
                    <button class="toolbar-button">{"Examples ▾"}</button>
                </div>

                <div class="editor-container">
                    <textarea
                        class="assembler-editor"
                        placeholder="Enter IBM 1130 assembly code here..."
                        value={sample_code}
                    />
                </div>
            </div>

            <div class="assembler-output-section">
                <h3 class="panel-title">{"Assembler Output"}</h3>

                <div class="output-tabs">
                    <button class="output-tab active">{"Messages"}</button>
                    <button class="output-tab">{"Listing"}</button>
                    <button class="output-tab">{"Symbol Table"}</button>
                </div>

                <div class="output-container">
                    <div class="output-content">
                        <div class="output-line">{"Ready to assemble..."}</div>
                        <div class="output-line muted">{"Click 'Assemble' to compile your code"}</div>
                    </div>
                </div>

                <div class="assembler-stats">
                    <div class="stat-item">
                        <span class="stat-label">{"Status:"}</span>
                        <span class="stat-value">{"Ready"}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Lines:"}</span>
                        <span class="stat-value">{"0"}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Errors:"}</span>
                        <span class="stat-value">{"0"}</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{"Warnings:"}</span>
                        <span class="stat-value">{"0"}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
