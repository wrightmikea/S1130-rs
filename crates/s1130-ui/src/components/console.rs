//! Console component with tabbed interface

use yew::prelude::*;
use crate::components::{Tabs, TabId, ConsolePanel, RegistersView, MemoryView, AssemblerView, IoDevicesView};

#[function_component(Console)]
pub fn console() -> Html {
    let active_tab = use_state(|| TabId::ConsolePanel);

    let on_tab_change = {
        let active_tab = active_tab.clone();
        Callback::from(move |tab_id: TabId| {
            active_tab.set(tab_id);
        })
    };

    // Render the active view based on selected tab
    let active_view = match *active_tab {
        TabId::ConsolePanel => html! { <ConsolePanel /> },
        TabId::Registers => html! { <RegistersView /> },
        TabId::Memory => html! { <MemoryView /> },
        TabId::Assembler => html! { <AssemblerView /> },
        TabId::IoDevices => html! { <IoDevicesView /> },
    };

    html! {
        <main class="app-main">
            <Tabs active_tab={*active_tab} on_tab_change={on_tab_change} />
            <div class="tab-content">
                {active_view}
            </div>
        </main>
    }
}
