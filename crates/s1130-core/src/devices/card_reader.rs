//! IBM 2501 Card Reader Device
//!
//! This device emulates an IBM 2501 Card Reader for the IBM 1130.
//! It's a block-mode device that uses DMA transfers via IOCC.
//!
//! Device code: 9 (0x09)
//!
//! Operations:
//! - Sense: Check device status
//! - InitRead: Start reading a card (0-80 words) into memory
//!
//! Status word bits:
//! - 0x1000: Last card (interrupt 4)
//! - 0x0800: Operation complete (interrupt 4)
//! - 0x0002: Busy (read in progress)
//! - 0x0001: Not ready or busy

use crate::devices::{Device, DeviceFunction, Iocc};
use crate::error::CpuError;
use std::collections::VecDeque;

/// Card data structure
///
/// IBM 1130 cards hold 80 columns of 16-bit words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Card columns (80 words)
    pub columns: [u16; 80],
}

impl Card {
    /// Create a blank card
    pub fn new() -> Self {
        Self { columns: [0; 80] }
    }

    /// Create a card from data
    pub fn from_data(data: &[u16]) -> Self {
        let mut card = Self::new();
        let len = data.len().min(80);
        card.columns[..len].copy_from_slice(&data[..len]);
        card
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

/// IBM 2501 Card Reader Device
///
/// This is a block-mode device that reads punched cards.
/// Cards are queued in a hopper and read one at a time into memory.
pub struct Device2501 {
    /// Card hopper (cards waiting to be read)
    hopper: VecDeque<Card>,

    /// Read operation state
    read_in_progress: bool,
    operation_complete: bool,
    last_card: bool,

    /// Memory address for current read operation
    read_address: u16,

    /// Number of words to read (typically 80)
    read_count: u16,
}

impl Device2501 {
    /// Create a new 2501 Card Reader device
    pub fn new() -> Self {
        Self {
            hopper: VecDeque::new(),
            read_in_progress: false,
            operation_complete: false,
            last_card: false,
            read_address: 0,
            read_count: 0,
        }
    }

    /// Load a card into the hopper
    ///
    /// This simulates placing a card in the card reader's input hopper.
    ///
    /// # Arguments
    /// * `card` - The card to load
    pub fn load_card(&mut self, card: Card) {
        self.hopper.push_back(card);
    }

    /// Load multiple cards into the hopper
    ///
    /// # Arguments
    /// * `cards` - The cards to load
    pub fn load_cards(&mut self, cards: Vec<Card>) {
        for card in cards {
            self.hopper.push_back(card);
        }
    }

    /// Check if hopper is empty
    pub fn is_empty(&self) -> bool {
        self.hopper.is_empty()
    }

    /// Get number of cards in hopper
    pub fn card_count(&self) -> usize {
        self.hopper.len()
    }

    /// Execute a read operation (called after InitRead)
    ///
    /// This transfers card data to memory. In the real hardware, this would
    /// happen asynchronously. For our emulator, we do it immediately.
    ///
    /// # Arguments
    /// * `memory` - Mutable reference to CPU memory
    ///
    /// # Returns
    /// * `true` if a card was read successfully
    /// * `false` if no card was available
    pub fn execute_read(&mut self, memory: &mut [u16]) -> bool {
        if !self.read_in_progress || self.hopper.is_empty() {
            return false;
        }

        // Dequeue the card
        if let Some(card) = self.hopper.pop_front() {
            // Transfer data to memory
            let count = self.read_count.min(80) as usize;
            let addr = self.read_address as usize;

            if addr + count <= memory.len() {
                memory[addr..addr + count].copy_from_slice(&card.columns[..count]);

                // Update status flags
                self.last_card = self.hopper.is_empty();
                self.operation_complete = true;
                self.read_in_progress = false;

                return true;
            }
        }

        false
    }

    /// Get device status word
    fn get_status(&self) -> u16 {
        let mut status = 0u16;

        // Bit 0x1000: Last card
        if self.last_card {
            status |= 0x1000;
        }

        // Bit 0x0800: Operation complete
        if self.operation_complete {
            status |= 0x0800;
        }

        // Bit 0x0002: Busy (read in progress)
        if self.read_in_progress {
            status |= 0x0002;
        }

        // Bit 0x0001: Not ready (hopper empty and not completing)
        if self.hopper.is_empty() && !self.operation_complete {
            status |= 0x0001;
        }

        status
    }

    /// Clear status flags (called by Sense with modifier bit 0)
    fn clear_status(&mut self) {
        self.operation_complete = false;
        self.last_card = false;
        // Note: Interrupts would be deactivated here in real implementation
    }
}

impl Default for Device2501 {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Device2501 {
    fn device_code(&self) -> u8 {
        0x09 // 2501 Card Reader
    }

    fn device_name(&self) -> &'static str {
        "2501 Card Reader"
    }

    fn execute_iocc(&mut self, iocc: &Iocc, memory: &mut [u16]) -> Result<(), CpuError> {
        match iocc.function {
            DeviceFunction::Sense => {
                // Sense Device - return status in accumulator
                // Note: In real implementation, status would be written to ACC
                // For now, we'll handle this through the CPU's XIO instruction

                // If modifier bit 0 is set, clear status flags
                if (iocc.modifiers & 0x01) == 0x01 {
                    self.clear_status();
                }

                // Status word will be returned by caller
                Ok(())
            }
            DeviceFunction::InitRead => {
                // Initiate Read - set up for block transfer
                if !self.read_in_progress {
                    // WCA points to word count in memory
                    let wca = iocc.wca as usize;
                    if wca >= memory.len() {
                        return Err(CpuError::InvalidAddress(iocc.wca));
                    }

                    // Read word count from memory
                    // In IBM 1130 IOCC format:
                    // - Negative word count at WCA
                    // - Data starts at WCA+1
                    let word_count = memory[wca] as i16;
                    let count = (-word_count).max(0) as u16;

                    self.read_address = (wca + 1) as u16;
                    self.read_count = count.min(80);
                    self.read_in_progress = true;

                    // Execute the read immediately (synchronous for emulator)
                    self.execute_read(memory);
                }
                Ok(())
            }
            _ => {
                // Unsupported function for this device
                Err(CpuError::InvalidDevice(self.device_code()))
            }
        }
    }

    fn is_busy(&self) -> bool {
        self.read_in_progress
    }

    fn reset(&mut self) {
        self.read_in_progress = false;
        self.operation_complete = false;
        self.last_card = false;
        self.read_address = 0;
        self.read_count = 0;
        // Note: hopper is NOT cleared on reset
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
