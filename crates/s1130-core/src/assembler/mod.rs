//! IBM 1130 Assembler
//!
//! This module implements a two-pass assembler for IBM 1130 assembly language.
//! It supports the full instruction set, pseudo-ops, labels, and expressions.

pub mod lexer;
pub mod parser;
pub mod symbols;

use crate::error::AssemblerError;
use std::collections::HashMap;

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;

/// Assembled program output
#[derive(Debug, Clone)]
pub struct AssembledProgram {
    /// Assembled binary words
    pub words: Vec<u16>,

    /// Starting address (from ORG or default 0)
    pub origin: u16,

    /// Symbol table (for debugging)
    pub symbols: HashMap<String, u16>,

    /// Entry point (from END directive or None)
    pub entry_point: Option<u16>,
}

/// Two-pass assembler
pub struct Assembler {
    /// Symbol table
    symbols: symbols::SymbolTable,

    /// Current location counter
    location_counter: u16,

    /// Origin address
    origin: u16,

    /// Entry point
    entry_point: Option<u16>,
}

impl Assembler {
    /// Create a new assembler
    pub fn new() -> Self {
        Self {
            symbols: symbols::SymbolTable::new(),
            location_counter: 0,
            origin: 0,
            entry_point: None,
        }
    }

    /// Assemble source code into binary
    pub fn assemble(&mut self, source: &str) -> Result<AssembledProgram> {
        // Reset state
        self.symbols.clear();
        self.location_counter = 0;
        self.origin = 0;
        self.entry_point = None;

        // Parse source into lines
        let lines = parser::parse_source(source)?;

        // Pass 1: Build symbol table
        self.pass1(&lines)?;

        // Pass 2: Generate code
        let words = self.pass2(&lines)?;

        Ok(AssembledProgram {
            words,
            origin: self.origin,
            symbols: self.symbols.get_all(),
            entry_point: self.entry_point,
        })
    }

    /// Pass 1: Build symbol table and calculate addresses
    fn pass1(&mut self, lines: &[parser::ParsedLine]) -> Result<()> {
        self.location_counter = self.origin;

        for (line_num, line) in lines.iter().enumerate() {
            // Process label if present
            if let Some(ref label) = line.label {
                self.symbols
                    .define(label, self.location_counter)
                    .map_err(|e| AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: e.to_string(),
                    })?;
            }

            // Update location counter based on instruction/pseudo-op
            match &line.operation {
                parser::Operation::Instruction(_) => {
                    // Instructions are 1 or 2 words
                    let size = self.get_instruction_size(&line.operation)?;
                    self.location_counter = self.location_counter.wrapping_add(size);
                }
                parser::Operation::PseudoOp(pseudo) => {
                    self.process_pseudo_pass1(pseudo, &line.operand, line_num)?;
                }
                parser::Operation::None => {}
            }
        }

