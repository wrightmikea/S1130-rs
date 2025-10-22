//! Unit tests for individual instruction execution
//!
//! These tests verify that each instruction correctly modifies CPU state.

use s1130_core::Cpu;

// === Load/Store Instructions ===

#[test]
fn test_ld_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: LD 0x0200
    cpu.write_memory(0x0100, 0x6000).unwrap(); // LD opcode
    cpu.write_memory(0x0101, 0x0200).unwrap(); // address
    cpu.write_memory(0x0200, 0x1234).unwrap(); // value to load

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0x1234);
    assert_eq!(cpu.get_iar(), 0x0102); // IAR advanced by 2
}

#[test]
fn test_ldd_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: LDD 0x0200
    cpu.write_memory(0x0100, 0x6800).unwrap(); // LDD opcode
    cpu.write_memory(0x0101, 0x0200).unwrap(); // address
    cpu.write_memory(0x0200, 0x1234).unwrap(); // first word
    cpu.write_memory(0x0201, 0x5678).unwrap(); // second word

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0x1234);
    assert_eq!(cpu.get_ext(), 0x5678);
}

#[test]
fn test_sto_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0xABCD);

    // Setup: STO 0x0200
    cpu.write_memory(0x0100, 0x7000).unwrap(); // STO opcode
    cpu.write_memory(0x0101, 0x0200).unwrap(); // address

    cpu.step().unwrap();

    assert_eq!(cpu.read_memory(0x0200).unwrap(), 0xABCD);
}

#[test]
fn test_std_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0x1111);
    cpu.set_ext(0x2222);

    // Setup: STD 0x0200
    cpu.write_memory(0x0100, 0x7800).unwrap(); // STD opcode
    cpu.write_memory(0x0101, 0x0200).unwrap(); // address

    cpu.step().unwrap();

    assert_eq!(cpu.read_memory(0x0200).unwrap(), 0x1111);
    assert_eq!(cpu.read_memory(0x0201).unwrap(), 0x2222);
}

// === Arithmetic Instructions ===

#[test]
fn test_add_positive() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(100);

    // Setup: A 0x0200 (add 50)
    cpu.write_memory(0x0100, 0xE000).unwrap(); // A opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 50).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 150);
    assert!(!cpu.get_carry());
    assert!(!cpu.get_overflow());
}

#[test]
fn test_add_with_carry() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0xFFFF);

    // Setup: A 0x0200 (add 1, should wrap)
    cpu.write_memory(0x0100, 0xE000).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 1).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0);
    assert!(cpu.get_carry());
}

#[test]
fn test_add_overflow() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0x7FFF); // Max positive i16

    // Setup: A 0x0200 (add 1, should overflow)
    cpu.write_memory(0x0100, 0xE000).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 1).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc() as i16, -32768); // Wraps to min negative
    assert!(cpu.get_overflow());
}

#[test]
fn test_subtract_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(100);

    // Setup: S 0x0200 (subtract 30)
    cpu.write_memory(0x0100, 0xC000).unwrap(); // S opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 30).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 70);
    assert!(!cpu.get_carry());
}

#[test]
fn test_subtract_with_borrow() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(10);

    // Setup: S 0x0200 (subtract 20, should underflow)
    cpu.write_memory(0x0100, 0xC000).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 20).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc() as i16, -10);
    assert!(cpu.get_carry()); // Borrow occurred
}

#[test]
fn test_multiply_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(12);

    // Setup: M 0x0200 (multiply by 5)
    cpu.write_memory(0x0100, 0xF000).unwrap(); // M opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 5).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc_ext(), 60);
}

#[test]
fn test_multiply_negative() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc((-10i16) as u16);

    // Setup: M 0x0200 (multiply by 3)
    cpu.write_memory(0x0100, 0xF000).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 3).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc_ext() as i32, -30);
}

#[test]
fn test_divide_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc_ext(100);

    // Setup: D 0x0200 (divide by 7)
    cpu.write_memory(0x0100, 0xF800).unwrap(); // D opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 7).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 14); // Quotient
    assert_eq!(cpu.get_ext(), 2); // Remainder
    assert!(!cpu.get_overflow());
}

