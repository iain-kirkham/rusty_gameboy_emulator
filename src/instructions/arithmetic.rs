// Arithmetic and logical operations
#[derive(Debug, Copy, Clone)]
pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ArithmeticTarget {
    /// Decode an ArithmeticTarget from the lower 3 bits of an opcode
    /// Game Boy opcodes often encode register operands in bits 0-2
    pub fn from_lower_bits(bits: u8) -> Option<Self> {
        match bits & 0x07 {
            0x00 => Some(ArithmeticTarget::B),
            0x01 => Some(ArithmeticTarget::C),
            0x02 => Some(ArithmeticTarget::D),
            0x03 => Some(ArithmeticTarget::E),
            0x04 => Some(ArithmeticTarget::H),
            0x05 => Some(ArithmeticTarget::L),
            0x06 => None, // (HL) - memory indirect addressing
            0x07 => Some(ArithmeticTarget::A),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_lower_bits() {
        assert!(matches!(
            ArithmeticTarget::from_lower_bits(0x00),
            Some(ArithmeticTarget::B)
        ));
        assert!(matches!(
            ArithmeticTarget::from_lower_bits(0x07),
            Some(ArithmeticTarget::A)
        ));
        assert!(matches!(
            ArithmeticTarget::from_lower_bits(0x85), // bits = 0b10000101, lower 3 = 0b101 = 5
            Some(ArithmeticTarget::L)
        ));
        assert!(ArithmeticTarget::from_lower_bits(0x06).is_none());
    }
}
