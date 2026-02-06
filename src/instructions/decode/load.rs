//! Load instruction decoder module.
//!
//! This module decodes load and store instructions that move data between
//! registers, memory, and immediate values.

use crate::instructions::{
    Instruction, LoadByteSource, LoadByteTarget, LoadType, LoadWordSource, LoadWordTarget,
};

/// Decode load and store instructions
/// These instructions move data between registers, memory, and immediate values
pub fn decode(byte: u8) -> Option<Instruction> {
    // ===== REGISTER-TO-REGISTER LOADS (0x40-0x7F, excluding 0x76 HALT) =====
    // This covers all combinations of 8-bit register loads
    if (0x40..=0x7F).contains(&byte) && byte != 0x76 {
        return decode_register_load(byte);
    }

    // ===== IMMEDIATE BYTE LOADS (LD r, d8) =====
    // Load an 8-bit immediate value into a register
    match byte {
        0x06 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::D8,
            )))
        }
        0x0E => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::D8,
            )))
        }
        0x16 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::D8,
            )))
        }
        0x1E => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::D8,
            )))
        }
        0x26 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::D8,
            )))
        }
        0x2E => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::D8,
            )))
        }
        0x3E => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::D8,
            )))
        }
        0x36 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::D8,
            )))
        }
        _ => {}
    }

    // ===== IMMEDIATE WORD LOADS (LD rr, d16) =====
    // Load a 16-bit immediate value into a register pair
    if matches!(byte, 0x01 | 0x11 | 0x21 | 0x31) {
        if let Some(dst) = LoadWordTarget::from_bits(byte) {
            return Some(Instruction::LD(LoadType::Word(dst, LoadWordSource::D16)));
        }
    }

    // ===== SPECIAL MEMORY LOADS =====
    match byte {
        0x0A => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::BCI,
            )));
        }
        // LD (BC), A - Store A to address pointed to by BC
        0x02 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::BCI,
                LoadByteSource::A,
            )))
        }
        0x08 => {
            return Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::A16I,
                LoadWordSource::SP,
            )))
        }
        // LD (HL+), A - Store A to (HL), then increment HL
        0x22 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI_INC,
                LoadByteSource::A,
            )))
        }
        // LD (HL-), A - Store A to (HL), then decrement HL
        0x32 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI_DEC,
                LoadByteSource::A,
            )))
        }
        // LD A, (DE) - Load A from address pointed to by DE
        0x1A => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::DEI,
            )))
        }
        // LD A, (HL+) - Load from (HL), then increment HL
        0x2A => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLI_INC,
            )))
        }
        // LD A, (HL-) - Load from (HL), then decrement HL
        0x3A => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLI_DEC,
            )))
        }
        // LD (DE), A - Store A to memory address pointed to by DE
        0x12 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::DEI,
                LoadByteSource::A,
            )))
        }
        // LD (a16), A - Store A to absolute 16-bit address
        0xEA => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A16I,
                LoadByteSource::A,
            )))
        }
        // LD A, (a16) - Load A from absolute 16-bit address
        0xFA => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::A16I,
            )))
        }
        // LDH (a8), A - Store A to high RAM (0xFF00 + a8)
        0xE0 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A8I,
                LoadByteSource::A,
            )))
        }
        // LDH A, (a8) - Load A from high RAM (0xFF00 + a8)
        0xF0 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::A8I,
            )))
        }
        // LD (C), A - Store A to high RAM (0xFF00 + C)
        0xE2 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CI,
                LoadByteSource::A,
            )))
        }
        // LD A, (C) - Load A from high RAM (0xFF00 + C)
        0xF2 => {
            return Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::CI,
            )))
        }
        _ => {}
    }

    None
}

/// Decode register-to-register load instructions (0x40-0x7F)
/// The destination is encoded in bits 5-3, source in bits 2-0
fn decode_register_load(byte: u8) -> Option<Instruction> {
    let dst = LoadByteTarget::from_upper_bits(byte)?;
    let src = LoadByteSource::from_lower_bits(byte)?;
    Some(Instruction::LD(LoadType::Byte(dst, src)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_to_register_load() {
        // LD B, C (0x41)
        if let Some(Instruction::LD(LoadType::Byte(dst, src))) = decode(0x41) {
            assert!(matches!(dst, LoadByteTarget::B));
            assert!(matches!(src, LoadByteSource::C));
        } else {
            panic!("Failed to decode LD B, C");
        }

        // LD A, A (0x7F)
        if let Some(Instruction::LD(LoadType::Byte(dst, src))) = decode(0x7F) {
            assert!(matches!(dst, LoadByteTarget::A));
            assert!(matches!(src, LoadByteSource::A));
        } else {
            panic!("Failed to decode LD A, A");
        }
    }

    #[test]
    fn test_halt_not_decoded_as_load() {
        // 0x76 is HALT, not a load instruction
        assert!(decode(0x76).is_none());
    }

    #[test]
    fn test_immediate_byte_loads() {
        // LD A, d8 (0x3E)
        assert!(matches!(
            decode(0x3E),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::D8
            )))
        ));

        // LD (HL), d8 (0x36)
        assert!(matches!(
            decode(0x36),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::D8
            )))
        ));
    }

    #[test]
    fn test_immediate_word_loads() {
        // LD HL, d16 (0x21)
        assert!(matches!(
            decode(0x21),
            Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::D16
            )))
        ));

        // LD SP, d16 (0x31)
        assert!(matches!(
            decode(0x31),
            Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::D16
            )))
        ));
    }

    #[test]
    fn test_special_loads() {
        // LD A, (HL+)
        assert!(matches!(
            decode(0x2A),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLI_INC
            )))
        ));

        // LD A, (HL-)
        assert!(matches!(
            decode(0x3A),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLI_DEC
            )))
        ));

        // LD (DE), A
        assert!(matches!(
            decode(0x12),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::DEI,
                LoadByteSource::A
            )))
        ));
    }

    #[test]
    fn test_high_ram_loads() {
        // LDH (a8), A
        assert!(matches!(
            decode(0xE0),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A8I,
                LoadByteSource::A
            )))
        ));

        // LDH A, (a8)
        assert!(matches!(
            decode(0xF0),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::A8I
            )))
        ));
    }

    #[test]
    fn test_c_indirect_loads() {
        // LD (C), A
        assert!(matches!(
            decode(0xE2),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CI,
                LoadByteSource::A
            )))
        ));

        // LD A, (C)
        assert!(matches!(
            decode(0xF2),
            Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::CI
            )))
        ));
    }

    #[test]
    fn test_invalid_opcodes() {
        // Test some opcodes that aren't load instructions
        assert!(decode(0x00).is_none()); // NOP
        assert!(decode(0xC3).is_none()); // JP
        assert!(decode(0xFF).is_none()); // Invalid
    }
}
