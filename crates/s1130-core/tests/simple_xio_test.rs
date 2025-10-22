//! Simple XIO test to debug device interaction

use s1130_core::assembler::Assembler;
use s1130_core::devices::{DeviceConsoleKeyboard, DeviceConsolePrinter};
use s1130_core::Cpu;

#[test]
fn test_xio_basic() {
    let mut cpu = Cpu::new();

    let mut keyboard = DeviceConsoleKeyboard::new();
    keyboard.type_char(b'A' as u16);

    cpu.attach_device(Box::new(keyboard)).unwrap();
    cpu.attach_device(Box::new(DeviceConsolePrinter::new()))
        .unwrap();

    // Very simple program: just read one char and write it
    let source = r#"
        ORG 0x100

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

    println!("Program assembled successfully");
    println!("Origin: 0x{:04X}", program.origin);
    println!("Words: {:?}", program.words);
    println!("Symbols: {:?}", program.symbols);

    cpu.write_memory_range(program.origin as usize, &program.words)
        .unwrap();
    cpu.set_iar(program.origin);

    println!("Starting execution...");

    // Try single-stepping instead of run
    for i in 0..10 {
        println!("Step {}: IAR=0x{:04X}", i, cpu.get_iar());
        match cpu.step() {
            Ok(()) => println!("  Step OK"),
            Err(e) => {
                println!("  Step error: {:?}", e);
                break;
            }
        }
        let state = cpu.get_state();
        println!("  ACC=0x{:04X}, Wait={}", state.acc, state.wait);

        if state.wait {
            break;
        }
    }

    println!("\nFinal state:");
    let state = cpu.get_state();
    println!("  IAR: 0x{:04X}", state.iar);
    println!("  ACC: 0x{:04X}", state.acc);
    println!("  Steps: {}", state.instruction_count);

    // Check printer output
    let printer_device = cpu.get_device_mut_ref(2).unwrap();
    let printer = printer_device
        .as_any()
        .downcast_ref::<DeviceConsolePrinter>()
        .unwrap();

    let output = printer.get_output();
    println!("  Printer output: {:?}", output);

    assert_eq!(output, "A");
}
