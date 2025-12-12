use crate::instructions::{Instruction, PrefixTarget};

/// Decode CB-prefixed instructions
/// These are extended instructions for bit operations, rotations, and shifts
pub fn decode(byte: u8) -> Option<Instruction> {
    match byte {
        // ===== RLC (Rotate Left Circular) =====
        0x00 => Some(Instruction::RLC(PrefixTarget::B)),
        0x01 => Some(Instruction::RLC(PrefixTarget::C)),
        0x02 => Some(Instruction::RLC(PrefixTarget::D)),
        0x03 => Some(Instruction::RLC(PrefixTarget::E)),
        0x04 => Some(Instruction::RLC(PrefixTarget::H)),
        0x05 => Some(Instruction::RLC(PrefixTarget::L)),
        0x06 => Some(Instruction::RLC(PrefixTarget::HLI)),
        0x07 => Some(Instruction::RLC(PrefixTarget::A)),

        // TODO: Add remaining CB-prefixed instructions:
        // - RRC (Rotate Right Circular)
        // - RL (Rotate Left through Carry)
        // - RR (Rotate Right through Carry)
        // - SLA (Shift Left Arithmetic)
        // - SRA (Shift Right Arithmetic)
        // - SWAP (Swap nibbles)
        // - SRL (Shift Right Logical)
        // - BIT (Test bit)
        // - RES (Reset bit)
        // - SET (Set bit)
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
    fn test_unimplemented_opcodes() {
        // These should return None until implemented
        assert!(decode(0x08).is_none()); // RRC B (not yet implemented)
        assert!(decode(0xFF).is_none());
    }
}
