//! Error types for the S1130 emulator

use thiserror::Error;

/// Errors that can occur during CPU operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CpuError {
    /// Invalid instruction encountered
    #[error("Invalid instruction at address {0:#06x}")]
    InvalidInstruction(u16),

    /// Memory access violation
    #[error("Memory access violation at address {0:#06x}")]
    MemoryViolation(u16),

    /// Device error
    #[error("Device error: {0}")]
    DeviceError(String),

    /// Execution halted by WAIT instruction
    #[error("Execution halted by WAIT instruction")]
    WaitState,

    /// No instruction loaded for execution
    #[error("No instruction loaded for execution")]
    NoInstructionLoaded,
}

/// Errors that can occur during instruction execution
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InstructionError {
    /// Invalid opcode
    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    /// Missing displacement word for long format instruction
    #[error("Missing displacement word for long format instruction")]
    MissingDisplacement,

    /// Memory access error during instruction execution
    #[error("Memory access error: {0}")]
    MemoryError(#[from] CpuError),
}

/// Errors that can occur during assembly
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssemblerError {
    /// Syntax error in assembly source
    #[error("Syntax error on line {line}: {message}")]
    SyntaxError {
        /// Line number (1-indexed)
        line: usize,
        /// Error message
        message: String,
    },

    /// Undefined symbol reference
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    /// Duplicate label definition
    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),

    /// Invalid address
    #[error("Invalid address: {0:#06x}")]
    InvalidAddress(u16),

    /// Value out of range
    #[error("Value out of range: {0}")]
    ValueOutOfRange(i32),
}

/// Errors that can occur during device operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DeviceError {
    /// Device not ready
    #[error("Device not ready")]
    NotReady,

    /// Unsupported device function
    #[error("Unsupported function: {0:?}")]
    UnsupportedFunction(u8),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
}

/// Result type for CPU operations
pub type Result<T> = std::result::Result<T, CpuError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CpuError::MemoryViolation(0x1234);
        assert_eq!(err.to_string(), "Memory access violation at address 0x1234");
    }

    #[test]
    fn test_instruction_error_from_cpu_error() {
        let cpu_err = CpuError::MemoryViolation(0x5678);
        let instr_err: InstructionError = cpu_err.into();
        assert_eq!(
            instr_err.to_string(),
            "Memory access error: Memory access violation at address 0x5678"
        );
    }

    #[test]
    fn test_assembler_syntax_error() {
        let err = AssemblerError::SyntaxError {
            line: 42,
            message: "Missing operand".to_string(),
        };
        assert_eq!(err.to_string(), "Syntax error on line 42: Missing operand");
    }
}
