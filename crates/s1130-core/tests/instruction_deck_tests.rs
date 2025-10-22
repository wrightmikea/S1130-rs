//! Integration tests using instruction "decks"
//!
//! These tests simulate loading binary instruction decks (programs) into memory
//! and executing them, verifying the CPU state after execution.

use s1130_core::{Cpu, CpuState};

/// Helper struct to represent an instruction deck (binary program)
struct InstructionDeck {
    /// Human-readable name for the test
    name: &'static str,
    /// Starting address to load the program
    start_address: u16,
    /// Binary instruction words
    instructions: Vec<u16>,
    /// Initial CPU state setup (before execution)
    setup: Box<dyn Fn(&mut Cpu)>,
    /// Expected CPU state after execution
    verify: Box<dyn Fn(&CpuState)>,
}

impl InstructionDeck {
    /// Create a new instruction deck
    fn new(
        name: &'static str,
        start_address: u16,
        instructions: Vec<u16>,
        setup: impl Fn(&mut Cpu) + 'static,
        verify: impl Fn(&CpuState) + 'static,
    ) -> Self {
        Self {
            name,
            start_address,
            instructions,
            setup: Box::new(setup),
            verify: Box::new(verify),
        }
    }

    /// Load the deck into CPU memory and execute
    fn execute(&self, max_steps: u64) -> CpuState {
        let mut cpu = Cpu::new();

        // Apply initial setup
        (self.setup)(&mut cpu);

        // Load instructions into memory
        cpu.write_memory_range(self.start_address as usize, &self.instructions)
            .expect("Failed to load instruction deck");

        // Set IAR to start address
        cpu.set_iar(self.start_address);

        // Execute
        let _steps = cpu.run(max_steps);

        // Return final state
        cpu.get_state()
    }

    /// Run the test
    fn run(&self, max_steps: u64) {
        println!("Running deck test: {}", self.name);
        let final_state = self.execute(max_steps);
        (self.verify)(&final_state);
    }
}

#[test]
fn test_single_wait_instruction() {
    let deck = InstructionDeck::new(
        "Single WAIT",
        0x0100,
        vec![0xB000], // WAIT
        |_cpu| {
            // No initial setup needed
        },
        |state| {
            // Verify CPU halted
            assert!(state.wait, "CPU should be in wait state");
            assert_eq!(state.iar, 0x0101, "IAR should advance by 1");
            assert_eq!(state.instruction_count, 1, "Should execute 1 instruction");
        },
    );

    deck.run(10);
}

#[test]
fn test_wait_with_initial_acc() {
    let deck = InstructionDeck::new(
        "WAIT with ACC initialized",
        0x0200,
        vec![0xB000], // WAIT
        |cpu| {
            cpu.set_acc(0x1234);
            cpu.set_ext(0x5678);
        },
        |state| {
            assert!(state.wait);
            assert_eq!(state.acc, 0x1234, "ACC should be preserved");
            assert_eq!(state.ext, 0x5678, "EXT should be preserved");
        },
    );

    deck.run(10);
}

#[test]
fn test_wait_with_index_registers() {
    let deck = InstructionDeck::new(
        "WAIT with index registers",
        0x0100,
        vec![0xB000], // WAIT
        |cpu| {
            cpu.set_index_register(1, 0x1111);
            cpu.set_index_register(2, 0x2222);
            cpu.set_index_register(3, 0x3333);
        },
        |state| {
            assert!(state.wait);
            assert_eq!(state.xr1, 0x1111, "XR1 should be preserved");
            assert_eq!(state.xr2, 0x2222, "XR2 should be preserved");
            assert_eq!(state.xr3, 0x3333, "XR3 should be preserved");
        },
    );

    deck.run(10);
}

#[test]
fn test_wait_with_flags() {
    let deck = InstructionDeck::new(
        "WAIT with flags set",
        0x0100,
        vec![0xB000], // WAIT
        |cpu| {
            cpu.set_carry(true);
            cpu.set_overflow(true);
        },
        |state| {
            assert!(state.wait);
            assert!(state.carry, "Carry flag should be preserved");
            assert!(state.overflow, "Overflow flag should be preserved");
        },
    );

    deck.run(10);
}

