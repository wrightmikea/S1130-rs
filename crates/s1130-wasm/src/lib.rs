//! WASM bindings for S1130 emulator
//!
//! This crate provides WebAssembly bindings for the s1130-core library,
//! allowing the emulator to run in web browsers.

use s1130_core::Cpu;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Result of assembly operation
#[derive(Serialize)]
struct AssemblyResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    origin: Option<u16>,
    #[serde(rename = "entryPoint", skip_serializing_if = "Option::is_none")]
    entry_point: Option<u16>,
    #[serde(rename = "codeSize", skip_serializing_if = "Option::is_none")]
    code_size: Option<usize>,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
}

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

    /// Assemble source code and load into memory
    #[wasm_bindgen]
    pub fn assemble(&mut self, source: &str) -> Result<JsValue, JsValue> {
        web_sys::console::log_1(&"[WASM] assemble() called".into());
        use s1130_core::assembler::Assembler;

        let mut assembler = Assembler::new();
        web_sys::console::log_1(&"[WASM] Assembler created, calling assemble()".into());
        match assembler.assemble(source) {
            Ok(program) => {
                web_sys::console::log_1(
                    &format!(
                        "[WASM] Assembly successful, loading {} words",
                        program.words.len()
                    )
                    .into(),
                );
                // Load program into memory starting at origin
                for (i, word) in program.words.iter().enumerate() {
                    let addr = program.origin as usize + i;
                    if let Err(e) = self.inner.write_memory(addr, *word) {
                        return Err(JsValue::from_str(&format!("Memory write error: {}", e)));
                    }
                }

                // Set IAR to entry point, or origin if not specified
                let iar_address = program.entry_point.unwrap_or(program.origin);
                self.inner.set_iar(iar_address);
                web_sys::console::log_1(
                    &format!(
                        "[WASM] Set IAR to 0x{:04X} ({})",
                        iar_address,
                        if program.entry_point.is_some() {
                            "entry point"
                        } else {
                            "origin"
                        }
                    )
                    .into(),
                );

                // Return assembly result
                let result = AssemblyResult {
                    success: true,
                    origin: Some(program.origin),
                    entry_point: program.entry_point,
                    code_size: Some(program.words.len()),
                    message: "Assembly successful".to_string(),
                    errors: vec![],
                };
                Ok(serde_wasm_bindgen::to_value(&result).unwrap())
            }
            Err(error) => {
                web_sys::console::log_1(&format!("[WASM] Assembly failed: {}", error).into());
                let result = AssemblyResult {
                    success: false,
                    origin: None,
                    entry_point: None,
                    code_size: None,
                    message: "Assembly failed".to_string(),
                    errors: vec![error.to_string()],
                };
                Ok(serde_wasm_bindgen::to_value(&result).unwrap())
            }
        }
    }

    /// Execute one instruction (step)
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.step() {
            Ok(_) => {
                let state = self.inner.get_state();
                Ok(serde_wasm_bindgen::to_value(&state).unwrap())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Run N instructions
    #[wasm_bindgen]
    pub fn run(&mut self, steps: u32) -> Result<JsValue, JsValue> {
        for _ in 0..steps {
            if let Err(e) = self.inner.step() {
                return Err(JsValue::from_str(&e.to_string()));
            }
        }
        let state = self.inner.get_state();
        Ok(serde_wasm_bindgen::to_value(&state).unwrap())
    }

    /// Get CPU registers as formatted strings
    #[wasm_bindgen(js_name = getRegisters)]
    pub fn get_registers(&self) -> JsValue {
        let state = self.inner.get_state();
        let registers = serde_json::json!({
            "iar": format!("0x{:04X}", state.iar),
            "acc": format!("0x{:04X}", state.acc),
            "ext": format!("0x{:04X}", state.ext),
            "xr1": format!("0x{:04X}", state.xr1),
            "xr2": format!("0x{:04X}", state.xr2),
            "xr3": format!("0x{:04X}", state.xr3),
            "carry": state.carry,
            "overflow": state.overflow,
            "instructionCount": state.instruction_count
        });
        serde_wasm_bindgen::to_value(&registers).unwrap()
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
