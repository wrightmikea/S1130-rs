//! CPU Memory Management
//!
//! This module handles memory operations in isolation.
//! All memory access goes through bounds-checked methods.

use crate::error::{CpuError, Result};

/// IBM 1130 Memory
///
/// Word-addressable memory with configurable size.
/// Default size is 32K words (32,768 = 0x8000).
pub struct Memory {
    data: Vec<u16>,
}

impl Memory {
    /// Create memory with default size (32K words)
    pub fn new() -> Self {
        Self::with_size(32768)
    }

    /// Create memory with specific size in words
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// Get memory size in words
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Read word from memory with bounds checking
    ///
    /// # Errors
    ///
    /// Returns `CpuError::MemoryViolation` if address is out of bounds
    pub fn read(&self, address: usize) -> Result<u16> {
        self.data
            .get(address)
            .copied()
            .ok_or(CpuError::MemoryViolation(address as u16))
    }

    /// Write word to memory with bounds checking
    ///
    /// # Errors
    ///
    /// Returns `CpuError::MemoryViolation` if address is out of bounds
    pub fn write(&mut self, address: usize, value: u16) -> Result<()> {
        if address < self.data.len() {
            self.data[address] = value;
            Ok(())
        } else {
            Err(CpuError::MemoryViolation(address as u16))
        }
    }

    /// Read multiple words starting at address
    ///
    /// Returns only valid words, stops at bounds or count limit
    pub fn read_range(&self, address: usize, count: usize) -> Vec<u16> {
        self.data
            .iter()
            .skip(address)
            .take(count)
            .copied()
            .collect()
    }

    /// Write multiple words starting at address
    ///
    /// Stops writing if bounds exceeded
    ///
    /// # Errors
    ///
    /// Returns error if starting address is out of bounds
    pub fn write_range(&mut self, address: usize, values: &[u16]) -> Result<()> {
        if address >= self.data.len() {
            return Err(CpuError::MemoryViolation(address as u16));
        }

        let end = (address + values.len()).min(self.data.len());
        let count = end - address;
        self.data[address..end].copy_from_slice(&values[..count]);
        Ok(())
    }

    /// Clear all memory to zero
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Get direct slice reference (for performance-critical operations)
    ///
    /// Use with caution - bypasses bounds checking
    pub fn as_slice(&self) -> &[u16] {
        &self.data
    }

    /// Get direct mutable slice reference (for performance-critical operations)
    ///
    /// Use with caution - bypasses bounds checking
    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.data
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_new() {
        let mem = Memory::new();
        assert_eq!(mem.size(), 32768);
    }

    #[test]
    fn test_memory_with_size() {
        let mem = Memory::with_size(8192);
        assert_eq!(mem.size(), 8192);
    }

    #[test]
    fn test_memory_read_write() {
        let mut mem = Memory::new();
        mem.write(0x100, 0x1234).unwrap();
        assert_eq!(mem.read(0x100).unwrap(), 0x1234);
    }

    #[test]
    fn test_memory_read_bounds_check() {
        let mem = Memory::with_size(100);
        let result = mem.read(100);
        assert!(result.is_err());
        match result {
            Err(CpuError::MemoryViolation(addr)) => assert_eq!(addr, 100),
            _ => panic!("Expected MemoryViolation"),
        }
    }

    #[test]
    fn test_memory_write_bounds_check() {
        let mut mem = Memory::with_size(100);
        let result = mem.write(100, 0x1234);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_read_range() {
        let mut mem = Memory::new();
        mem.write(0x100, 0x1111).unwrap();
        mem.write(0x101, 0x2222).unwrap();
        mem.write(0x102, 0x3333).unwrap();

        let values = mem.read_range(0x100, 3);
        assert_eq!(values, vec![0x1111, 0x2222, 0x3333]);
    }

    #[test]
    fn test_memory_read_range_at_boundary() {
        let mem = Memory::with_size(10);
        let values = mem.read_range(8, 5); // Request 5, but only 2 available
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_memory_write_range() {
        let mut mem = Memory::new();
        let values = vec![0x1111, 0x2222, 0x3333];
        mem.write_range(0x100, &values).unwrap();

        assert_eq!(mem.read(0x100).unwrap(), 0x1111);
        assert_eq!(mem.read(0x101).unwrap(), 0x2222);
        assert_eq!(mem.read(0x102).unwrap(), 0x3333);
    }

    #[test]
    fn test_memory_write_range_bounds() {
        let mut mem = Memory::with_size(10);
        let values = vec![1, 2, 3, 4, 5];
        let result = mem.write_range(8, &values); // Only 2 slots available
        assert!(result.is_ok()); // Writes what fits

        assert_eq!(mem.read(8).unwrap(), 1);
        assert_eq!(mem.read(9).unwrap(), 2);
        // Values 3, 4, 5 were not written (out of bounds)
    }

    #[test]
    fn test_memory_clear() {
        let mut mem = Memory::with_size(10);
        mem.write(0, 0x1234).unwrap();
        mem.write(5, 0x5678).unwrap();
        mem.write(9, 0xABCD).unwrap();

        mem.clear();

        for i in 0..10 {
            assert_eq!(mem.read(i).unwrap(), 0);
        }
    }

    #[test]
    fn test_memory_as_slice() {
        let mut mem = Memory::with_size(5);
        mem.write(0, 1).unwrap();
        mem.write(1, 2).unwrap();
        mem.write(2, 3).unwrap();

        let slice = mem.as_slice();
        assert_eq!(slice[0], 1);
        assert_eq!(slice[1], 2);
        assert_eq!(slice[2], 3);
    }

    #[test]
    fn test_memory_as_mut_slice() {
        let mut mem = Memory::with_size(5);

        let slice = mem.as_mut_slice();
        slice[0] = 0x1111;
        slice[1] = 0x2222;

        assert_eq!(mem.read(0).unwrap(), 0x1111);
        assert_eq!(mem.read(1).unwrap(), 0x2222);
    }
}
