//! Control flow instruction decoder module.
//!
//! This module decodes control flow instructions including:
//! - Basic control: NOP, HALT, DI, EI
//! - Jumps: JP nn, JP cc,nn, JP (HL)
//! - Relative jumps: JR r8, JR cc,r8
//! - Calls: CALL nn, CALL cc,nn
//! - Returns: RET, RET cc, RETI

use crate::instructions::{Instruction, JumpTest};

/// Decode control flow instructions (jumps, calls, returns, and basic control)
///
/// This includes:
/// - Basic control: NOP, HALT, DI, EI
/// - Jumps: JP nn, JP cc,nn, JP (HL)
/// - Relative jumps: JR r8, JR cc,r8
/// - Calls: CALL nn, CALL cc,nn
/// - Returns: RET, RET cc, RETI
pub fn decode(byte: u8) -> Option<Instruction> {
    match byte {
        // ===== BASIC CONTROL =====
        0x00 => Some(Instruction::NOP),
        0x10 => Some(Instruction::STOP),
        0x76 => Some(Instruction::HALT),
        0xF3 => Some(Instruction::DI),
        0xFB => Some(Instruction::EI),

        // ===== ROTATE ACCUMULATOR =====
        0x07 => Some(Instruction::RLCA),
        0x0F => Some(Instruction::RRCA),
        0x17 => Some(Instruction::RLA),
        0x1F => Some(Instruction::RRA),

        // ===== MISCELLANEOUS ARITHMETIC =====
        0x27 => Some(Instruction::DAA),
        0x2F => Some(Instruction::CPL),
        0x37 => Some(Instruction::SCF),
        0x3F => Some(Instruction::CCF),

        // ===== JUMPS (JP) =====
        0xC3 => Some(Instruction::JP(JumpTest::Always)),
        0xC2 | 0xCA | 0xD2 | 0xDA => JumpTest::from_bits(byte).map(Instruction::JP),
        0xE9 => Some(Instruction::JP_HL),

        // ===== RELATIVE JUMPS (JR) =====
        0x18 => Some(Instruction::JR(JumpTest::Always)),
        0x20 | 0x28 | 0x30 | 0x38 => JumpTest::from_bits(byte).map(Instruction::JR),

        // ===== CALLS =====
        0xCD => Some(Instruction::CALL(JumpTest::Always)),
        0xC4 | 0xCC | 0xD4 | 0xDC => JumpTest::from_bits(byte).map(Instruction::CALL),

        // ===== RETURNS =====
        0xC9 => Some(Instruction::RET(JumpTest::Always)),
        0xC0 | 0xC8 | 0xD0 | 0xD8 => JumpTest::from_bits(byte).map(Instruction::RET),
        0xD9 => Some(Instruction::RETI),

        // ===== RESTARTS (RST) =====
        0xC7 => Some(Instruction::RST(0x00)),
        0xCF => Some(Instruction::RST(0x08)),
        0xD7 => Some(Instruction::RST(0x10)),
        0xDF => Some(Instruction::RST(0x18)),
        0xE7 => Some(Instruction::RST(0x20)),
        0xEF => Some(Instruction::RST(0x28)),
        0xF7 => Some(Instruction::RST(0x30)),
        0xFF => Some(Instruction::RST(0x38)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_control() {
        assert!(matches!(decode(0x00), Some(Instruction::NOP)));
        assert!(matches!(decode(0x10), Some(Instruction::STOP)));
        assert!(matches!(decode(0x76), Some(Instruction::HALT)));
        assert!(matches!(decode(0xF3), Some(Instruction::DI)));
        assert!(matches!(decode(0xFB), Some(Instruction::EI)));
    }

    #[test]
    fn test_rotate_accumulator() {
        assert!(matches!(decode(0x07), Some(Instruction::RLCA)));
        assert!(matches!(decode(0x0F), Some(Instruction::RRCA)));
        assert!(matches!(decode(0x17), Some(Instruction::RLA)));
        assert!(matches!(decode(0x1F), Some(Instruction::RRA)));
    }

    #[test]
    fn test_misc_arithmetic() {
        assert!(matches!(decode(0x27), Some(Instruction::DAA)));
        assert!(matches!(decode(0x2F), Some(Instruction::CPL)));
        assert!(matches!(decode(0x37), Some(Instruction::SCF)));
        assert!(matches!(decode(0x3F), Some(Instruction::CCF)));
    }

    #[test]
    fn test_jumps() {
        assert!(matches!(
            decode(0xC3),
            Some(Instruction::JP(JumpTest::Always))
        ));
        assert!(matches!(
            decode(0xC2),
            Some(Instruction::JP(JumpTest::NotZero))
        ));
        assert!(matches!(
            decode(0xCA),
            Some(Instruction::JP(JumpTest::Zero))
        ));
        assert!(matches!(
            decode(0xD2),
            Some(Instruction::JP(JumpTest::NotCarry))
        ));
        assert!(matches!(
            decode(0xDA),
            Some(Instruction::JP(JumpTest::Carry))
        ));
        assert!(matches!(decode(0xE9), Some(Instruction::JP_HL)));
    }

    #[test]
    fn test_relative_jumps() {
        assert!(matches!(
            decode(0x18),
            Some(Instruction::JR(JumpTest::Always))
        ));
        assert!(matches!(
            decode(0x20),
            Some(Instruction::JR(JumpTest::NotZero))
        ));
        assert!(matches!(
            decode(0x28),
            Some(Instruction::JR(JumpTest::Zero))
        ));
        assert!(matches!(
            decode(0x30),
            Some(Instruction::JR(JumpTest::NotCarry))
        ));
        assert!(matches!(
            decode(0x38),
            Some(Instruction::JR(JumpTest::Carry))
        ));
    }

    #[test]
    fn test_calls() {
        assert!(matches!(
            decode(0xCD),
            Some(Instruction::CALL(JumpTest::Always))
        ));
        assert!(matches!(
            decode(0xC4),
            Some(Instruction::CALL(JumpTest::NotZero))
        ));
        assert!(matches!(
            decode(0xCC),
            Some(Instruction::CALL(JumpTest::Zero))
        ));
        assert!(matches!(
            decode(0xD4),
            Some(Instruction::CALL(JumpTest::NotCarry))
        ));
        assert!(matches!(
            decode(0xDC),
            Some(Instruction::CALL(JumpTest::Carry))
        ));
    }

    #[test]
    fn test_returns() {
        assert!(matches!(
            decode(0xC9),
            Some(Instruction::RET(JumpTest::Always))
        ));
        assert!(matches!(
            decode(0xC0),
            Some(Instruction::RET(JumpTest::NotZero))
        ));
        assert!(matches!(
            decode(0xC8),
            Some(Instruction::RET(JumpTest::Zero))
        ));
        assert!(matches!(
            decode(0xD0),
            Some(Instruction::RET(JumpTest::NotCarry))
        ));
        assert!(matches!(
            decode(0xD8),
            Some(Instruction::RET(JumpTest::Carry))
        ));
    }

    #[test]
    fn test_reti() {
        assert!(matches!(decode(0xD9), Some(Instruction::RETI)));
    }

    #[test]
    fn test_rst_instructions() {
        assert!(matches!(decode(0xC7), Some(Instruction::RST(0x00))));
        assert!(matches!(decode(0xCF), Some(Instruction::RST(0x08))));
        assert!(matches!(decode(0xD7), Some(Instruction::RST(0x10))));
        assert!(matches!(decode(0xDF), Some(Instruction::RST(0x18))));
        assert!(matches!(decode(0xE7), Some(Instruction::RST(0x20))));
        assert!(matches!(decode(0xEF), Some(Instruction::RST(0x28))));
        assert!(matches!(decode(0xF7), Some(Instruction::RST(0x30))));
        assert!(matches!(decode(0xFF), Some(Instruction::RST(0x38))));
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(decode(0x80).is_none());
    }
}
