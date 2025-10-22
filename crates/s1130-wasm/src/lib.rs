//! WASM bindings for S1130 emulator
//!
//! This crate provides WebAssembly bindings for the s1130-core library,
//! allowing the emulator to run in web browsers.

use s1130_core::Cpu;
use wasm_bindgen::prelude::*;

/// WASM wrapper for CPU
#[wasm_bindgen]
pub struct WasmCpu {
    inner: Cpu,
}

#[wasm_bindgen]
impl WasmCpu {
    /// Create a new CPU instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages in browser
        console_error_panic_hook::set_once();

        Self { inner: Cpu::new() }
    }

    /// Reset CPU to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get current CPU state as JSON
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> JsValue {
        let state = self.inner.get_state();
        serde_wasm_bindgen::to_value(&state).unwrap()
    }

    /// Read memory at address
    #[wasm_bindgen(js_name = readMemory)]
    pub fn read_memory(&self, address: u16) -> Result<u16, JsValue> {
        self.inner
            .read_memory(address as usize)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Write value to memory at address
    #[wasm_bindgen(js_name = writeMemory)]
    pub fn write_memory(&mut self, address: u16, value: u16) -> Result<(), JsValue> {
        self.inner
            .write_memory(address as usize, value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Read a range of memory
    #[wasm_bindgen(js_name = readMemoryRange)]
    pub fn read_memory_range(&self, address: u16, count: u16) -> Vec<u16> {
        (address..address.saturating_add(count))
            .filter_map(|addr| self.inner.read_memory(addr as usize).ok())
            .collect()
    }
}

impl Default for WasmCpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s1130_core::CpuState;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_wasm_cpu_creation() {
        let cpu = WasmCpu::new();
        let state: CpuState = serde_wasm_bindgen::from_value(cpu.get_state()).unwrap();
        assert_eq!(state.acc, 0);
    }

    #[wasm_bindgen_test]
    fn test_wasm_memory_operations() {
        let mut cpu = WasmCpu::new();
        cpu.write_memory(0x100, 0x1234).unwrap();
        assert_eq!(cpu.read_memory(0x100).unwrap(), 0x1234);
    }
}
