//! Main application component with layout

use yew::prelude::*;
use crate::components::{Header, Footer, Sidebar, Console};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app-layout">
            <Header />
            <div class="app-body">
                <Sidebar />
                <Console />
            </div>
            <Footer />
        </div>
    }
}
