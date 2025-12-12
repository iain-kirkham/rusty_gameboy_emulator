use crate::instructions::{ArithmeticTarget, Instruction};

/// Decode arithmetic and logical instructions
/// These instructions operate on the A register and various source operands
pub fn decode(byte: u8) -> Option<Instruction> {
    // ADD A, r instructions (0x80-0x87, excluding 0x86 which uses (HL))
    if byte >= 0x80 && byte <= 0x87 {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::ADD);
    }

    // ADC A, r instructions (Add with Carry) (0x88-0x8F)
    if byte >= 0x88 && byte <= 0x8F {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::ADC);
    }

    // SUB A, r instructions (0x90-0x97)
    if byte >= 0x90 && byte <= 0x97 {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::SUB);
    }

    // SBC A, r instructions (Subtract with Carry) (0x98-0x9F)
    if byte >= 0x98 && byte <= 0x9F {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::SBC);
    }

    // AND A, r instructions (0xA0-0xA7)
    if byte >= 0xA0 && byte <= 0xA7 {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::AND);
    }

    // XOR A, r instructions (0xA8-0xAF)
    if byte >= 0xA8 && byte <= 0xAF {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::XOR);
    }

    // OR A, r instructions (0xB0-0xB7)
    if byte >= 0xB0 && byte <= 0xB7 {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::OR);
    }

    // CP A, r instructions (Compare) (0xB8-0xBF)
    if byte >= 0xB8 && byte <= 0xBF {
        return ArithmeticTarget::from_lower_bits(byte).map(Instruction::CP);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_instructions() {
        assert!(matches!(
            decode(0x80),
            Some(Instruction::ADD(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0x87),
            Some(Instruction::ADD(ArithmeticTarget::A))
        ));
        // 0x86 would be ADD A, (HL) which returns None from from_lower_bits
        assert!(decode(0x86).is_none());
    }

    #[test]
    fn test_adc_instructions() {
        assert!(matches!(
            decode(0x88),
            Some(Instruction::ADC(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0x8F),
            Some(Instruction::ADC(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_sub_instructions() {
        assert!(matches!(
            decode(0x90),
            Some(Instruction::SUB(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0x97),
            Some(Instruction::SUB(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_sbc_instructions() {
        assert!(matches!(
            decode(0x98),
            Some(Instruction::SBC(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0x9F),
            Some(Instruction::SBC(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_and_instructions() {
        assert!(matches!(
            decode(0xA0),
            Some(Instruction::AND(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0xA7),
            Some(Instruction::AND(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_xor_instructions() {
        assert!(matches!(
            decode(0xA8),
            Some(Instruction::XOR(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0xAF),
            Some(Instruction::XOR(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_or_instructions() {
        assert!(matches!(
            decode(0xB0),
            Some(Instruction::OR(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0xB7),
            Some(Instruction::OR(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_cp_instructions() {
        assert!(matches!(
            decode(0xB8),
            Some(Instruction::CP(ArithmeticTarget::B))
        ));
        assert!(matches!(
            decode(0xBF),
            Some(Instruction::CP(ArithmeticTarget::A))
        ));
    }

    #[test]
    fn test_invalid_opcodes() {
        // Test opcodes outside arithmetic ranges
        assert!(decode(0x00).is_none());
        assert!(decode(0x7F).is_none());
        assert!(decode(0xC0).is_none());
        assert!(decode(0xFF).is_none());
    }
}
