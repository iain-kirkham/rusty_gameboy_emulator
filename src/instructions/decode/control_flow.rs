use crate::instructions::{Instruction, JumpTest};

/// Decode control flow instructions (jumps, calls, returns, and basic control)
///
///   CALL and RET opcodes are not yet implemented here. When adding:
///   CALL nn should push return address (PC + 3) to the stack and set PC to nn.
///   RET should pop return address from stack and set PC to it.
///   Conditional CALL/RET must evaluate flags and only modify stack/PC on taken paths.

pub fn decode(byte: u8) -> Option<Instruction> {
    match byte {
        // ===== BASIC CONTROL =====
        0x00 => Some(Instruction::NOP),
        0x76 => Some(Instruction::HALT),
        0xF3 => Some(Instruction::DI), // Disable interrupts
        0xFB => Some(Instruction::EI), // Enable interrupts

        // ===== JUMPS (JP) =====
        0xC3 => Some(Instruction::JP(JumpTest::Always)), // JP nn

        // ===== RELATIVE JUMPS (JR) =====
        0x20 => Some(Instruction::JR(JumpTest::NotZero)), // JR NZ, r8
        0x28 => Some(Instruction::JR(JumpTest::Zero)),    // JR Z, r8
        0x30 => Some(Instruction::JR(JumpTest::NotCarry)), // JR NC, r8
        0x38 => Some(Instruction::JR(JumpTest::Carry)),   // JR C, r8
        0x18 => Some(Instruction::JR(JumpTest::Always)),  // JR r8 (unconditional)

        // ===== CALLS =====
        // TODO: Add CALL instructions when implemented

        // ===== RETURNS =====
        // TODO: Add RET instructions when implemented
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_control() {
        assert!(matches!(decode(0x00), Some(Instruction::NOP)));
        assert!(matches!(decode(0x76), Some(Instruction::HALT)));
        assert!(matches!(decode(0xF3), Some(Instruction::DI)));
        assert!(matches!(decode(0xFB), Some(Instruction::EI)));
    }

    #[test]
    fn test_jumps() {
        assert!(matches!(
            decode(0xC3),
            Some(Instruction::JP(JumpTest::Always))
        ));
    }

    #[test]
    fn test_relative_jumps() {
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
        assert!(matches!(
            decode(0x18),
            Some(Instruction::JR(JumpTest::Always))
        ));
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(decode(0xFF).is_none());
    }
}
