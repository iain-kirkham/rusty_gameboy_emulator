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
        0x76 => Some(Instruction::HALT),
        0xF3 => Some(Instruction::DI), // Disable interrupts
        0xFB => Some(Instruction::EI), // Enable interrupts

        // ===== ROTATE ACCUMULATOR =====
        0x07 => Some(Instruction::RLCA), // Rotate A left circular
        0x0F => Some(Instruction::RRCA), // Rotate A right circular
        0x17 => Some(Instruction::RLA),  // Rotate A left through carry
        0x1F => Some(Instruction::RRA),  // Rotate A right through carry

        // ===== MISCELLANEOUS ARITHMETIC =====
        0x27 => Some(Instruction::DAA), // Decimal Adjust Accumulator
        0x2F => Some(Instruction::CPL), // Complement A
        0x37 => Some(Instruction::SCF), // Set Carry Flag
        0x3F => Some(Instruction::CCF), // Complement Carry Flag

        // ===== JUMPS (JP) =====
        0xC3 => Some(Instruction::JP(JumpTest::Always)), // JP nn
        0xC2 | 0xCA | 0xD2 | 0xDA => JumpTest::from_bits(byte).map(Instruction::JP), // JP cc, nn
        0xE9 => Some(Instruction::JP_HL),                // JP (HL) - jump to address in HL

        // ===== RELATIVE JUMPS (JR) =====
        0x18 => Some(Instruction::JR(JumpTest::Always)), // JR r8 (unconditional)
        0x20 | 0x28 | 0x30 | 0x38 => JumpTest::from_bits(byte).map(Instruction::JR), // JR cc, r8

        // ===== CALLS =====
        0xCD => Some(Instruction::CALL(JumpTest::Always)), // CALL nn
        0xC4 | 0xCC | 0xD4 | 0xDC => JumpTest::from_bits(byte).map(Instruction::CALL), // CALL cc, nn

        // ===== RETURNS =====
        0xC9 => Some(Instruction::RET(JumpTest::Always)), // RET
        0xC0 | 0xC8 | 0xD0 | 0xD8 => JumpTest::from_bits(byte).map(Instruction::RET), // RET cc
        0xD9 => Some(Instruction::RETI),                  // RETI - Return and enable interrupts

        // ===== RESTARTS (RST) =====
        // These are single-byte CALLs to fixed addresses
        0xC7 => Some(Instruction::RST(0x00)), // RST 00H
        0xCF => Some(Instruction::RST(0x08)), // RST 08H
        0xD7 => Some(Instruction::RST(0x10)), // RST 10H
        0xDF => Some(Instruction::RST(0x18)), // RST 18H
        0xE7 => Some(Instruction::RST(0x20)), // RST 20H
        0xEF => Some(Instruction::RST(0x28)), // RST 28H
        0xF7 => Some(Instruction::RST(0x30)), // RST 30H
        0xFF => Some(Instruction::RST(0x38)), // RST 38H
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
        assert!(decode(0x80).is_none()); // ADD A, B
    }
}
