use crate::instructions::{IncDecTarget, Instruction};
use crate::register::{Register16::*, Register8::*};

/// Decode increment and decrement instructions
/// These operate on both 8-bit and 16-bit registers
pub fn decode(byte: u8) -> Option<Instruction> {
    match byte {
        // ===== INC 8-bit =====
        0x04 => Some(Instruction::INC(IncDecTarget::Reg8(B))),
        0x0C => Some(Instruction::INC(IncDecTarget::Reg8(C))),
        0x14 => Some(Instruction::INC(IncDecTarget::Reg8(D))),
        0x1C => Some(Instruction::INC(IncDecTarget::Reg8(E))),
        0x24 => Some(Instruction::INC(IncDecTarget::Reg8(H))),
        0x2C => Some(Instruction::INC(IncDecTarget::Reg8(L))),
        0x3C => Some(Instruction::INC(IncDecTarget::Reg8(A))),

        // ===== INC 16-bit =====
        0x03 => Some(Instruction::INC(IncDecTarget::Reg16(BC))),
        0x13 => Some(Instruction::INC(IncDecTarget::Reg16(DE))),
        0x23 => Some(Instruction::INC(IncDecTarget::Reg16(HL))),
        0x33 => Some(Instruction::INC(IncDecTarget::Reg16(SP))),

        // ===== DEC 8-bit =====
        0x05 => Some(Instruction::DEC(IncDecTarget::Reg8(B))),
        0x0D => Some(Instruction::DEC(IncDecTarget::Reg8(C))),
        0x15 => Some(Instruction::DEC(IncDecTarget::Reg8(D))),
        0x1D => Some(Instruction::DEC(IncDecTarget::Reg8(E))),
        0x25 => Some(Instruction::DEC(IncDecTarget::Reg8(H))),
        0x2D => Some(Instruction::DEC(IncDecTarget::Reg8(L))),
        0x3D => Some(Instruction::DEC(IncDecTarget::Reg8(A))),

        // ===== DEC 16-bit =====
        0x0B => Some(Instruction::DEC(IncDecTarget::Reg16(BC))),
        0x1B => Some(Instruction::DEC(IncDecTarget::Reg16(DE))),
        0x2B => Some(Instruction::DEC(IncDecTarget::Reg16(HL))),
        0x3B => Some(Instruction::DEC(IncDecTarget::Reg16(SP))),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inc_8bit() {
        assert!(matches!(
            decode(0x04),
            Some(Instruction::INC(IncDecTarget::Reg8(B)))
        ));
        assert!(matches!(
            decode(0x3C),
            Some(Instruction::INC(IncDecTarget::Reg8(A)))
        ));
    }

    #[test]
    fn test_inc_16bit() {
        assert!(matches!(
            decode(0x03),
            Some(Instruction::INC(IncDecTarget::Reg16(BC)))
        ));
        assert!(matches!(
            decode(0x33),
            Some(Instruction::INC(IncDecTarget::Reg16(SP)))
        ));
    }

    #[test]
    fn test_dec_8bit() {
        assert!(matches!(
            decode(0x05),
            Some(Instruction::DEC(IncDecTarget::Reg8(B)))
        ));
        assert!(matches!(
            decode(0x3D),
            Some(Instruction::DEC(IncDecTarget::Reg8(A)))
        ));
    }

    #[test]
    fn test_dec_16bit() {
        assert!(matches!(
            decode(0x0B),
            Some(Instruction::DEC(IncDecTarget::Reg16(BC)))
        ));
        assert!(matches!(
            decode(0x3B),
            Some(Instruction::DEC(IncDecTarget::Reg16(SP)))
        ));
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(decode(0x00).is_none());
        assert!(decode(0xFF).is_none());
    }
}
