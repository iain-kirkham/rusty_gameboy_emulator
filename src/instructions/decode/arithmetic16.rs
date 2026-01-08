use crate::instructions::Instruction;
use crate::register::Register16;

/// Decode 16-bit arithmetic instructions
///
/// This includes:
/// - ADD HL, rr (0x09, 0x19, 0x29, 0x39)
/// - ADD SP, r8 (0xE8)
/// - LD HL, SP+r8 (0xF8)
/// - LD SP, HL (0xF9)
pub fn decode(byte: u8) -> Option<Instruction> {
    match byte {
        // ===== ADD HL, rr =====
        // Add 16-bit register to HL
        0x09 => Some(Instruction::ADDHL(Register16::BC)), // ADD HL, BC
        0x19 => Some(Instruction::ADDHL(Register16::DE)), // ADD HL, DE
        0x29 => Some(Instruction::ADDHL(Register16::HL)), // ADD HL, HL
        0x39 => Some(Instruction::ADDHL(Register16::SP)), // ADD HL, SP

        // ===== ADD SP, r8 =====
        // Add signed 8-bit immediate to SP
        0xE8 => Some(Instruction::ADDSP), // ADD SP, r8

        // ===== LD HL, SP+r8 =====
        // Load HL with SP + signed 8-bit immediate
        0xF8 => Some(Instruction::LDHLSP), // LD HL, SP+r8

        // ===== LD SP, HL =====
        // Load SP with HL
        0xF9 => Some(Instruction::LD(crate::instructions::LoadType::Word(
            crate::instructions::LoadWordTarget::SP,
            crate::instructions::LoadWordSource::HL,
        ))),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_hl_rr() {
        assert!(matches!(
            decode(0x09),
            Some(Instruction::ADDHL(Register16::BC))
        ));
        assert!(matches!(
            decode(0x19),
            Some(Instruction::ADDHL(Register16::DE))
        ));
        assert!(matches!(
            decode(0x29),
            Some(Instruction::ADDHL(Register16::HL))
        ));
        assert!(matches!(
            decode(0x39),
            Some(Instruction::ADDHL(Register16::SP))
        ));
    }

    #[test]
    fn test_add_sp_r8() {
        assert!(matches!(decode(0xE8), Some(Instruction::ADDSP)));
    }

    #[test]
    fn test_ld_hl_sp_plus_r8() {
        assert!(matches!(decode(0xF8), Some(Instruction::LDHLSP)));
    }

    #[test]
    fn test_ld_sp_hl() {
        assert!(matches!(
            decode(0xF9),
            Some(Instruction::LD(crate::instructions::LoadType::Word(_, _)))
        ));
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(decode(0x00).is_none());
        assert!(decode(0xFF).is_none());
    }
}
