//! Registers view - shows CPU register state

use crate::cpu_context::use_cpu;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Debug, Deserialize)]
struct CpuState {
    iar: u16,
    acc: u16,
    ext: u16,
    xr1: u16,
    xr2: u16,
    xr3: u16,
    carry: bool,
    overflow: bool,
    wait: bool,
    instruction_count: u64,
    current_interrupt_level: Option<u8>,
}

/// Format a u16 value as binary with spaces every 4 bits
fn format_binary(value: u16) -> String {
    format!(
        "{:04b} {:04b} {:04b} {:04b}",
        (value >> 12) & 0xF,
        (value >> 8) & 0xF,
        (value >> 4) & 0xF,
        value & 0xF
    )
}

/// Render a single register
#[derive(Properties, PartialEq)]
struct RegisterItemProps {
    name: AttrValue,
    desc: AttrValue,
    value: u16,
}

#[function_component(RegisterItem)]
fn register_item(props: &RegisterItemProps) -> Html {
    html! {
        <div class="register-item">
            <div class="register-name">{&props.name}</div>
            <div class="register-desc">{&props.desc}</div>
            <div class="register-value">{format!("0x{:04X}", props.value)}</div>
            <div class="register-binary">{format_binary(props.value)}</div>
        </div>
    }
}

#[function_component(RegistersView)]
pub fn registers_view() -> Html {
    let cpu_ctx = use_cpu();

    // Get CPU state
    let state: CpuState = {
        let cpu = cpu_ctx.cpu.borrow();
        let js_state = cpu.get_state();
        serde_wasm_bindgen::from_value(js_state).unwrap_or(CpuState {
            iar: 0,
            acc: 0,
            ext: 0,
            xr1: 0,
            xr2: 0,
            xr3: 0,
            carry: false,
            overflow: false,
            wait: false,
            instruction_count: 0,
            current_interrupt_level: None,
        })
    };

    let cpu_state_text = if state.wait { "Halted" } else { "Ready" };
    let interrupt_text = state
        .current_interrupt_level
        .map(|level| format!("{}", level))
        .unwrap_or_else(|| "-".to_string());

    html! {
        <div class="view-panel registers-view">
            <div class="panel-section">
                <h3 class="panel-title">{"CPU Registers"}</h3>
                <div class="registers-grid">
                    <RegisterItem name="IAR" desc="Instruction Address Register" value={state.iar} />
                    <RegisterItem name="ACC" desc="Accumulator" value={state.acc} />
                    <RegisterItem name="EXT" desc="Extension Register" value={state.ext} />
                    <RegisterItem name="XR1" desc="Index Register 1" value={state.xr1} />
                    <RegisterItem name="XR2" desc="Index Register 2" value={state.xr2} />
                    <RegisterItem name="XR3" desc="Index Register 3" value={state.xr3} />
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Status Flags"}</h3>
                <div class="flags-grid">
                    <div class="flag-item">
                        <span class="flag-name">{"Carry"}</span>
                        <span class={classes!("flag-value", if state.carry { "on" } else { "off" })}>
                            {if state.carry { "1" } else { "0" }}
                        </span>
                    </div>
                    <div class="flag-item">
                        <span class="flag-name">{"Overflow"}</span>
                        <span class={classes!("flag-value", if state.overflow { "on" } else { "off" })}>
                            {if state.overflow { "1" } else { "0" }}
                        </span>
                    </div>
                    <div class="flag-item">
                        <span class="flag-name">{"Wait"}</span>
                        <span class={classes!("flag-value", if state.wait { "on" } else { "off" })}>
                            {if state.wait { "1" } else { "0" }}
                        </span>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Execution Info"}</h3>
                <div class="info-grid">
                    <div class="info-row">
                        <span class="info-label">{"Instructions Executed:"}</span>
                        <span class="info-value">{state.instruction_count}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"Current Interrupt Level:"}</span>
                        <span class="info-value">{interrupt_text}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"CPU State:"}</span>
                        <span class="info-value">{cpu_state_text}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