#[test]
fn test_divide_by_zero() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc_ext(100);

    // Setup: D 0x0200 (divide by 0)
    cpu.write_memory(0x0100, 0xF800).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0).unwrap();

    cpu.step().unwrap();

    assert!(cpu.get_overflow()); // Overflow set on divide by zero
}

#[test]
fn test_add_double() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc_ext(1000);

    // Setup: AD 0x0200 (add 500)
    cpu.write_memory(0x0100, 0xE800).unwrap(); // AD opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0).unwrap(); // High word
    cpu.write_memory(0x0201, 500).unwrap(); // Low word

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc_ext(), 1500);
}

#[test]
fn test_subtract_double() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc_ext(1000);

    // Setup: SD 0x0200 (subtract 300)
    cpu.write_memory(0x0100, 0xC800).unwrap(); // SD opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0).unwrap();
    cpu.write_memory(0x0201, 300).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc_ext(), 700);
}

// === Logical Instructions ===

#[test]
fn test_and_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b1111_0000_1111_0000);

    // Setup: AND 0x0200
    cpu.write_memory(0x0100, 0x8000).unwrap(); // AND opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0b1010_1010_1010_1010).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0b1010_0000_1010_0000);
}

#[test]
fn test_or_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b1111_0000_0000_0000);

    // Setup: OR 0x0200
    cpu.write_memory(0x0100, 0x9000).unwrap(); // OR opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0b0000_1111_0000_0000).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0b1111_1111_0000_0000);
}

#[test]
fn test_eor_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b1111_0000_1111_0000);

    // Setup: EOR 0x0200
    cpu.write_memory(0x0100, 0x9800).unwrap(); // EOR opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0b1111_1111_0000_0000).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0b0000_1111_1111_0000);
}

// === Shift Instructions ===

#[test]
fn test_sla_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b0000_0001_0000_0000);

    // Setup: SLA 4 (shift left 4 bits)
    cpu.write_memory(0x0100, 0x2004).unwrap(); // SLA opcode with count=4

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0b0001_0000_0000_0000);
}

#[test]
fn test_sla_carry() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b1000_0000_0000_0000);

    // Setup: SLA 1
    cpu.write_memory(0x0100, 0x2001).unwrap();

    cpu.step().unwrap();

    assert!(cpu.get_carry());
}

#[test]
fn test_sra_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b0001_0000_0000_0000);

    // Setup: SRA 4 (shift right 4 bits, arithmetic)
    cpu.write_memory(0x0100, 0x3004).unwrap(); // SRA opcode

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0b0000_0001_0000_0000);
}

#[test]
fn test_sra_sign_extend() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0b1000_0000_0000_0000); // Negative number

    // Setup: SRA 1
    cpu.write_memory(0x0100, 0x3001).unwrap();

    cpu.step().unwrap();

    // Should sign-extend (fill with 1s)
    assert_eq!(cpu.get_acc(), 0b1100_0000_0000_0000);
}

#[test]
fn test_slca_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0x0001);
    cpu.set_ext(0x0000);

    // Setup: SLCA 8
    cpu.write_memory(0x0100, 0x2808).unwrap(); // SLCA opcode

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc(), 0x0100);
    assert_eq!(cpu.get_ext(), 0x0000);
}

#[test]
fn test_srt_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_acc(0x1234);
    cpu.set_ext(0x5678);

    // Setup: SRT 8
    cpu.write_memory(0x0100, 0x3808).unwrap(); // SRT opcode

    cpu.step().unwrap();

    assert_eq!(cpu.get_acc_ext(), 0x00123456);
}

// === Branch Instructions ===

#[test]
fn test_bsi_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: BSI 0x0300
    cpu.write_memory(0x0100, 0x4800).unwrap(); // BSI opcode
    cpu.write_memory(0x0101, 0x0300).unwrap();

    cpu.step().unwrap();

    // Return address stored at 0x0300
    assert_eq!(cpu.read_memory(0x0300).unwrap(), 0x0102); // IAR after instruction
                                                          // IAR should be 0x0301 (subroutine entry)
    assert_eq!(cpu.get_iar(), 0x0301);
}

