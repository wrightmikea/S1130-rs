//! I/O Devices view - shows status of all attached devices

use yew::prelude::*;

#[function_component(IoDevicesView)]
pub fn io_devices_view() -> Html {
    html! {
        <div class="view-panel io-devices-view">
            <div class="panel-section">
                <h3 class="panel-title">{"Console Devices"}</h3>
                <div class="devices-grid">
                    <div class="device-card">
                        <div class="device-header">
                            <span class="device-icon">{"⌨️"}</span>
                            <span class="device-name">{"Console Keyboard"}</span>
                            <span class="device-status ready">{"Ready"}</span>
                        </div>
                        <div class="device-info">
                            <div class="info-row">
                                <span>{"Device Code:"}</span>
                                <span class="mono">{"0x01"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Type:"}</span>
                                <span>{"Character-mode"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Buffer:"}</span>
                                <span>{"Empty"}</span>
                            </div>
                        </div>
                        <div class="device-actions">
                            <button class="device-button">{"Type..."}</button>
                        </div>
                    </div>

                    <div class="device-card">
                        <div class="device-header">
                            <span class="device-icon">{"🖨️"}</span>
                            <span class="device-name">{"Console Printer"}</span>
                            <span class="device-status ready">{"Ready"}</span>
                        </div>
                        <div class="device-info">
                            <div class="info-row">
                                <span>{"Device Code:"}</span>
                                <span class="mono">{"0x02"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Type:"}</span>
                                <span>{"Character-mode"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Output:"}</span>
                                <span>{"0 chars"}</span>
                            </div>
                        </div>
                        <div class="device-actions">
                            <button class="device-button">{"View Output"}</button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Card Equipment"}</h3>
                <div class="devices-grid">
                    <div class="device-card">
                        <div class="device-header">
                            <span class="device-icon">{"📇"}</span>
                            <span class="device-name">{"2501 Card Reader"}</span>
                            <span class="device-status not-ready">{"Not Ready"}</span>
                        </div>
                        <div class="device-info">
                            <div class="info-row">
                                <span>{"Device Code:"}</span>
                                <span class="mono">{"0x09"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Type:"}</span>
                                <span>{"Block-mode"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Hopper:"}</span>
                                <span>{"Empty"}</span>
                            </div>
                        </div>
                        <div class="device-actions">
                            <button class="device-button">{"Load Cards..."}</button>
                        </div>
                    </div>

                    <div class="device-card disabled">
                        <div class="device-header">
                            <span class="device-icon">{"🎴"}</span>
                            <span class="device-name">{"1442 Card Punch"}</span>
                            <span class="device-status disabled">{"Not Installed"}</span>
                        </div>
                        <div class="device-info">
                            <div class="info-row">
                                <span>{"Device Code:"}</span>
                                <span class="mono">{"0x03"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Type:"}</span>
                                <span>{"Character-mode"}</span>
                            </div>
                        </div>
                        <div class="device-actions">
                            <button class="device-button" disabled=true>{"Install"}</button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="panel-section">
                <h3 class="panel-title">{"Disk Storage"}</h3>
                <div class="devices-grid">
                    <div class="device-card disabled">
                        <div class="device-header">
                            <span class="device-icon">{"💿"}</span>
                            <span class="device-name">{"2310 Disk Drive"}</span>
                            <span class="device-status disabled">{"Not Installed"}</span>
                        </div>
                        <div class="device-info">
                            <div class="info-row">
                                <span>{"Device Code:"}</span>
                                <span class="mono">{"TBD"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Type:"}</span>
                                <span>{"Block-mode"}</span>
                            </div>
                            <div class="info-row">
                                <span>{"Capacity:"}</span>
                                <span>{"1.5MB"}</span>
                            </div>
                        </div>
                        <div class="device-actions">
                            <button class="device-button" disabled=true>{"Install"}</button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="panel-note">
                <p>{"Device integration and control coming in future phases"}</p>
            </div>
        </div>
    }
}
