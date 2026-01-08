// Stack operation decoders: PUSH and POP instructions

use crate::instructions::{Instruction, StackTarget};

/// Decode stack instructions (PUSH and POP)
///
/// PUSH opcodes: 0xC5 (BC), 0xD5 (DE), 0xE5 (HL), 0xF5 (AF)
/// POP opcodes:  0xC1 (BC), 0xD1 (DE), 0xE1 (HL), 0xF1 (AF)
///
/// The pattern is:
/// - PUSH: 0bxx00_0101 where xx encodes the register pair
/// - POP:  0bxx00_0001 where xx encodes the register pair
pub fn decode(byte: u8) -> Option<Instruction> {
    // Use the bit-extraction helper to map the opcode to a StackTarget for
    // PUSH and POP families. This centralises the decoding logic and avoids
    // duplicating the mapping in multiple places.
    // PUSH family: 0xC5, 0xD5, 0xE5, 0xF5
    if matches!(byte, 0xC5 | 0xD5 | 0xE5 | 0xF5) {
        if let Some(dst) = StackTarget::from_bits(byte) {
            return Some(Instruction::PUSH(dst));
        }
    }

    // POP family: 0xC1, 0xD1, 0xE1, 0xF1
    if matches!(byte, 0xC1 | 0xD1 | 0xE1 | 0xF1) {
        if let Some(dst) = StackTarget::from_bits(byte) {
            return Some(Instruction::POP(dst));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_instructions() {
        assert!(matches!(
            decode(0xC5),
            Some(Instruction::PUSH(StackTarget::BC))
        ));
        assert!(matches!(
            decode(0xD5),
            Some(Instruction::PUSH(StackTarget::DE))
        ));
        assert!(matches!(
            decode(0xE5),
            Some(Instruction::PUSH(StackTarget::HL))
        ));
        assert!(matches!(
            decode(0xF5),
            Some(Instruction::PUSH(StackTarget::AF))
        ));
    }

    #[test]
    fn test_pop_instructions() {
        assert!(matches!(
            decode(0xC1),
            Some(Instruction::POP(StackTarget::BC))
        ));
        assert!(matches!(
            decode(0xD1),
            Some(Instruction::POP(StackTarget::DE))
        ));
        assert!(matches!(
            decode(0xE1),
            Some(Instruction::POP(StackTarget::HL))
        ));
        assert!(matches!(
            decode(0xF1),
            Some(Instruction::POP(StackTarget::AF))
        ));
    }

    #[test]
    fn test_invalid_opcodes() {
        // Test some bytes that are not stack operations
        assert!(decode(0x00).is_none()); // NOP
        assert!(decode(0x80).is_none()); // ADD A, B
        assert!(decode(0xC0).is_none()); // RET NZ (conditional return, not POP)
    }
}
