//! Integration tests for the assembler
//!
//! These tests verify end-to-end assembly of IBM 1130 programs.

use s1130_core::assembler::Assembler;
use s1130_core::Cpu;

#[test]
fn test_assemble_simple_program() {
    let source = r#"
        ORG 0x100
START   LD  VALUE
        A   ONE
        STO RESULT
        WAIT
VALUE   DC  10
ONE     DC  1
RESULT  BSS 1
        END START
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    // Check that we got some code
    assert!(!program.words.is_empty());

    // Check symbol table
    assert!(program.symbols.contains_key("START"));
    assert!(program.symbols.contains_key("VALUE"));
    assert!(program.symbols.contains_key("ONE"));
    assert!(program.symbols.contains_key("RESULT"));

    // Check entry point
    assert_eq!(
        program.entry_point,
        Some(*program.symbols.get("START").unwrap())
    );
}

#[test]
fn test_assemble_and_execute() {
    let source = r#"
        ORG 0x100
        LD  VALUE
        A   ONE
        STO RESULT
        WAIT
VALUE   DC  10
ONE     DC  1
RESULT  BSS 1
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    // Load program into CPU
    let mut cpu = Cpu::new();
    cpu.write_memory_range(program.origin as usize, &program.words)
        .unwrap();
    cpu.set_iar(program.origin);

    // Execute until WAIT
    cpu.run(100);

    // Check result: should be 10 + 1 = 11
    let result_addr = program.symbols.get("RESULT").unwrap();
    let result_value = cpu.read_memory(*result_addr as usize).unwrap();
    assert_eq!(result_value, 11);
}

#[test]
fn test_assemble_with_labels() {
    let source = r#"
LOOP    LD  COUNT
        S   ONE
        STO COUNT
        WAIT
COUNT   DC  5
ONE     DC  1
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    assert_eq!(program.symbols.get("LOOP"), Some(&0));
    assert!(program.symbols.contains_key("COUNT"));
    assert!(program.symbols.contains_key("ONE"));
}

#[test]
fn test_assemble_hex_literals() {
    let source = r#"
        LD  0x100
        STO 0x200
        WAIT
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());
}

#[test]
fn test_assemble_octal_literals() {
    let source = r#"
        LD  0777
        WAIT
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());
}

#[test]
fn test_assemble_indirect_addressing() {
    let source = r#"
        LD  /PTR
        WAIT
PTR     DC  0x100
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());
}

#[test]
fn test_assemble_indexed_addressing() {
    let source = r#"
        LDX 1,START
        LD  100,1
        WAIT
START   DC  0
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());
}

#[test]
fn test_assemble_all_instructions() {
    let source = r#"
        LD   100
        LDD  200
        STO  300
        STD  400
        A    100
        AD   200
        S    100
        SD   200
        M    100
        D    100
        AND  100
        OR   100
        EOR  100
        SLA  4
        SLCA 8
        SRA  4
        SRT  8
        WAIT
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());

    // Should have assembled all instructions
    let words = program.unwrap().words;
    assert!(words.len() > 17); // At least one word per instruction
}

#[test]
fn test_assemble_bss_pseudo_op() {
    let source = r#"
START   DC   1
BUFFER  BSS  10
END_BUF DC   2
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    // BSS should reserve 10 words
    let start_addr = *program.symbols.get("START").unwrap();
    let buffer_addr = *program.symbols.get("BUFFER").unwrap();
    let end_addr = *program.symbols.get("END_BUF").unwrap();

    assert_eq!(buffer_addr, start_addr + 1);
    assert_eq!(end_addr, buffer_addr + 10);
}

#[test]
fn test_assemble_multiple_dc() {
    let source = r#"
        ORG 0x200
DATA1   DC  100
DATA2   DC  200
DATA3   DC  300
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    assert_eq!(program.origin, 0x200);
    assert_eq!(program.words[0], 100);
    assert_eq!(program.words[1], 200);
    assert_eq!(program.words[2], 300);
}

#[test]
fn test_assemble_with_comments() {
    let source = r#"
* This is a comment
        LD  100  * Load value
        WAIT     * Halt
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source);
    assert!(program.is_ok());
}

#[test]
fn test_assemble_empty_program() {
    let source = r#"
* Just comments
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();
    assert!(program.words.is_empty());
}

#[test]
fn test_duplicate_label_error() {
    let source = r#"
LABEL   DC  1
LABEL   DC  2
    "#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_undefined_symbol_error() {
    let source = r#"
        LD  UNDEFINED
        WAIT
    "#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);
    assert!(result.is_err());
}
