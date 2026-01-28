//! Control flow instructions module.
//!
//! This module defines the target types for jumps, calls, and returns,
//! and provides decoding utilities for conditional jump tests.

#[derive(Debug, Copy, Clone)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpTest {
    /// Decode a JumpTest condition from bits 4-3 of an opcode
    /// Used for conditional jump/call/return instructions
    pub fn from_bits(bits: u8) -> Option<Self> {
        match (bits >> 3) & 0x03 {
            0x00 => Some(JumpTest::NotZero),
            0x01 => Some(JumpTest::Zero),
            0x02 => Some(JumpTest::NotCarry),
            0x03 => Some(JumpTest::Carry),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bits() {
        assert!(matches!(JumpTest::from_bits(0x00), Some(JumpTest::NotZero)));
        assert!(matches!(JumpTest::from_bits(0x08), Some(JumpTest::Zero)));
        assert!(matches!(
            JumpTest::from_bits(0x10),
            Some(JumpTest::NotCarry)
        ));
        assert!(matches!(JumpTest::from_bits(0x18), Some(JumpTest::Carry)));
    }
}
