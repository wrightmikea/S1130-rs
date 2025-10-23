//! Console Panel view - IBM 1130 hardware buttons, switches, and lights

use crate::cpu_context::use_cpu;
use gloo::console;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize)]
struct CpuState {
    iar: u16,
    acc: u16,
    ext: u16,
    carry: bool,
    overflow: bool,
}

#[function_component(ConsolePanel)]
pub fn console_panel() -> Html {
    let cpu_ctx = use_cpu();

    // Get current CPU state
    let cpu_state = {
        let cpu = cpu_ctx.cpu.borrow();
        let state_js = cpu.get_state();
        serde_wasm_bindgen::from_value::<CpuState>(state_js).unwrap_or(CpuState {
            iar: 0,
            acc: 0,
            ext: 0,
            carry: false,
            overflow: false,
        })
    };

    let on_step = {
        let ctx = cpu_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Console Panel] INST STEP button clicked");
            {
                let mut cpu = ctx.cpu.borrow_mut();
                match cpu.step() {
                    Ok(_) => {
                        console::log!("[Console Panel] Step executed successfully");
                    }
                    Err(e) => {
                        console::log!(format!("[Console Panel] Step error: {:?}", e));
                    }
                }
            }
            // Trigger re-render by incrementing version
            let mut new_ctx = (*ctx).clone();
            new_ctx.version += 1;
            ctx.set(new_ctx);
        })
    };

    let on_start = {
        let ctx = cpu_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Console Panel] START button clicked");
            {
                let mut cpu = ctx.cpu.borrow_mut();
                match cpu.run(100) {
                    Ok(_) => {
                        console::log!("[Console Panel] Run completed successfully");
                    }
                    Err(e) => {
                        console::log!(format!("[Console Panel] Run error: {:?}", e));
                    }
                }
            }
            // Trigger re-render by incrementing version
            let mut new_ctx = (*ctx).clone();
            new_ctx.version += 1;
            ctx.set(new_ctx);
        })
    };

    let on_reset = {
        let ctx = cpu_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            console::log!("[Console Panel] RESET button clicked");
            {
                let mut cpu = ctx.cpu.borrow_mut();
                cpu.reset();
            }
            console::log!("[Console Panel] CPU reset");
            // Trigger re-render by incrementing version
            let mut new_ctx = (*ctx).clone();
            new_ctx.version += 1;
            ctx.set(new_ctx);
        })
    };

    html! {
        <div class="view-panel console-panel">
            <div class="panel-section">
                <h3 class="panel-title">{"Console Lights"}</h3>
                <div class="lights-grid">
                    <div class="light-group">
                        <span class="light-label">{"Power"}</span>
                        <div class="indicator-light on"></div>
                    </div>
                    <div class="light-group">
                        <span class="light-label">{"Run"}</span>
                        <div class="indicator-light"></div>
                    </div>
                    <div class="light-group">
                        <span class="light-label">{"Wait"}</span>
                        <div class="indicator-light"></div>
                    </div>
                    <div class="light-group">
                        <span class="light-label">{"Carry"}</span>
                        <div class={classes!("indicator-light", cpu_state.carry.then_some("on"))}></div>
                    </div>
                    <div class="light-group">
                        <span class="light-label">{"Overflow"}</span>
                        <div class={classes!("indicator-light", cpu_state.overflow.then_some("on"))}></div>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Control Switches"}</h3>
                <div class="switches-grid">
                    <div class="switch-group">
                        <label class="switch-label">{"Program Start"}</label>
                        <button class="panel-button" onclick={on_start}>{"START"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Program Stop"}</label>
                        <button class="panel-button" disabled=true>{"STOP"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Instruction Step"}</label>
                        <button class="panel-button" onclick={on_step}>{"INST STEP"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Reset"}</label>
                        <button class="panel-button reset" onclick={on_reset}>{"RESET"}</button>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Address Entry Switches"}</h3>
                <div class="address-switches">
                    {for (0..16).map(|bit| {
                        html! {
                            <div class="bit-switch">
                                <label class="bit-label">{format!("{}", 15 - bit)}</label>
                                <input
                                    type="checkbox"
                                    class="toggle-switch"
                                />
                            </div>
                        }
                    })}
                </div>
                <div class="switch-group">
                    <button class="panel-button secondary" disabled=true>{"Load IAR"}</button>
                    <button class="panel-button secondary" disabled=true>{"Deposit"}</button>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Console Display"}</h3>
                <div class="console-display">
                    <div class="display-row">
                        <span class="display-label">{"IAR:"}</span>
                        <span class="display-value">{format!("0x{:04X}", cpu_state.iar)}</span>
                    </div>
                    <div class="display-row">
                        <span class="display-label">{"ACC:"}</span>
                        <span class="display-value">{format!("0x{:04X}", cpu_state.acc)}</span>
                    </div>
                    <div class="display-row">
                        <span class="display-label">{"EXT:"}</span>
                        <span class="display-value">{format!("0x{:04X}", cpu_state.ext)}</span>
                    </div>
                </div>
            </div>

            <div class="panel-note">
                <p>{"INST STEP, START, and RESET buttons now functional!"}</p>
            </div>
        </div>
    }
}
