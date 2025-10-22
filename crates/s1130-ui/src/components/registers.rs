//! Registers view - shows CPU register state

use yew::prelude::*;

#[function_component(RegistersView)]
pub fn registers_view() -> Html {
    html! {
        <div class="view-panel registers-view">
            <div class="panel-section">
                <h3 class="panel-title">{"CPU Registers"}</h3>
                <div class="registers-grid">
                    <div class="register-item">
                        <div class="register-name">{"IAR"}</div>
                        <div class="register-desc">{"Instruction Address Register"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>

                    <div class="register-item">
                        <div class="register-name">{"ACC"}</div>
                        <div class="register-desc">{"Accumulator"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>

                    <div class="register-item">
                        <div class="register-name">{"EXT"}</div>
                        <div class="register-desc">{"Extension Register"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>

                    <div class="register-item">
                        <div class="register-name">{"XR1"}</div>
                        <div class="register-desc">{"Index Register 1"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>

                    <div class="register-item">
                        <div class="register-name">{"XR2"}</div>
                        <div class="register-desc">{"Index Register 2"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>

                    <div class="register-item">
                        <div class="register-name">{"XR3"}</div>
                        <div class="register-desc">{"Index Register 3"}</div>
                        <div class="register-value">{"0x0000"}</div>
                        <div class="register-binary">{"0000 0000 0000 0000"}</div>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Status Flags"}</h3>
                <div class="flags-grid">
                    <div class="flag-item">
                        <span class="flag-name">{"Carry"}</span>
                        <span class="flag-value off">{"0"}</span>
                    </div>
                    <div class="flag-item">
                        <span class="flag-name">{"Overflow"}</span>
                        <span class="flag-value off">{"0"}</span>
                    </div>
                    <div class="flag-item">
                        <span class="flag-name">{"Wait"}</span>
                        <span class="flag-value off">{"0"}</span>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Execution Info"}</h3>
                <div class="info-grid">
                    <div class="info-row">
                        <span class="info-label">{"Instructions Executed:"}</span>
                        <span class="info-value">{"0"}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"Current Interrupt Level:"}</span>
                        <span class="info-value">{"-"}</span>
                    </div>
                    <div class="info-row">
                        <span class="info-label">{"CPU State:"}</span>
                        <span class="info-value">{"Stopped"}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
