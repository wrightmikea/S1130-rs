//! Memory view - shows memory contents in machine code format

use yew::prelude::*;

#[function_component(MemoryView)]
pub fn memory_view() -> Html {
    // Sample memory data (will be replaced with actual CPU memory later)
    let memory_lines = (0..16).map(|i| {
        let addr = i * 8;
        (addr, vec![0u16; 8])
    }).collect::<Vec<_>>();

    html! {
        <div class="view-panel memory-view">
            <div class="panel-section">
                <h3 class="panel-title">{"Memory Contents"}</h3>

                <div class="memory-controls">
                    <div class="control-group">
                        <label>{"Address:"}</label>
                        <input
                            type="text"
                            class="address-input"
                            placeholder="0x0000"
                        />
                        <button class="memory-button">{"Go"}</button>
                    </div>
                    <div class="control-group">
                        <label>{"Display Format:"}</label>
                        <select class="format-select">
                            <option>{"Hexadecimal"}</option>
                            <option>{"Binary"}</option>
                            <option>{"Decimal"}</option>
                            <option>{"Octal"}</option>
                        </select>
                    </div>
                </div>

                <div class="memory-table-container">
                    <table class="memory-table">
                        <thead>
                            <tr>
                                <th class="addr-col">{"Address"}</th>
                                {for (0..8).map(|i| html! {
                                    <th class="data-col">{format!("+{}", i)}</th>
                                })}
                            </tr>
                        </thead>
                        <tbody>
                            {for memory_lines.iter().map(|(addr, data)| {
                                html! {
                                    <tr class="memory-row">
                                        <td class="addr-cell">{format!("0x{:04X}", addr)}</td>
                                        {for data.iter().map(|word| {
                                            html! {
                                                <td class="data-cell">{format!("{:04X}", word)}</td>
                                            }
                                        })}
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Memory Statistics"}</h3>
                <div class="info-grid">
                    <div class="info-row">
                        <span class="info-label">{"Total Memory:"}</span>
                        <span class="info-value">{"32K words (64KB)"}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"Used Memory:"}</span>
                        <span class="info-value">{"0 words"}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"Memory Protection:"}</span>
                        <span class="info-value">{"Enabled"}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
