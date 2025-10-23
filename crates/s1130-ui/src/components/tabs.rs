//! Tabs component for tabbed interface

use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    ConsolePanel,
    Registers,
    Memory,
    Assembler,
    IoDevices,
}

impl TabId {
    pub fn label(&self) -> &'static str {
        match self {
            TabId::ConsolePanel => "Console Panel",
            TabId::Registers => "Registers",
            TabId::Memory => "Memory",
            TabId::Assembler => "Assembler",
            TabId::IoDevices => "I/O Devices",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TabId::ConsolePanel => "⚙",
            TabId::Registers => "📊",
            TabId::Memory => "💾",
            TabId::Assembler => "📝",
            TabId::IoDevices => "🔌",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct TabsProps {
    pub active_tab: TabId,
    pub on_tab_change: Callback<TabId>,
}

#[function_component(Tabs)]
pub fn tabs(props: &TabsProps) -> Html {
    let tabs = [
        TabId::ConsolePanel,
        TabId::Registers,
        TabId::Memory,
        TabId::Assembler,
        TabId::IoDevices,
    ];

    html! {
        <div class="tabs-container">
            <div class="tabs-list">
                {for tabs.iter().map(|tab| {
                    let is_active = *tab == props.active_tab;
                    let tab_id = *tab;
                    let on_click = props.on_tab_change.clone();

                    html! {
                        <button
                            class={classes!("tab-button", is_active.then_some("active"))}
                            onclick={move |_| on_click.emit(tab_id)}
                        >
                            <span class="tab-icon">{tab.icon()}</span>
                            <span class="tab-label">{tab.label()}</span>
                        </button>
                    }
                })}
            </div>
        </div>
    }
}
