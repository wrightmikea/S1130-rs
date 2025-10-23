//! Memory view - shows memory contents in machine code format

use crate::cpu_context::use_cpu;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum DisplayFormat {
    Hexadecimal,
    Binary,
    Decimal,
    Octal,
}

impl DisplayFormat {
    fn format(&self, value: u16) -> String {
        match self {
            DisplayFormat::Hexadecimal => format!("{:04X}", value),
            DisplayFormat::Binary => format!("{:016b}", value),
            DisplayFormat::Decimal => format!("{:05}", value),
            DisplayFormat::Octal => format!("{:06o}", value),
        }
    }
}

#[function_component(MemoryView)]
pub fn memory_view() -> Html {
    let cpu_ctx = use_cpu();
    let base_address = use_state(|| 0u16);
    let format = use_state(|| DisplayFormat::Hexadecimal);
    let address_input_ref = use_node_ref();

    // Read memory from CPU
    let memory_lines: Vec<(u16, Vec<u16>)> = {
        let cpu = cpu_ctx.cpu.borrow();
        (0..16)
            .map(|i| {
                let addr = (*base_address + i * 8) & 0x7FFF; // Wrap at 32K
                let data = cpu.read_memory_range(addr, 8);
                (addr, data)
            })
            .collect()
    };

    // Count non-zero words for used memory statistic
    let used_memory: usize = {
        let cpu = cpu_ctx.cpu.borrow();
        (0..0x8000u16)
            .filter(|&addr| cpu.read_memory(addr).unwrap_or(0) != 0)
            .count()
    };

    let on_address_change = {
        let base_address = base_address.clone();
        let address_input_ref = address_input_ref.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(input) = address_input_ref.cast::<HtmlInputElement>() {
                let value_str = input.value().trim().to_lowercase();
                // Parse hex address (with or without 0x prefix)
                let addr_str = value_str.strip_prefix("0x").unwrap_or(&value_str);
                if let Ok(addr) = u16::from_str_radix(addr_str, 16) {
                    base_address.set(addr & 0x7FF8); // Align to 8-word boundary
                }
            }
        })
    };

    let on_format_change = {
        let format = format.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
            if let Some(select) = target {
                let new_format = match select.value().as_str() {
                    "Binary" => DisplayFormat::Binary,
                    "Decimal" => DisplayFormat::Decimal,
                    "Octal" => DisplayFormat::Octal,
                    _ => DisplayFormat::Hexadecimal,
                };
                format.set(new_format);
            }
        })
    };

    html! {
        <div class="view-panel memory-view">
            <div class="panel-section">
                <h3 class="panel-title">{"Memory Contents"}</h3>

                <div class="memory-controls">
                    <div class="control-group">
                        <label>{"Address:"}</label>
                        <input
                            ref={address_input_ref.clone()}
                            type="text"
                            class="address-input"
                            placeholder="0x0000"
                        />
                        <button class="memory-button" onclick={on_address_change}>{"Go"}</button>
                    </div>
                    <div class="control-group">
                        <label>{"Display Format:"}</label>
                        <select class="format-select" onchange={on_format_change}>
                            <option selected={*format == DisplayFormat::Hexadecimal}>{"Hexadecimal"}</option>
                            <option selected={*format == DisplayFormat::Binary}>{"Binary"}</option>
                            <option selected={*format == DisplayFormat::Decimal}>{"Decimal"}</option>
                            <option selected={*format == DisplayFormat::Octal}>{"Octal"}</option>
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
                                                <td class="data-cell">{format.format(*word)}</td>
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
                        <span class="info-value">{format!("{} words", used_memory)}</span>
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
