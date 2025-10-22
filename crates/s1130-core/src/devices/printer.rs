//! Console Printer Device
//!
//! This device emulates a console printer for the IBM 1130.
//! It's a character-mode device that receives characters for output.
//!
//! Device code: 2 (standard console printer)
//!
//! Operations:
//! - Sense: Check if printer is ready
//! - Write: Write a character to printer

use crate::devices::{Device, DeviceFunction, Iocc};
use crate::error::CpuError;

/// Console Printer Device
///
/// This is a character-mode output device. Programs use XIO to:
/// 1. Sense if printer is ready
/// 2. Write characters one at a time
pub struct DeviceConsolePrinter {
    /// Output buffer (characters that have been printed)
    output_buffer: Vec<u16>,

    /// Device status flags
    busy: bool,
}

impl DeviceConsolePrinter {
    /// Create a new console printer device
    pub fn new() -> Self {
        Self {
            output_buffer: Vec::new(),
            busy: false,
        }
    }

    /// Get the printed output as a string
    ///
    /// Converts the output buffer to a String for inspection/testing.
    pub fn get_output(&self) -> String {
        self.output_buffer
            .iter()
            .map(|&ch| char::from_u32(ch as u32).unwrap_or('?'))
            .collect()
    }

    /// Get the output buffer as a slice
    pub fn get_output_raw(&self) -> &[u16] {
        &self.output_buffer
    }

    /// Clear the output buffer
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }

    /// Write a character to the output
    fn write_char(&mut self, ch: u16) {
        self.output_buffer.push(ch);
    }
}

impl Default for DeviceConsolePrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for DeviceConsolePrinter {
    fn device_code(&self) -> u8 {
        2 // Console printer
    }

    fn device_name(&self) -> &'static str {
        "Console Printer"
    }

    fn execute_iocc(&mut self, iocc: &Iocc, memory: &mut [u16]) -> Result<(), CpuError> {
        match iocc.function {
            DeviceFunction::Sense => {
                // Sense operation: return status in WCA location
                // Bit 15 (LSB) = 1 if printer ready (always ready in this simple impl)
                let status = 1; // Always ready
                if (iocc.wca as usize) < memory.len() {
                    memory[iocc.wca as usize] = status;
                }
                Ok(())
            }

            DeviceFunction::Write => {
                // Write operation: write one character from WCA location
                if (iocc.wca as usize) < memory.len() {
                    let ch = memory[iocc.wca as usize];
                    self.write_char(ch);
                    Ok(())
                } else {
                    Err(CpuError::DeviceError(
                        "Printer: Invalid memory address".to_string(),
                    ))
                }
            }

            _ => Err(CpuError::DeviceError(format!(
                "Printer: Unsupported function {:?}",
                iocc.function
            ))),
        }
    }

    fn is_busy(&self) -> bool {
        self.busy
    }

    fn reset(&mut self) {
        self.output_buffer.clear();
        self.busy = false;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printer_creation() {
        let printer = DeviceConsolePrinter::new();
        assert_eq!(printer.device_code(), 2);
        assert_eq!(printer.device_name(), "Console Printer");
        assert_eq!(printer.get_output(), "");
    }

    #[test]
    fn test_sense_operation() {
        let mut printer = DeviceConsolePrinter::new();
        let mut memory = vec![0u16; 100];

        let iocc = Iocc {
            wca: 50,
            device_code: 2,
            function: DeviceFunction::Sense,
            modifiers: 0,
        };

        printer.execute_iocc(&iocc, &mut memory).unwrap();
        assert_eq!(memory[50], 1); // Always ready
    }

    #[test]
    fn test_write_operation() {
        let mut printer = DeviceConsolePrinter::new();
        let mut memory = vec![0u16; 100];

        memory[50] = b'A' as u16;

        let iocc = Iocc {
            wca: 50,
            device_code: 2,
            function: DeviceFunction::Write,
            modifiers: 0,
        };

        printer.execute_iocc(&iocc, &mut memory).unwrap();
        assert_eq!(printer.get_output(), "A");
    }

    #[test]
    fn test_write_multiple_chars() {
        let mut printer = DeviceConsolePrinter::new();
        let mut memory = vec![0u16; 100];

        // Write "HELLO"
        for ch in "HELLO".chars() {
            memory[50] = ch as u16;
            let iocc = Iocc {
                wca: 50,
                device_code: 2,
                function: DeviceFunction::Write,
                modifiers: 0,
            };
            printer.execute_iocc(&iocc, &mut memory).unwrap();
        }

        assert_eq!(printer.get_output(), "HELLO");
    }

    #[test]
    fn test_clear_output() {
        let mut printer = DeviceConsolePrinter::new();
        printer.write_char(b'A' as u16);
        assert_eq!(printer.get_output(), "A");

        printer.clear_output();
        assert_eq!(printer.get_output(), "");
    }
}
