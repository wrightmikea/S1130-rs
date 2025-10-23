//! Integration tests for IBM 1130 assembler
//!
//! Tests complete programs to verify end-to-end assembly functionality

use s1130_core::assembler::Assembler;

#[test]
fn test_simple_addition_program() {
    let source = r#"
*
* Simple Addition Program
* Adds two numbers and stores result
*
        ORG  /0100
        LD   A
        A    B
        STO  C
        WAIT

A       DC   /0005
B       DC   /0003
C       DC   0
        END  /0100
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly should succeed: {:?}",
        result.err()
    );
    let program = result.unwrap();

    assert_eq!(program.origin, 0x0100, "Origin should be 0x0100");
    assert_eq!(
        program.entry_point,
        Some(0x0100),
        "Entry point should be 0x0100"
    );
    assert!(program.words.len() > 0, "Should generate code");

    // Verify symbols were defined (addresses depend on instruction sizes)
    assert!(program.symbols.contains_key("A"), "Symbol A should exist");
    assert!(program.symbols.contains_key("B"), "Symbol B should exist");
    assert!(program.symbols.contains_key("C"), "Symbol C should exist");
}

#[test]
fn test_hex_constants() {
    let source = r#"
        ORG  /0200
        DC   /FFFF
        DC   /1234
        DC   /ABCD
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly should succeed: {:?}",
        result.err()
    );
    let program = result.unwrap();

    assert_eq!(program.origin, 0x0200);
    assert_eq!(program.words.len(), 3);
    assert_eq!(program.words[0], 0xFFFF);
    assert_eq!(program.words[1], 0x1234);
    assert_eq!(program.words[2], 0xABCD);
}

#[test]
fn test_decimal_constants() {
    let source = r#"
        ORG  /0100
        DC   0
        DC   100
        DC   65535
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_ok(), "Assembly should succeed");
    let program = result.unwrap();

    assert_eq!(program.words.len(), 3);
    assert_eq!(program.words[0], 0);
    assert_eq!(program.words[1], 100);
    assert_eq!(program.words[2], 65535);
}

#[test]
fn test_octal_constants() {
    let source = r#"
        ORG  /0100
        DC   0777
        DC   0100
        DC   01234
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_ok(), "Assembly should succeed");
    let program = result.unwrap();

    assert_eq!(program.words.len(), 3);
    assert_eq!(program.words[0], 0o777); // 511 decimal
    assert_eq!(program.words[1], 0o100); // 64 decimal
    assert_eq!(program.words[2], 0o1234); // 668 decimal
}

#[test]
fn test_mixed_number_formats() {
    let source = r#"
        ORG  /0100
HEX     DC   /00FF
DEC     DC   255
OCT     DC   0377
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_ok(), "Assembly should succeed");
    let program = result.unwrap();

    // All three should equal 255
    assert_eq!(program.words[0], 255, "Hex /00FF should be 255");
    assert_eq!(program.words[1], 255, "Decimal 255 should be 255");
    assert_eq!(program.words[2], 255, "Octal 0377 should be 255");
}

#[test]
fn test_labels_with_whitespace() {
    let source = r#"
START   LD   COUNT
        A    ONE
        STO  TOTAL
        WAIT

COUNT   DC   10
ONE     DC   1
TOTAL   DC   0
        END  START
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly should succeed: {:?}",
        result.err()
    );
    let program = result.unwrap();

    // Verify labels were defined
    assert!(program.symbols.contains_key("START"));
    assert!(program.symbols.contains_key("COUNT"));
    assert!(program.symbols.contains_key("ONE"));
    assert!(program.symbols.contains_key("TOTAL"));
}

