//! CB-prefixed instruction decoder module.
//!
//! This module decodes CB-prefixed instructions for bit operations,
//! rotations, and shifts on registers and memory.

use crate::instructions::{Instruction, PrefixTarget};

/// Decode CB-prefixed instructions
/// These are extended instructions for bit operations, rotations, and shifts
pub fn decode(byte: u8) -> Option<Instruction> {
    // CB-prefixed instructions follow a pattern:
    // Bits 7-6: Operation type
    // Bits 5-3: Bit number (for BIT/RES/SET)
    // Bits 2-0: Target register

    let target = decode_prefix_target(byte & 0x07)?;

    match byte {
        // ===== RLC (Rotate Left Circular) 0x00-0x07 =====
        0x00..=0x07 => Some(Instruction::RLC(target)),

        // ===== RRC (Rotate Right Circular) 0x08-0x0F =====
        0x08..=0x0F => Some(Instruction::RRC(target)),

        // ===== RL (Rotate Left through Carry) 0x10-0x17 =====
        0x10..=0x17 => Some(Instruction::RL(target)),

        // ===== RR (Rotate Right through Carry) 0x18-0x1F =====
        0x18..=0x1F => Some(Instruction::RR(target)),

        // ===== SLA (Shift Left Arithmetic) 0x20-0x27 =====
        0x20..=0x27 => Some(Instruction::SLA(target)),

        // ===== SRA (Shift Right Arithmetic) 0x28-0x2F =====
        0x28..=0x2F => Some(Instruction::SRA(target)),

        // ===== SWAP (Swap nibbles) 0x30-0x37 =====
        0x30..=0x37 => Some(Instruction::SWAP(target)),

        // ===== SRL (Shift Right Logical) 0x38-0x3F =====
        0x38..=0x3F => Some(Instruction::SRL(target)),

        // ===== BIT (Test bit) 0x40-0x7F =====
        0x40..=0x7F => {
            let bit = (byte >> 3) & 0x07;
            Some(Instruction::BIT(bit, target))
        }

        // ===== RES (Reset bit) 0x80-0xBF =====
        0x80..=0xBF => {
            let bit = (byte >> 3) & 0x07;
            Some(Instruction::RES(bit, target))
        }

        // ===== SET (Set bit) 0xC0-0xFF =====
        0xC0..=0xFF => {
            let bit = (byte >> 3) & 0x07;
            Some(Instruction::SET(bit, target))
        }
    }
}

/// Decode the target register from the lower 3 bits of a CB-prefixed opcode
fn decode_prefix_target(bits: u8) -> Option<PrefixTarget> {
    match bits & 0x07 {
        0x00 => Some(PrefixTarget::B),
        0x01 => Some(PrefixTarget::C),
        0x02 => Some(PrefixTarget::D),
        0x03 => Some(PrefixTarget::E),
        0x04 => Some(PrefixTarget::H),
        0x05 => Some(PrefixTarget::L),
        0x06 => Some(PrefixTarget::HLI),
        0x07 => Some(PrefixTarget::A),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlc_instructions() {
        assert!(matches!(
            decode(0x00),
            Some(Instruction::RLC(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x07),
            Some(Instruction::RLC(PrefixTarget::A))
        ));
        assert!(matches!(
            decode(0x06),
            Some(Instruction::RLC(PrefixTarget::HLI))
        ));
    }

    #[test]
    fn test_rrc_instructions() {
        assert!(matches!(
            decode(0x08),
            Some(Instruction::RRC(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x0F),
            Some(Instruction::RRC(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_rl_instructions() {
        assert!(matches!(
            decode(0x10),
            Some(Instruction::RL(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x17),
            Some(Instruction::RL(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_rr_instructions() {
        assert!(matches!(
            decode(0x18),
            Some(Instruction::RR(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x1F),
            Some(Instruction::RR(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_sla_instructions() {
        assert!(matches!(
            decode(0x20),
            Some(Instruction::SLA(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x27),
            Some(Instruction::SLA(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_sra_instructions() {
        assert!(matches!(
            decode(0x28),
            Some(Instruction::SRA(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x2F),
            Some(Instruction::SRA(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_swap_instructions() {
        assert!(matches!(
            decode(0x30),
            Some(Instruction::SWAP(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x37),
            Some(Instruction::SWAP(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_srl_instructions() {
        assert!(matches!(
            decode(0x38),
            Some(Instruction::SRL(PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x3F),
            Some(Instruction::SRL(PrefixTarget::A))
        ));
    }

    #[test]
    fn test_bit_instructions() {
        assert!(matches!(
            decode(0x40),
            Some(Instruction::BIT(0, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x47),
            Some(Instruction::BIT(0, PrefixTarget::A))
        ));
        assert!(matches!(
            decode(0x78),
            Some(Instruction::BIT(7, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x7F),
            Some(Instruction::BIT(7, PrefixTarget::A))
        ));
    }

    #[test]
    fn test_res_instructions() {
        assert!(matches!(
            decode(0x80),
            Some(Instruction::RES(0, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0x87),
            Some(Instruction::RES(0, PrefixTarget::A))
        ));
        assert!(matches!(
            decode(0xB8),
            Some(Instruction::RES(7, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0xBF),
            Some(Instruction::RES(7, PrefixTarget::A))
        ));
    }

    #[test]
    fn test_set_instructions() {
        assert!(matches!(
            decode(0xC0),
            Some(Instruction::SET(0, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0xC7),
            Some(Instruction::SET(0, PrefixTarget::A))
        ));
        assert!(matches!(
            decode(0xF8),
            Some(Instruction::SET(7, PrefixTarget::B))
        ));
        assert!(matches!(
            decode(0xFF),
            Some(Instruction::SET(7, PrefixTarget::A))
        ));
    }

    #[test]
    fn test_decode_prefix_target() {
        assert!(matches!(decode_prefix_target(0x00), Some(PrefixTarget::B)));
        assert!(matches!(decode_prefix_target(0x07), Some(PrefixTarget::A)));
        assert!(matches!(
            decode_prefix_target(0x06),
            Some(PrefixTarget::HLI)
        ));
    }
}