#[test]
fn test_bc_unconditional() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: BC with indirect addressing to branch to 0x0500
    // BC is short format, so we use indirect bit and store target in memory
    cpu.write_memory(0x0100, 0x4020 | 0x0010).unwrap(); // BC opcode, tag=0, indirect=1, address=0x10
    cpu.write_memory(0x0010, 0x0500).unwrap(); // Indirect target

    cpu.step().unwrap();

    assert_eq!(cpu.get_iar(), 0x0500);
}

#[test]
fn test_bc_carry_true() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_carry(true);

    // Setup: BC with tag=1 (branch if carry), indirect to 0x0500
    cpu.write_memory(0x0100, 0x4060 | 0x0010).unwrap(); // BC, tag=1, indirect=1, addr=0x10
    cpu.write_memory(0x0010, 0x0500).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_iar(), 0x0500);
}

#[test]
fn test_bc_carry_false() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_carry(false);

    // Setup: BC with tag=1 (branch if carry), indirect to 0x0500
    cpu.write_memory(0x0100, 0x4060 | 0x0010).unwrap(); // BC, tag=1, indirect=1, addr=0x10
    cpu.write_memory(0x0010, 0x0500).unwrap();

    cpu.step().unwrap();

    // Should not branch, IAR advances normally
    assert_eq!(cpu.get_iar(), 0x0101); // Short format, advances by 1
}

// === Index Register Instructions ===

#[test]
fn test_ldx_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: LDX 0x0200 (tag=1, load XR1)
    cpu.write_memory(0x0100, 0x7440).unwrap(); // LDX with tag=1
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0x5555).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_index_register(1), 0x5555);
}

#[test]
fn test_stx_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_index_register(2, 0x9999);

    // Setup: STX 0x0200 (tag=2, store XR2)
    cpu.write_memory(0x0100, 0x5480).unwrap(); // STX with tag=2
    cpu.write_memory(0x0101, 0x0200).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.read_memory(0x0200).unwrap(), 0x9999);
}

#[test]
fn test_mdx_no_skip() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_index_register(1, 10);

    // Setup: MDX 0x0200 (tag=1, add 5)
    cpu.write_memory(0x0100, 0x5840).unwrap(); // MDX with tag=1
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0102, 0xB000).unwrap(); // Next instruction (WAIT)
    cpu.write_memory(0x0200, 5).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_index_register(1), 15);
    assert_eq!(cpu.get_iar(), 0x0102); // Normal advance, no skip
}

#[test]
fn test_mdx_with_skip() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_index_register(1, 5);

    // Setup: MDX 0x0200 (tag=1, add -5)
    cpu.write_memory(0x0100, 0x5840).unwrap();
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0102, 0xB000).unwrap(); // This should be skipped
    cpu.write_memory(0x0103, 0xB000).unwrap(); // Next instruction after skip
    cpu.write_memory(0x0200, (-5i16) as u16).unwrap();

    cpu.step().unwrap();

    assert_eq!(cpu.get_index_register(1), 0);
    assert_eq!(cpu.get_iar(), 0x0103); // Skipped one instruction
}

// === Status Instructions ===

#[test]
fn test_lds_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);

    // Setup: LDS 0x0200
    cpu.write_memory(0x0100, 0xC400).unwrap(); // LDS opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();
    cpu.write_memory(0x0200, 0xC000).unwrap(); // Carry and Overflow bits set

    cpu.step().unwrap();

    assert!(cpu.get_carry());
    assert!(cpu.get_overflow());
}

#[test]
fn test_sts_basic() {
    let mut cpu = Cpu::new();
    cpu.set_iar(0x0100);
    cpu.set_carry(true);
    cpu.set_overflow(false);

    // Setup: STS 0x0200
    cpu.write_memory(0x0100, 0xCC00).unwrap(); // STS opcode
    cpu.write_memory(0x0101, 0x0200).unwrap();

    cpu.step().unwrap();

    let status_word = cpu.read_memory(0x0200).unwrap();
    assert_eq!(status_word & 0x8000, 0x8000); // Carry bit set
    assert_eq!(status_word & 0x4000, 0); // Overflow bit clear
}