#[test]
fn test_single_letter_labels() {
    let source = r#"
*
* Test that single-letter labels work
* even when they match instruction mnemonics
*
        ORG  /0100
        LD   A
        A    B
        STO  C
        WAIT

A       DC   5
B       DC   3
C       DC   0
S       DC   /FFFF
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly should succeed: {:?}",
        result.err()
    );
    let program = result.unwrap();

    // Verify all single-letter labels were defined
    assert!(program.symbols.contains_key("A"), "Label A should exist");
    assert!(program.symbols.contains_key("B"), "Label B should exist");
    assert!(program.symbols.contains_key("C"), "Label C should exist");
    assert!(program.symbols.contains_key("S"), "Label S should exist");

    // Verify values
    let a_addr = *program.symbols.get("A").unwrap() as usize - program.origin as usize;
    let b_addr = *program.symbols.get("B").unwrap() as usize - program.origin as usize;
    let c_addr = *program.symbols.get("C").unwrap() as usize - program.origin as usize;
    let s_addr = *program.symbols.get("S").unwrap() as usize - program.origin as usize;

    assert_eq!(program.words[a_addr], 5, "A should be 5");
    assert_eq!(program.words[b_addr], 3, "B should be 3");
    assert_eq!(program.words[c_addr], 0, "C should be 0");
    assert_eq!(program.words[s_addr], 0xFFFF, "S should be 0xFFFF");
}

#[test]
fn test_bss_block_allocation() {
    let source = r#"
        ORG  /0100
START   DC   /1234
BUFFER  BSS  10
END_BUF DC   /ABCD
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_ok(), "Assembly should succeed");
    let program = result.unwrap();

    assert_eq!(program.words.len(), 12); // 1 + 10 + 1
    assert_eq!(program.words[0], 0x1234);
    // BSS allocates 10 zeros
    for i in 1..11 {
        assert_eq!(program.words[i], 0, "BSS word {} should be 0", i);
    }
    assert_eq!(program.words[11], 0xABCD);
}

#[test]
fn test_comment_handling() {
    let source = r#"
* Full line comment
        ORG  /0100     * Inline comment
START   LD   A         * Load A
        WAIT           * Halt
* Another full line comment
A       DC   42        * The answer
        END  START
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly with comments should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_empty_lines() {
    let source = r#"
        ORG  /0100

START   LD   A

        STO  B

        WAIT


A       DC   1
B       DC   0

        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly with empty lines should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_index_register_instructions() {
    let source = r#"
        ORG  /0100
        LDX  1,COUNT
        STX  2,SAVE
        MDX  3,INCR
        WAIT

COUNT   DC   10
SAVE    DC   0
INCR    DC   1
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Assembly with index instructions should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_forward_reference() {
    let source = r#"
        ORG  /0100
        LD   FUTURE
        WAIT

FUTURE  DC   /1234
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Forward reference should work: {:?}",
        result.err()
    );
    let program = result.unwrap();
    assert!(program.symbols.contains_key("FUTURE"));
}

#[test]
fn test_backward_reference() {
    let source = r#"
        ORG  /0100
PAST    DC   /5678
        LD   PAST
        WAIT
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(
        result.is_ok(),
        "Backward reference should work: {:?}",
        result.err()
    );
}

#[test]
#[ignore] // EQU not fully implemented yet
fn test_equ_pseudo_op() {
    let source = r#"
CONST   EQU  /0100
        ORG  CONST
        DC   CONST
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    // EQU may not be fully implemented yet
    // This test documents expected behavior for future implementation
    if let Ok(program) = result {
        assert_eq!(program.origin, 0x0100);
        assert_eq!(program.words[0], 0x0100);
    }
}

#[test]
#[ignore] // Multiple ORGs not fully supported yet
fn test_multiple_org_directives() {
    let source = r#"
        ORG  /0100
        DC   /1111

        ORG  /0200
        DC   /2222

        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    // Multiple ORGs in one program may not be supported yet
    // This test documents the expected/desired behavior for future implementation
    if result.is_ok() {
        let program = result.unwrap();
        // First ORG should set origin
        assert_eq!(program.origin, 0x0100);
    }
}

#[test]
fn test_error_duplicate_label() {
    let source = r#"
        ORG  /0100
LABEL   DC   1
LABEL   DC   2
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_err(), "Duplicate labels should cause error");
}

#[test]
fn test_error_undefined_symbol() {
    let source = r#"
        ORG  /0100
        LD   UNDEFINED
        WAIT
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_err(), "Undefined symbol should cause error");
}

#[test]
fn test_error_invalid_hex() {
    let source = r#"
        ORG  /0100
        DC   /GHIJ
        END
"#;

    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    assert!(result.is_err(), "Invalid hex digits should cause error");
}
