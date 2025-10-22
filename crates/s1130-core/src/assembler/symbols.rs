//! Symbol Table Management
//!
//! Manages labels and their addresses during assembly.

use crate::error::AssemblerError;
use std::collections::HashMap;

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;

/// Symbol table for labels and constants
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Map of symbol names to addresses
    symbols: HashMap<String, u16>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Define a new symbol
    pub fn define(&mut self, name: &str, address: u16) -> Result<()> {
        if self.symbols.contains_key(name) {
            return Err(AssemblerError::DuplicateLabel(name.to_string()));
        }

        self.symbols.insert(name.to_string(), address);
        Ok(())
    }

    /// Look up a symbol
    pub fn lookup(&self, name: &str) -> Option<u16> {
        self.symbols.get(name).copied()
    }

    /// Check if symbol exists
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Clear all symbols
    pub fn clear(&mut self) {
        self.symbols.clear();
    }

    /// Get all symbols
    pub fn get_all(&self) -> HashMap<String, u16> {
        self.symbols.clone()
    }

    /// Get number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut table = SymbolTable::new();
        table.define("START", 0x100).unwrap();

        assert_eq!(table.lookup("START"), Some(0x100));
        assert_eq!(table.lookup("NOTFOUND"), None);
    }

    #[test]
    fn test_duplicate_label() {
        let mut table = SymbolTable::new();
        table.define("LABEL", 0x100).unwrap();

        let result = table.define("LABEL", 0x200);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut table = SymbolTable::new();
        table.define("A", 100).unwrap();
        table.define("B", 200).unwrap();

        assert_eq!(table.len(), 2);

        table.clear();

        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut table = SymbolTable::new();
        table.define("EXISTS", 100).unwrap();

        assert!(table.contains("EXISTS"));
        assert!(!table.contains("NOTFOUND"));
    }
}
