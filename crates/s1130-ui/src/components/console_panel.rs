//! Console Panel view - IBM 1130 hardware buttons, switches, and lights

use yew::prelude::*;

#[function_component(ConsolePanel)]
pub fn console_panel() -> Html {
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
                        <div class="indicator-light"></div>
                    </div>
                    <div class="light-group">
                        <span class="light-label">{"Overflow"}</span>
                        <div class="indicator-light"></div>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Control Switches"}</h3>
                <div class="switches-grid">
                    <div class="switch-group">
                        <label class="switch-label">{"Program Start"}</label>
                        <button class="panel-button">{"START"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Program Stop"}</label>
                        <button class="panel-button">{"STOP"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Instruction Step"}</label>
                        <button class="panel-button">{"INST STEP"}</button>
                    </div>
                    <div class="switch-group">
                        <label class="switch-label">{"Reset"}</label>
                        <button class="panel-button reset">{"RESET"}</button>
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
                    <button class="panel-button secondary">{"Load IAR"}</button>
                    <button class="panel-button secondary">{"Deposit"}</button>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Console Display"}</h3>
                <div class="console-display">
                    <div class="display-row">
                        <span class="display-label">{"IAR:"}</span>
                        <span class="display-value">{"0x0000"}</span>
                    </div>
                    <div class="display-row">
                        <span class="display-label">{"ACC:"}</span>
                        <span class="display-value">{"0x0000"}</span>
                    </div>
                    <div class="display-row">
                        <span class="display-label">{"EXT:"}</span>
                        <span class="display-value">{"0x0000"}</span>
                    </div>
                </div>
            </div>

            <div class="panel-note">
                <p>{"Note: Full hardware control integration coming in future phase"}</p>
            </div>
        </div>
    }
}
