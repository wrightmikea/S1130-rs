//! Main application component with layout

use crate::components::{Console, Footer, Header, Sidebar};
use crate::cpu_context::CpuProvider;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <CpuProvider>
            <div class="app-layout">
                <Header />
                <div class="app-body">
                    <Sidebar />
                    <Console />
                </div>
                <Footer />
            </div>
        </CpuProvider>
    }
}