#[test]
fn test_deck_at_different_addresses() {
    // Test that instruction execution works at various memory locations
    let test_addresses = [0x0000, 0x0100, 0x1000, 0x7FFF];

    for &addr in &test_addresses {
        let deck = InstructionDeck::new(
            "WAIT at various addresses",
            addr,
            vec![0xB000], // WAIT
            |_cpu| {},
            move |state| {
                assert!(state.wait);
                assert_eq!(
                    state.iar,
                    addr.wrapping_add(1),
                    "IAR should advance from address {:#06x}",
                    addr
                );
            },
        );

        deck.run(10);
    }
}

#[test]
fn test_instruction_deck_format() {
    // Test that we can load and decode instructions correctly
    let deck = InstructionDeck::new(
        "Mixed format instructions (not executed)",
        0x0100,
        vec![
            0xB000, // WAIT (short format, 1 word)
            0x6000, 0x1234, // LD 0x1234 (long format, 2 words)
            0xB000, // WAIT (short format, 1 word)
        ],
        |_cpu| {},
        |state| {
            // First WAIT should execute
            assert!(state.wait);
            assert_eq!(state.iar, 0x0101);
        },
    );

    deck.run(1); // Only execute first instruction
}

#[test]
fn test_empty_deck() {
    // Test that loading an empty deck doesn't cause issues
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Don't load any instructions, memory will be all zeros (invalid opcode)
    let result = cpu.step();

    // Should fail with invalid instruction
    assert!(result.is_err());
}

#[test]
fn test_memory_mapped_registers_in_deck() {
    let deck = InstructionDeck::new(
        "WAIT with memory-mapped index registers",
        0x0100,
        vec![0xB000], // WAIT
        |cpu| {
            // Set index registers via memory writes
            cpu.write_memory(0x0001, 0xAAAA).unwrap(); // XR1
            cpu.write_memory(0x0002, 0xBBBB).unwrap(); // XR2
            cpu.write_memory(0x0003, 0xCCCC).unwrap(); // XR3
        },
        |state| {
            assert!(state.wait);
            assert_eq!(state.xr1, 0xAAAA, "XR1 via memory-mapped location");
            assert_eq!(state.xr2, 0xBBBB, "XR2 via memory-mapped location");
            assert_eq!(state.xr3, 0xCCCC, "XR3 via memory-mapped location");
        },
    );

    deck.run(10);
}

/// Test helper: create a deck that verifies instruction count
#[test]
fn test_instruction_counting() {
    let deck = InstructionDeck::new(
        "Instruction count verification",
        0x0100,
        vec![0xB000], // One WAIT instruction
        |_cpu| {},
        |state| {
            assert_eq!(state.instruction_count, 1, "Should count 1 instruction");
        },
    );

    deck.run(10);
}

/// Future test template: this will be enabled when more instructions are implemented
#[test]
#[ignore = "LD instruction not yet implemented"]
fn test_ld_instruction_deck() {
    let deck = InstructionDeck::new(
        "LD instruction",
        0x0100,
        vec![
            0x6000, 0x0200, // LD 0x0200 (load from address 0x0200)
            0xB000, // WAIT
        ],
        |cpu| {
            cpu.write_memory(0x0200, 0xABCD).unwrap(); // Value to load
        },
        |state| {
            assert_eq!(state.acc, 0xABCD, "ACC should contain loaded value");
            assert!(state.wait);
        },
    );

    deck.run(10);
}

/// Future test template: this will be enabled when arithmetic is implemented
#[test]
#[ignore = "Arithmetic instructions not yet implemented"]
fn test_arithmetic_deck() {
    let deck = InstructionDeck::new(
        "ADD instruction",
        0x0100,
        vec![
            0x6000, 0x0200, // LD 0x0200
            0xE000, 0x0201, // A 0x0201 (add)
            0xB000, // WAIT
        ],
        |cpu| {
            cpu.write_memory(0x0200, 5).unwrap();
            cpu.write_memory(0x0201, 3).unwrap();
        },
        |state| {
            assert_eq!(state.acc, 8, "5 + 3 = 8");
            assert!(state.wait);
        },
    );

    deck.run(10);
}