        Ok(())
    }

    /// Pass 2: Generate machine code
    fn pass2(&mut self, lines: &[parser::ParsedLine]) -> Result<Vec<u16>> {
        let mut words = Vec::new();
        self.location_counter = self.origin;

        for (line_num, line) in lines.iter().enumerate() {
            match &line.operation {
                parser::Operation::Instruction(instr) => {
                    let encoded = self.encode_instruction(instr, &line.operand, line_num)?;
                    words.extend_from_slice(&encoded);
                    self.location_counter =
                        self.location_counter.wrapping_add(encoded.len() as u16);
                }
                parser::Operation::PseudoOp(pseudo) => {
                    let data = self.process_pseudo_pass2(pseudo, &line.operand, line_num)?;
                    words.extend_from_slice(&data);
                }
                parser::Operation::None => {}
            }
        }

        Ok(words)
    }

    /// Get instruction size in words
    fn get_instruction_size(&self, op: &parser::Operation) -> Result<u16> {
        match op {
            parser::Operation::Instruction(instr) => {
                use crate::instructions::OpCode;

                let opcode = match instr.as_str() {
                    "LD" => OpCode::LD,
                    "LDD" => OpCode::LDD,
                    "STO" => OpCode::STO,
                    "STD" => OpCode::STD,
                    "A" => OpCode::A,
                    "AD" => OpCode::AD,
                    "S" => OpCode::S,
                    "SD" => OpCode::SD,
                    "M" => OpCode::M,
                    "D" => OpCode::D,
                    "AND" => OpCode::AND,
                    "OR" => OpCode::OR,
                    "EOR" => OpCode::EOR,
                    "SLA" => OpCode::SLA,
                    "SLCA" => OpCode::SLCA,
                    "SRA" => OpCode::SRA,
                    "SRT" => OpCode::SRT,
                    "BSI" => OpCode::BSI,
                    "BC" => OpCode::BC,
                    "BSC" => OpCode::BSC,
                    "LDX" => OpCode::LDX,
                    "STX" => OpCode::STX,
                    "MDX" => OpCode::MDX,
                    "WAIT" => OpCode::WAIT,
                    "LDS" => OpCode::LDS,
                    "STS" => OpCode::STS,
                    "XIO" => OpCode::XIO,
                    "SDS" => OpCode::SDS,
                    _ => {
                        return Err(AssemblerError::SyntaxError {
                            line: 0,
                            message: format!("Unknown instruction: {}", instr),
                        })
                    }
                };

                Ok(if opcode.is_long_format() { 2 } else { 1 })
            }
            _ => Ok(0),
        }
    }

    /// Process pseudo-op in pass 1
    fn process_pseudo_pass1(
        &mut self,
        pseudo: &str,
        operand: &Option<String>,
        line_num: usize,
    ) -> Result<()> {
        match pseudo {
            "ORG" => {
                // Update location counter and origin for pass 1
                if let Some(ref addr_str) = operand {
                    let addr = self.parse_expression(addr_str, line_num)?;
                    self.location_counter = addr;
                    self.origin = addr;
                }
            }
            "DC" => {
                // Define constant - advances location by 1
                self.location_counter = self.location_counter.wrapping_add(1);
            }
            "BSS" => {
                // Block started by symbol - reserve space
                if let Some(ref size_str) = operand {
                    let size = self.parse_expression(size_str, line_num)?;
                    self.location_counter = self.location_counter.wrapping_add(size);
                }
            }
            "END" => {
                // End of assembly
            }
            "EQU" => {
                // Equate - handled during label processing
            }
            _ => {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Unknown pseudo-op: {}", pseudo),
                });
            }
        }
        Ok(())
    }

    /// Process pseudo-op in pass 2
    fn process_pseudo_pass2(
        &mut self,
        pseudo: &str,
        operand: &Option<String>,
        line_num: usize,
    ) -> Result<Vec<u16>> {
        match pseudo {
            "ORG" => {
                if let Some(ref addr_str) = operand {
                    let addr = self.parse_expression(addr_str, line_num)?;
                    self.location_counter = addr;
                    self.origin = addr;
                }
                Ok(vec![])
            }
            "DC" => {
                if let Some(ref value_str) = operand {
                    let value = self.parse_expression(value_str, line_num)?;
                    self.location_counter = self.location_counter.wrapping_add(1);
                    Ok(vec![value])
                } else {
                    Err(AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: "DC requires an operand".to_string(),
                    })
                }
            }
            "BSS" => {
                if let Some(ref size_str) = operand {
                    let size = self.parse_expression(size_str, line_num)?;
                    self.location_counter = self.location_counter.wrapping_add(size);
                    Ok(vec![0; size as usize])
                } else {
                    Err(AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: "BSS requires a size operand".to_string(),
                    })
                }
            }
            "END" => {
                if let Some(ref entry_str) = operand {
                    let entry = self.parse_expression(entry_str, line_num)?;
                    self.entry_point = Some(entry);
                }
                Ok(vec![])
            }
            "EQU" => {
                // EQU is handled during symbol definition
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    /// Encode an instruction to machine code
    fn encode_instruction(
        &self,
        mnemonic: &str,
        operand: &Option<String>,
        line_num: usize,
    ) -> Result<Vec<u16>> {
        use crate::instructions::OpCode;

        // Map mnemonic to opcode
        let opcode = match mnemonic {
            "LD" => OpCode::LD as u16,
            "LDD" => OpCode::LDD as u16,
            "STO" => OpCode::STO as u16,
            "STD" => OpCode::STD as u16,
            "A" => OpCode::A as u16,
            "AD" => OpCode::AD as u16,
            "S" => OpCode::S as u16,
            "SD" => OpCode::SD as u16,
            "M" => OpCode::M as u16,
            "D" => OpCode::D as u16,
            "AND" => OpCode::AND as u16,
            "OR" => OpCode::OR as u16,
            "EOR" => OpCode::EOR as u16,
            "SLA" => OpCode::SLA as u16,
            "SLCA" => OpCode::SLCA as u16,
            "SRA" => OpCode::SRA as u16,
            "SRT" => OpCode::SRT as u16,
            "BSI" => OpCode::BSI as u16,
            "BC" => OpCode::BC as u16,
            "BSC" => OpCode::BSC as u16,
            "LDX" => OpCode::LDX as u16,
            "STX" => OpCode::STX as u16,
            "MDX" => OpCode::MDX as u16,
            "WAIT" => OpCode::WAIT as u16,
            "LDS" => OpCode::LDS as u16,
            "STS" => OpCode::STS as u16,
            "XIO" => OpCode::XIO as u16,
            "SDS" => OpCode::SDS as u16,
            _ => {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Unknown instruction: {}", mnemonic),
                });
            }
        };

        // Parse operand if present
        // Note: LDX/STX/MDX have reversed operand format: "tag,address" not "address,tag"
        let (displacement, tag, indirect) = if let Some(ref op_str) = operand {
            if matches!(mnemonic, "LDX" | "STX" | "MDX") {
                self.parse_index_operand(op_str, line_num)?
            } else {
                self.parse_operand(op_str, line_num)?
            }
        } else {
            (0, 0, false)
        };

        // Encode based on format
        let is_long = matches!(
            mnemonic,
            "LD" | "LDD"
                | "STO"
                | "STD"
                | "A"
                | "AD"
                | "S"
                | "SD"
                | "M"
                | "D"
                | "AND"
                | "OR"
                | "EOR"
                | "BSI"
                | "LDX"
                | "STX"
                | "MDX"
                | "LDS"
                | "STS"
                | "XIO"
        );

        if is_long {
            // Long format: opcode + tag + indirect + displacement word
            let word1 = (opcode << 8) | ((tag as u16) << 6) | (if indirect { 0x20 } else { 0 });
            Ok(vec![word1, displacement])
        } else {
            // Short format: opcode + tag + indirect + 5-bit address
            let word1 = (opcode << 8)
                | ((tag as u16) << 6)
                | (if indirect { 0x20 } else { 0 })
                | (displacement & 0x1F);
            Ok(vec![word1])
        }
    }

    /// Parse operand string into (displacement, tag, indirect)
    fn parse_operand(&self, operand: &str, line_num: usize) -> Result<(u16, u8, bool)> {
        let operand = operand.trim();

        // Check for indirect addressing: /address or *address
        let (indirect, operand) = if operand.starts_with('/') || operand.starts_with('*') {
            (true, &operand[1..])
        } else {
            (false, operand)
        };

        // Check for index register: address,1 or address,2 or address,3
        let (address_str, tag) = if let Some(comma_pos) = operand.rfind(',') {
            let addr = &operand[..comma_pos];
            let tag_str = operand[comma_pos + 1..].trim();
            let tag = tag_str
                .parse::<u8>()
                .map_err(|_| AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Invalid index register: {}", tag_str),
                })?;
            if tag > 3 {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Index register must be 0-3, got {}", tag),
                });
            }
            (addr, tag)
        } else {
            (operand, 0)
        };

        // Parse address expression
        let displacement = self.parse_expression(address_str, line_num)?;

        Ok((displacement, tag, indirect))
    }

    /// Parse index register operand (format: "tag,address" for LDX/STX/MDX)
    fn parse_index_operand(&self, operand: &str, line_num: usize) -> Result<(u16, u8, bool)> {
        let operand = operand.trim();

        // Check for indirect addressing: /address or *address
        let (indirect, operand) = if operand.starts_with('/') || operand.starts_with('*') {
            (true, &operand[1..])
        } else {
            (false, operand)
        };

        // For index instructions, format is "tag,address" (reversed from normal)
        if let Some(comma_pos) = operand.find(',') {
            let tag_str = operand[..comma_pos].trim();
            let address_str = &operand[comma_pos + 1..].trim();

            let tag = tag_str
                .parse::<u8>()
                .map_err(|_| AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Invalid index register: {}", tag_str),
                })?;

            if tag > 3 {
                return Err(AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Index register must be 0-3, got {}", tag),
                });
            }

            let displacement = self.parse_expression(address_str, line_num)?;
            Ok((displacement, tag, indirect))
        } else {
            // No comma - just an address with tag=0
            let displacement = self.parse_expression(operand, line_num)?;
            Ok((displacement, 0, indirect))
        }
    }

    /// Parse numeric expression (supports decimal, hex, octal, and symbols)
    fn parse_expression(&self, expr: &str, line_num: usize) -> Result<u16> {
        let expr = expr.trim();

        // Check if it's a symbol
        if let Some(value) = self.symbols.lookup(expr) {
            return Ok(value);
        }

        // Parse numeric literal
        if expr.starts_with("0X") || expr.starts_with("0x") {
            // Hexadecimal
            u16::from_str_radix(&expr[2..], 16).map_err(|_| AssemblerError::SyntaxError {
                line: line_num + 1,
                message: format!("Invalid hex literal: {}", expr),
            })
        } else if expr.starts_with('0') && expr.len() > 1 {
            // Octal
            u16::from_str_radix(&expr[1..], 8).map_err(|_| AssemblerError::SyntaxError {
                line: line_num + 1,
                message: format!("Invalid octal literal: {}", expr),
            })
        } else {
            // Decimal (or try as symbol first)
            expr.parse::<u16>()
                .map_err(|_| AssemblerError::SyntaxError {
                    line: line_num + 1,
                    message: format!("Undefined symbol or invalid number: {}", expr),
                })
        }
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}
