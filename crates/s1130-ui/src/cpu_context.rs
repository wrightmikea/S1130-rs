//! CPU Context for sharing emulator state across components

use s1130_wasm::WasmCpu;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

/// Shared CPU state wrapped in Rc<RefCell<>> for interior mutability
#[derive(Clone)]
pub struct CpuContext {
    pub cpu: Rc<RefCell<WasmCpu>>,
    pub version: u32, // Incremented on each CPU state change to trigger re-renders
}

impl CpuContext {
    pub fn new() -> Self {
        Self {
            cpu: Rc::new(RefCell::new(WasmCpu::new())),
            version: 0,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for CpuContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.cpu, &other.cpu) && self.version == other.version
    }
}

/// Context provider component
#[derive(Properties, PartialEq)]
pub struct CpuProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component(CpuProvider)]
pub fn cpu_provider(props: &CpuProviderProps) -> Html {
    let ctx = use_state(CpuContext::new);

    html! {
        <ContextProvider<UseStateHandle<CpuContext>> context={ctx.clone()}>
            { props.children.clone() }
        </ContextProvider<UseStateHandle<CpuContext>>>
    }
}

/// Hook to access CPU context
#[hook]
pub fn use_cpu() -> UseStateHandle<CpuContext> {
    use_context::<UseStateHandle<CpuContext>>()
        .expect("CpuContext not found. Make sure CpuProvider wraps your component tree.")
}
