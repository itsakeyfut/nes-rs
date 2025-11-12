// Instructions module for 6502 CPU
// This module organizes CPU instructions by semantic grouping

pub mod arithmetic;
pub mod branch;
pub mod compare;
pub mod jump_subroutine;
pub mod load_store;
pub mod logic;
pub mod shift_rotate;
pub mod transfer;

use crate::bus::Bus;
use crate::cpu::addressing::AddressingResult;

impl crate::cpu::Cpu {
    // ========================================
    // Helper Functions
    // ========================================

    /// Helper function to read a value from memory using an addressing result
    ///
    /// If the addressing result contains an immediate value, returns that value.
    /// Otherwise, reads from the address specified in the addressing result.
    #[inline]
    pub(crate) fn read_operand(&self, bus: &Bus, addr_result: &AddressingResult) -> u8 {
        if let Some(value) = addr_result.value {
            value
        } else {
            bus.read(addr_result.address)
        }
    }
}
