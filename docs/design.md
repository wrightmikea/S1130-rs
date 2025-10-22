# S1130 Rust + Yew Detailed Design

## Table of Contents

1. [Core Emulator Design](#core-emulator-design)
2. [Assembler Design](#assembler-design)
3. [Device System Design](#device-system-design)
4. [WASM Bindings Design](#wasm-bindings-design)
5. [UI Component Design](#ui-component-design)
6. [State Management Design](#state-management-design)
7. [Testing Design](#testing-design)
8. [Performance Design](#performance-design)

## Core Emulator Design

### CPU Structure

```rust
// src/cpu/mod.rs

/// Core CPU structure representing the IBM 1130 processor
pub struct Cpu {
    /// Main accumulator (16-bit)
    acc: u16,

    /// Extension register for 32-bit operations (16-bit)
    ext: u16,

    /// Instruction Address Register (program counter)
    iar: u16,

    /// Index registers XR1, XR2, XR3
    index_registers: IndexRegisters,

    /// Status flags
    carry: bool,
    overflow: bool,
    wait: bool,

    /// Main memory (32K words = 65536 bytes)
    memory: Vec<u16>,

    /// Attached devices
    devices: HashMap<u8, Box<dyn Device>>,

    /// Interrupt system
    interrupt_system: InterruptSystem,

    /// Instruction set
    instructions: InstructionSet,

    /// Current instruction decode state
    current_instruction: Option<InstructionInfo>,

    /// Performance counter
    instruction_count: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            acc: 0,
            ext: 0,
            iar: 0,
            index_registers: IndexRegisters::new(),
            carry: false,
            overflow: false,
            wait: false,
            memory: vec![0; 32768],
            devices: HashMap::new(),
            interrupt_system: InterruptSystem::new(),
            instructions: InstructionSet::new(),
            current_instruction: None,
            instruction_count: 0,
        }
    }

    /// Fetch next instruction from memory
    pub fn next_instruction(&mut self) -> Result<(), CpuError> {
        let first_word = self.read_memory(self.iar as usize)?;
        self.iar = self.iar.wrapping_add(1);

        let opcode = OpCode::from_u8((first_word >> 11) as u8)?;
        let format_bit = (first_word & 0x0400) != 0;
        let tag = ((first_word & 0x0300) >> 8) as u8;
        let modifiers = (first_word & 0x00FF) as u8;

        let instruction = self.instructions.get(opcode)?;

        let (format, displacement, indirect) = if format_bit && instruction.has_long_format() {
            let second_word = self.read_memory(self.iar as usize)?;
            self.iar = self.iar.wrapping_add(1);
            let indirect = (first_word & 0x0080) != 0;
            (InstructionFormat::Long, second_word, indirect)
        } else {
            (InstructionFormat::Short, modifiers as u16, false)
        };

        self.current_instruction = Some(InstructionInfo {
            opcode,
            format,
            tag,
            displacement,
            indirect,
            modifiers,
        });

        Ok(())
    }

    /// Execute current instruction
    pub fn execute_instruction(&mut self) -> Result<(), CpuError> {
        if let Some(ref info) = self.current_instruction.clone() {
            let instruction = self.instructions.get(info.opcode)?;
            instruction.execute(self, info)?;
            self.interrupt_system.handle_interrupts(self)?;
            self.instruction_count += 1;
            Ok(())
        } else {
            Err(CpuError::NoInstructionLoaded)
        }
    }

    /// Single step (fetch + execute)
    pub fn step(&mut self) -> Result<(), CpuError> {
        if self.wait {
            return Err(CpuError::WaitState);
        }
        self.next_instruction()?;
        self.execute_instruction()?;
        Ok(())
    }

    /// Read from memory with bounds checking
    pub fn read_memory(&self, address: usize) -> Result<u16, CpuError> {
        self.memory.get(address)
            .copied()
            .ok_or(CpuError::MemoryViolation(address as u16))
    }

    /// Write to memory with bounds checking
    pub fn write_memory(&mut self, address: usize, value: u16) -> Result<(), CpuError> {
        if address < self.memory.len() {
            self.memory[address] = value;

            // Handle memory-mapped index registers
            match address {
                0x0001 => self.index_registers.xr1 = value,
                0x0002 => self.index_registers.xr2 = value,
                0x0003 => self.index_registers.xr3 = value,
                _ => {}
            }

            Ok(())
        } else {
            Err(CpuError::MemoryViolation(address as u16))
        }
    }

    /// Get current CPU state (for UI display)
    pub fn get_state(&self) -> CpuState {
        CpuState {
            acc: self.acc,
            ext: self.ext,
            iar: self.iar,
            xr1: self.index_registers.xr1,
            xr2: self.index_registers.xr2,
            xr3: self.index_registers.xr3,
            carry: self.carry,
            overflow: self.overflow,
            wait: self.wait,
            instruction_count: self.instruction_count,
            current_interrupt_level: self.interrupt_system.current_level(),
        }
    }
}
```

### Register Structures

```rust
// src/cpu/registers.rs

/// Index registers XR1, XR2, XR3
#[derive(Debug, Clone, Copy)]
pub struct IndexRegisters {
    pub xr1: u16,
    pub xr2: u16,
    pub xr3: u16,
}

impl IndexRegisters {
    pub fn new() -> Self {
        Self {
            xr1: 0,
            xr2: 0,
            xr3: 0,
        }
    }

    /// Get index register by tag (1-3)
    pub fn get(&self, tag: u8) -> u16 {
        match tag {
            1 => self.xr1,
            2 => self.xr2,
            3 => self.xr3,
            _ => 0,  // Tag 0 = no index
        }
    }

    /// Set index register by tag (1-3)
    pub fn set(&mut self, tag: u8, value: u16) {
        match tag {
            1 => self.xr1 = value,
            2 => self.xr2 = value,
            3 => self.xr3 = value,
            _ => {}
        }
    }
}

/// CPU state snapshot for external observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub acc: u16,
    pub ext: u16,
    pub iar: u16,
    pub xr1: u16,
    pub xr2: u16,
    pub xr3: u16,
    pub carry: bool,
    pub overflow: bool,
    pub wait: bool,
    pub instruction_count: u64,
    pub current_interrupt_level: Option<u8>,
}
```

### Instruction System

```rust
// src/instructions/mod.rs

/// Instruction trait - all instructions implement this
pub trait Instruction: Send + Sync {
    fn opcode(&self) -> OpCode;
    fn name(&self) -> &'static str;
    fn has_long_format(&self) -> bool;
    fn execute(&self, cpu: &mut Cpu, info: &InstructionInfo) -> Result<(), InstructionError>;
}

/// Decoded instruction information
#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub opcode: OpCode,
    pub format: InstructionFormat,
    pub tag: u8,
    pub displacement: u16,
    pub indirect: bool,
    pub modifiers: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    Short,
    Long,
}

/// Opcode enumeration (28 opcodes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    Load = 0x18,
    Store = 0x1A,
    Add = 0x10,
    Subtract = 0x14,
    Multiply = 0x1C,
    Divide = 0x1D,
    And = 0x12,
    Or = 0x13,
    ExclusiveOr = 0x11,
    BranchSkip = 0x09,
    BranchStoreIar = 0x08,
    ModifyIndex = 0x05,
    ExecuteIO = 0x01,
    Wait = 0x16,
    // ... remaining opcodes
}

impl OpCode {
    pub fn from_u8(value: u8) -> Result<Self, InstructionError> {
        match value {
            0x18 => Ok(OpCode::Load),
            0x1A => Ok(OpCode::Store),
            // ... all opcodes
            _ => Err(InstructionError::InvalidOpcode(value))
        }
    }
}

/// Base functionality for address calculation
pub trait InstructionBase {
    /// Calculate effective address considering format, index, indirect
    fn get_effective_address(&self, cpu: &Cpu, info: &InstructionInfo) -> u16 {
        let base_address = match info.format {
            InstructionFormat::Long => info.displacement,
            InstructionFormat::Short => {
                // Sign-extend 8-bit displacement
                let signed_disp = if info.displacement & 0x80 != 0 {
                    (info.displacement as i16) | 0xFF00
                } else {
                    info.displacement as i16
                };
                cpu.iar.wrapping_add_signed(signed_disp)
            }
        };

        // Apply index register
        let indexed_address = if info.tag > 0 {
            base_address.wrapping_add(cpu.index_registers.get(info.tag))
        } else {
            base_address
        };

        // Apply indirect addressing
        if info.indirect {
            cpu.read_memory(indexed_address as usize).unwrap_or(0)
        } else {
            indexed_address
        }
    }
}
```

### Instruction Implementation Example

```rust
// src/instructions/load.rs

use super::*;

pub struct Load;

impl Instruction for Load {
    fn opcode(&self) -> OpCode {
        OpCode::Load
    }

    fn name(&self) -> &'static str {
        "LD"
    }

    fn has_long_format(&self) -> bool {
        true
    }

    fn execute(&self, cpu: &mut Cpu, info: &InstructionInfo) -> Result<(), InstructionError> {
        let address = self.get_effective_address(cpu, info);
        let value = cpu.read_memory(address as usize)?;
        cpu.acc = value;
        Ok(())
    }
}

impl InstructionBase for Load {}
```

## Assembler Design

### Assembler Structure

```rust
// src/assembler/mod.rs

pub struct Assembler {
    source_lines: Vec<String>,
    symbols: HashMap<String, u16>,
    errors: Vec<AssemblyError>,
    location_counter: u16,
    current_line: usize,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            source_lines: Vec::new(),
            symbols: HashMap::new(),
            errors: Vec::new(),
            location_counter: 0,
            current_line: 0,
        }
    }

    /// Main assembly entry point
    pub fn assemble(&mut self, source: &str, cpu: &mut Cpu) -> AssemblyResult {
        self.source_lines = source.lines().map(String::from).collect();
        self.symbols.clear();
        self.errors.clear();

        // Pass 1: Build symbol table
        self.pass1();

        if self.errors.is_empty() {
            // Pass 2: Generate code
            self.pass2(cpu);
        }

        AssemblyResult {
            success: self.errors.is_empty(),
            errors: self.errors.clone(),
            symbols: self.symbols.clone(),
            listing: self.generate_listing(),
        }
    }

    /// Pass 1: Symbol collection
    fn pass1(&mut self) {
        self.location_counter = 0;

        for (line_num, line) in self.source_lines.iter().enumerate() {
            self.current_line = line_num + 1;

            let parsed = self.parse_line(line);
            if let Some(label) = parsed.label {
                if self.symbols.contains_key(&label) {
                    self.add_error(format!("Duplicate label: {}", label));
                } else {
                    self.symbols.insert(label, self.location_counter);
                }
            }

            match parsed.operation.as_deref() {
                Some("ORG") => {
                    if let Some(addr) = self.parse_address(&parsed.operand) {
                        self.location_counter = addr;
                    }
                }
                Some("DC") => {
                    self.location_counter += 1;
                }
                Some("BSS") => {
                    if let Some(count) = self.parse_count(&parsed.operand) {
                        self.location_counter += count;
                    }
                }
                Some("EQU") => {
                    if let Some(label) = parsed.label {
                        if let Some(value) = self.evaluate_expression(&parsed.operand) {
                            self.symbols.insert(label, value);
                        }
                    }
                }
                Some(op) if self.is_instruction(op) => {
                    let is_long = parsed.operand.contains("L");
                    self.location_counter += if is_long { 2 } else { 1 };
                }
                _ => {}
            }
        }
    }

    /// Pass 2: Code generation
    fn pass2(&mut self, cpu: &mut Cpu) {
        self.location_counter = 0;

        for (line_num, line) in self.source_lines.iter().enumerate() {
            self.current_line = line_num + 1;

            let parsed = self.parse_line(line);

            match parsed.operation.as_deref() {
                Some("ORG") => {
                    if let Some(addr) = self.parse_address(&parsed.operand) {
                        self.location_counter = addr;
                    }
                }
                Some("DC") => {
                    if let Some(value) = self.evaluate_expression(&parsed.operand) {
                        cpu.write_memory(self.location_counter as usize, value).ok();
                        self.location_counter += 1;
                    }
                }
                Some("BSS") => {
                    if let Some(count) = self.parse_count(&parsed.operand) {
                        self.location_counter += count;
                    }
                }
                Some(op) if self.is_instruction(op) => {
                    self.generate_instruction(cpu, op, &parsed);
                }
                _ => {}
            }
        }
    }
}

/// Parsed assembly line
struct ParsedLine {
    label: Option<String>,
    operation: Option<String>,
    operand: String,
}

/// Assembly result returned to caller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyResult {
    pub success: bool,
    pub errors: Vec<AssemblyError>,
    pub symbols: HashMap<String, u16>,
    pub listing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyError {
    pub line: usize,
    pub message: String,
}
```

### Lexer

```rust
// src/assembler/lexer.rs

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let ch = self.peek()?;

        match ch {
            '/' => Some(self.read_hex()),
            '0'..='9' => Some(self.read_number()),
            'A'..='Z' | 'a'..='z' | '_' => Some(self.read_identifier()),
            ',' => {
                self.advance();
                Some(Token::Comma)
            }
            '*' => {
                self.advance();
                Some(Token::Star)
            }
            '+' | '-' => {
                let op = self.advance().unwrap();
                Some(Token::Operator(op))
            }
            _ => {
                self.advance();
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(u16),
    HexNumber(u16),
    Comma,
    Star,
    Operator(char),
}
```

## Device System Design

### Device Trait

```rust
// src/devices/mod.rs

pub trait Device: Send + Sync {
    fn device_code(&self) -> u8;
    fn name(&self) -> &'static str;

    /// Execute IOCC command
    fn execute_iocc(&mut self, cpu: &mut Cpu) -> Result<(), DeviceError>;

    /// Reset device to initial state
    fn reset(&mut self);

    /// Get device status
    fn get_status(&self) -> DeviceStatus;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_code: u8,
    pub name: String,
    pub busy: bool,
    pub interrupt_active: bool,
    pub details: HashMap<String, String>,
}

/// IOCC structure for I/O operations
#[derive(Debug, Clone)]
pub struct Iocc {
    pub wca: u16,           // Word Count Address
    pub device_code: u8,
    pub function: DevFunction,
    pub modifiers: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DevFunction {
    SenseDevice = 0,
    Control = 1,
    Read = 2,
    Write = 3,
    InitRead = 4,
    InitWrite = 5,
    SenseInterrupt = 6,
}
```

### Example Device Implementation

```rust
// src/devices/device2501.rs

use super::*;

/// IBM 2501 Card Reader
pub struct Device2501 {
    device_code: u8,
    read_hopper: VecDeque<Card>,
    read_stacker: VecDeque<Card>,
    busy: bool,
    active_interrupt: Option<Interrupt>,
}

impl Device2501 {
    pub fn new() -> Self {
        Self {
            device_code: 0x09,
            read_hopper: VecDeque::new(),
            read_stacker: VecDeque::new(),
            busy: false,
            active_interrupt: None,
        }
    }

    pub fn load_card(&mut self, card: Card) {
        self.read_hopper.push_back(card);
    }
}

impl Device for Device2501 {
    fn device_code(&self) -> u8 {
        self.device_code
    }

    fn name(&self) -> &'static str {
        "IBM 2501 Card Reader"
    }

    fn execute_iocc(&mut self, cpu: &mut Cpu) -> Result<(), DeviceError> {
        let iocc = cpu.get_iocc()?;

        match iocc.function {
            DevFunction::InitRead => {
                if let Some(card) = self.read_hopper.pop_front() {
                    self.busy = true;

                    // Read word count
                    let wc = cpu.read_memory(iocc.wca as usize)?;

                    // Transfer card data to memory
                    let buffer_start = iocc.wca + 1;
                    for (i, &column) in card.columns.iter().enumerate() {
                        if i >= wc as usize {
                            break;
                        }
                        cpu.write_memory((buffer_start + i as u16) as usize, column)?;
                    }

                    // Move to stacker
                    self.read_stacker.push_back(card);

                    // Generate interrupt on level 4
                    cpu.add_interrupt(Interrupt::new(4, 0x0008, self.device_code));

                    self.busy = false;
                    Ok(())
                } else {
                    Err(DeviceError::NoCard)
                }
            }
            DevFunction::SenseDevice => {
                let status = if self.read_hopper.is_empty() { 0x8000 } else { 0 };
                cpu.acc = status;
                Ok(())
            }
            _ => Err(DeviceError::UnsupportedFunction(iocc.function))
        }
    }

    fn reset(&mut self) {
        self.busy = false;
        self.active_interrupt = None;
    }

    fn get_status(&self) -> DeviceStatus {
        DeviceStatus {
            device_code: self.device_code,
            name: self.name().to_string(),
            busy: self.busy,
            interrupt_active: self.active_interrupt.is_some(),
            details: [
                ("hopper_count".to_string(), self.read_hopper.len().to_string()),
                ("stacker_count".to_string(), self.read_stacker.len().to_string()),
            ].into_iter().collect(),
        }
    }
}

/// Card representation (80 columns)
#[derive(Debug, Clone)]
pub struct Card {
    pub columns: [u16; 80],
}
```

## WASM Bindings Design

### WASM CPU Wrapper

```rust
// crates/s1130-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use s1130_core::{Cpu, CpuState, AssemblyResult};

#[wasm_bindgen]
pub struct WasmCpu {
    inner: Cpu,
}

#[wasm_bindgen]
impl WasmCpu {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();

        Self {
            inner: Cpu::new(),
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        self.inner.step()
            .map(|_| JsValue::NULL)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner = Cpu::new();
    }

    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> JsValue {
        let state = self.inner.get_state();
        serde_wasm_bindgen::to_value(&state).unwrap()
    }

    #[wasm_bindgen]
    pub fn assemble(&mut self, source: &str) -> JsValue {
        let mut assembler = s1130_core::Assembler::new();
        let result = assembler.assemble(source, &mut self.inner);
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    #[wasm_bindgen(js_name = readMemory)]
    pub fn read_memory(&self, address: u16, count: u16) -> Vec<u16> {
        (address..address.saturating_add(count))
            .filter_map(|addr| self.inner.read_memory(addr as usize).ok())
            .collect()
    }

    #[wasm_bindgen(js_name = writeMemory)]
    pub fn write_memory(&mut self, address: u16, value: u16) -> Result<(), JsValue> {
        self.inner.write_memory(address as usize, value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

## UI Component Design

### Yew Application Structure

```rust
// crates/s1130-ui/src/app.rs

use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <EmulatorProvider>
            <div class="app-container">
                <header class="app-header">
                    <h1>{ "IBM 1130 Emulator" }</h1>
                </header>
                <main class="app-main">
                    <div class="left-panel">
                        <AssemblerEditor />
                    </div>
                    <div class="right-panel">
                        <ControlPanel />
                        <CpuConsole />
                        <MemoryViewer />
                    </div>
                </main>
            </div>
        </EmulatorProvider>
    }
}
```

### Emulator Context (Shared State)

```rust
// crates/s1130-ui/src/context/emulator.rs

use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use s1130_wasm::WasmCpu;
use gloo::timers::callback::Interval;

#[derive(Clone)]
pub struct EmulatorContext {
    pub cpu: Rc<RefCell<WasmCpu>>,
    pub is_running: bool,
    pub dispatch: Callback<EmulatorAction>,
}

pub enum EmulatorAction {
    Reset,
    Step,
    Run { ips: u32 },
    Stop,
    Assemble { source: String },
    UpdateState,
}

pub struct EmulatorProvider {
    cpu: Rc<RefCell<WasmCpu>>,
    is_running: bool,
    execution_handle: Option<Interval>,
}

pub enum EmulatorMsg {
    Action(EmulatorAction),
    Tick,
}

impl Component for EmulatorProvider {
    type Message = EmulatorMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            cpu: Rc::new(RefCell::new(WasmCpu::new())),
            is_running: false,
            execution_handle: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EmulatorMsg::Action(action) => {
                self.handle_action(ctx, action)
            }
            EmulatorMsg::Tick => {
                if self.is_running {
                    let _ = self.cpu.borrow_mut().step();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let context = EmulatorContext {
            cpu: self.cpu.clone(),
            is_running: self.is_running,
            dispatch: ctx.link().callback(EmulatorMsg::Action),
        };

        html! {
            <ContextProvider<EmulatorContext> {context}>
                { for ctx.props().children.iter() }
            </ContextProvider<EmulatorContext>>
        }
    }
}

impl EmulatorProvider {
    fn handle_action(&mut self, ctx: &Context<Self>, action: EmulatorAction) -> bool {
        match action {
            EmulatorAction::Reset => {
                self.cpu.borrow_mut().reset();
                self.stop_execution();
                true
            }
            EmulatorAction::Step => {
                let _ = self.cpu.borrow_mut().step();
                true
            }
            EmulatorAction::Run { ips } => {
                self.start_execution(ctx, ips);
                true
            }
            EmulatorAction::Stop => {
                self.stop_execution();
                true
            }
            EmulatorAction::Assemble { source } => {
                let result = self.cpu.borrow_mut().assemble(&source);
                // Dispatch result event
                true
            }
            EmulatorAction::UpdateState => true,
        }
    }

    fn start_execution(&mut self, ctx: &Context<Self>, ips: u32) {
        let interval_ms = 1000 / ips;
        let link = ctx.link().clone();

        let handle = Interval::new(interval_ms, move || {
            link.send_message(EmulatorMsg::Tick);
        });

        self.execution_handle = Some(handle);
        self.is_running = true;
    }

    fn stop_execution(&mut self) {
        self.execution_handle = None;
        self.is_running = false;
    }
}
```

### CPU Console Component

```rust
// crates/s1130-ui/src/components/cpu_console.rs

use yew::prelude::*;
use crate::context::EmulatorContext;

#[function_component(CpuConsole)]
pub fn cpu_console() -> Html {
    let context = use_context::<EmulatorContext>().expect("No context found");

    let state = use_state(|| {
        serde_wasm_bindgen::from_value(context.cpu.borrow().get_state()).unwrap()
    });

    // Update state periodically
    {
        let state = state.clone();
        let cpu = context.cpu.clone();
        use_effect_with((), move |_| {
            let interval = gloo::timers::callback::Interval::new(100, move || {
                let new_state = serde_wasm_bindgen::from_value(
                    cpu.borrow().get_state()
                ).unwrap();
                state.set(new_state);
            });

            move || drop(interval)
        });
    }

    html! {
        <div class="cpu-console">
            <h2>{ "CPU Registers" }</h2>
            <RegisterDisplay name="IAR" value={state.iar} />
            <RegisterDisplay name="ACC" value={state.acc} />
            <RegisterDisplay name="EXT" value={state.ext} />
            <RegisterDisplay name="XR1" value={state.xr1} />
            <RegisterDisplay name="XR2" value={state.xr2} />
            <RegisterDisplay name="XR3" value={state.xr3} />

            <div class="status-flags">
                <StatusLed label="Carry" active={state.carry} />
                <StatusLed label="Overflow" active={state.overflow} />
                <StatusLed label="Wait" active={state.wait} />
            </div>

            <div class="stats">
                <p>{ format!("Instructions: {}", state.instruction_count) }</p>
                {
                    if let Some(level) = state.current_interrupt_level {
                        html! { <p>{ format!("Interrupt Level: {}", level) }</p> }
                    } else {
                        html! { <p>{ "No interrupt active" }</p> }
                    }
                }
            </div>
        </div>
    }
}
```

### Register Display Component

```rust
// crates/s1130-ui/src/components/register_display.rs

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RegisterDisplayProps {
    pub name: String,
    pub value: u16,
}

#[function_component(RegisterDisplay)]
pub fn register_display(props: &RegisterDisplayProps) -> Html {
    html! {
        <div class="register-display">
            <label class="register-name">{ &props.name }</label>
            <div class="register-values">
                <span class="hex-value">
                    { format!("{:04X}", props.value) }
                </span>
                <span class="octal-value">
                    { format!("{:06o}", props.value) }
                </span>
                <span class="decimal-value">
                    { format!("{}", props.value) }
                </span>
            </div>
            <div class="bit-display">
                { for (0..16).rev().map(|bit| {
                    let is_set = (props.value & (1 << bit)) != 0;
                    html! {
                        <span class={classes!("bit", is_set.then(|| "bit-on"))}>
                            { if is_set { "1" } else { "0" } }
                        </span>
                    }
                }) }
            </div>
        </div>
    }
}
```

## Testing Design

### Unit Test Structure

```rust
// tests/cpu_tests.rs

#[cfg(test)]
mod cpu_tests {
    use s1130_core::*;

    fn setup_cpu() -> Cpu {
        let mut cpu = Cpu::new();
        cpu.iar = 0x100;
        cpu
    }

    #[test]
    fn test_load_short_format() {
        let mut cpu = setup_cpu();

        // Set up memory
        cpu.write_memory(0x105, 0x1234).unwrap();

        // Build LD 5 instruction at 0x100
        let instruction = build_short_instruction(OpCode::Load, 0, 5);
        cpu.write_memory(0x100, instruction).unwrap();

        // Execute
        cpu.next_instruction().unwrap();
        cpu.execute_instruction().unwrap();

        // Verify
        assert_eq!(cpu.acc, 0x1234);
        assert_eq!(cpu.iar, 0x101);
    }

    fn build_short_instruction(opcode: OpCode, tag: u8, displacement: u8) -> u16 {
        ((opcode as u16) << 11) | ((tag as u16) << 8) | (displacement as u16)
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutativity(a in any::<u16>(), b in any::<u16>()) {
        let mut cpu1 = Cpu::new();
        let mut cpu2 = Cpu::new();

        // Test a + b
        cpu1.acc = a;
        cpu1.write_memory(0x100, b).unwrap();
        // Execute add...

        // Test b + a
        cpu2.acc = b;
        cpu2.write_memory(0x100, a).unwrap();
        // Execute add...

        prop_assert_eq!(cpu1.acc, cpu2.acc);
        prop_assert_eq!(cpu1.carry, cpu2.carry);
    }
}
```

## Performance Design

### Optimization Strategies

1. **Inline Hot Paths**:
```rust
#[inline(always)]
pub fn execute_instruction(&mut self) -> Result<(), CpuError> {
    // Hot path - inline to eliminate call overhead
}
```

2. **Avoid Allocations**:
```rust
// Use stack-allocated arrays where possible
let mut buffer: [u16; 80] = [0; 80];

// Reuse Vec capacity
self.errors.clear();  // Don't drop and reallocate
```

3. **Efficient Lookups**:
```rust
// Use match for small enums (faster than HashMap)
match opcode {
    OpCode::Load => /* ... */,
    OpCode::Store => /* ... */,
}
```

4. **WASM-Specific**:
```rust
// Build with optimizations
[profile.release]
opt-level = "z"  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
```

### Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_instruction_execution(c: &mut Criterion) {
    c.bench_function("execute 1000 add instructions", |b| {
        let mut cpu = Cpu::new();
        setup_add_loop(&mut cpu);

        b.iter(|| {
            for _ in 0..1000 {
                cpu.step().unwrap();
            }
        });
    });
}

criterion_group!(benches, benchmark_instruction_execution);
criterion_main!(benches);
```

## Summary

This design document provides comprehensive technical specifications for implementing the S1130 Rust + Yew port. Key design decisions include:

- **Strong typing** with Rust's type system for correctness
- **Trait-based architecture** for extensibility and testability
- **WASM-native bindings** for seamless browser integration
- **Component-based UI** with Yew for reactive rendering
- **Context-based state management** for shared emulator state
- **Comprehensive testing** with unit, integration, and property-based tests
- **Performance-optimized** implementation targeting 100K+ IPS

The design maintains functional parity with the C# implementation while leveraging Rust's safety guarantees and modern web technologies for a superior user experience.
