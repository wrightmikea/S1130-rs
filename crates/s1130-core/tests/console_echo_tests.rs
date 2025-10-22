//! Console Echo Integration Tests
//!
//! These tests verify the console keyboard and printer devices work together
//! by running a simple echo program that reads from keyboard and writes to printer.

use s1130_core::assembler::Assembler;
use s1130_core::devices::{DeviceConsoleKeyboard, DeviceConsolePrinter};
use s1130_core::Cpu;

#[test]
fn test_simple_echo_hello() {
    // Create CPU with keyboard (device 1) and printer (device 2)
    let mut cpu = Cpu::new();

    // Type "hello\n" into keyboard before attaching
    let mut keyboard = DeviceConsoleKeyboard::new();
    keyboard.type_string("hello\n");

    let printer = DeviceConsolePrinter::new();

    // Attach devices to CPU (they're moved into Box)
    cpu.attach_device(Box::new(keyboard)).unwrap();
    cpu.attach_device(Box::new(printer)).unwrap();

    // Assemble echo program
    let source = r#"
        ORG 0x100

* Echo program: reads characters from keyboard and writes to printer
* Fixed number of characters (6 for "hello\n")

        XIO KREAD1      * Read char 1
        LD  CHAR
        XIO PWRITE

        XIO KREAD1      * Read char 2
        LD  CHAR
        XIO PWRITE

        XIO KREAD1      * Read char 3
        LD  CHAR
        XIO PWRITE

        XIO KREAD1      * Read char 4
        LD  CHAR
        XIO PWRITE

        XIO KREAD1      * Read char 5
        LD  CHAR
        XIO PWRITE

        XIO KREAD1      * Read char 6 (newline)
        LD  CHAR
        XIO PWRITE

        WAIT

* IOCC structures for keyboard (device 1)
KREAD1  DC  CHAR        * WCA for read
        DC  0x0B00      * Device 1 (0x0800), Function 3 (0x0300)

* IOCC structures for printer (device 2)
PWRITE  DC  CHAR        * WCA for write
        DC  0x1500      * Device 2 (0x1000), Function 5 (0x0500)

* Data areas
CHAR    BSS 1           * Character buffer
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    // Load program into memory
    cpu.write_memory_range(program.origin as usize, &program.words)
        .unwrap();
    cpu.set_iar(program.origin);

    // Run program (max 1000 steps to prevent infinite loop)
    let steps = cpu.run(1000);

    println!("Executed {} steps", steps);

    // Get printer output by downcasting
    let printer_device = cpu.get_device_mut_ref(2).unwrap();
    let printer = printer_device
        .as_any()
        .downcast_ref::<DeviceConsolePrinter>()
        .expect("Expected DeviceConsolePrinter");

    let output = printer.get_output();
    println!("Printer output: {:?}", output);

    // Verify output
    assert_eq!(output, "hello\n");
}

#[test]
fn test_echo_single_char() {
    let mut cpu = Cpu::new();

    let mut keyboard = DeviceConsoleKeyboard::new();
    keyboard.type_char(b'X' as u16);
    keyboard.type_char(b'\n' as u16);

    cpu.attach_device(Box::new(keyboard)).unwrap();
    cpu.attach_device(Box::new(DeviceConsolePrinter::new()))
        .unwrap();

    // Simple 2-character echo
    let source = r#"
        ORG 0x100

        XIO KREAD
        LD  CHAR
        XIO PWRITE

        XIO KREAD
        LD  CHAR
        XIO PWRITE

        WAIT

KREAD   DC  CHAR
        DC  0x0B00      * Device 1 (0x0800), Function 3 (0x0300)

PWRITE  DC  CHAR
        DC  0x1500      * Device 2 (0x1000), Function 5 (0x0500)

CHAR    BSS 1
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    cpu.write_memory_range(program.origin as usize, &program.words)
        .unwrap();
    cpu.set_iar(program.origin);

    cpu.run(1000);

    let printer_device = cpu.get_device_mut_ref(2).unwrap();
    let printer = printer_device
        .as_any()
        .downcast_ref::<DeviceConsolePrinter>()
        .unwrap();

    assert_eq!(printer.get_output(), "X\n");
}

#[test]
fn test_keyboard_sense_before_read() {
    // Test that sensing keyboard works correctly
    let mut cpu = Cpu::new();

    let mut keyboard = DeviceConsoleKeyboard::new();
    keyboard.type_char(b'A' as u16);

    cpu.attach_device(Box::new(keyboard)).unwrap();

    let source = r#"
        ORG 0x100

* Sense keyboard
        XIO KSENSE
        LD  KSTAT
        STO RESULT      * Should be 1 (char ready)
        WAIT

KSENSE  DC  KSTAT
        DC  0x0800      * Device 1, Sense

KSTAT   BSS 1
RESULT  BSS 1
    "#;

    let mut assembler = Assembler::new();
    let program = assembler.assemble(source).unwrap();

    cpu.write_memory_range(program.origin as usize, &program.words)
        .unwrap();
    cpu.set_iar(program.origin);

    cpu.run(100);

    let result_addr = program.symbols.get("RESULT").unwrap();
    let result = cpu.read_memory(*result_addr as usize).unwrap();

    assert_eq!(result, 1); // Character is ready
}
