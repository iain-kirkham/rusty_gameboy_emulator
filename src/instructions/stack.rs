// Stack operations: PUSH and POP instructions

#[derive(Debug, Copy, Clone)]
pub enum StackTarget {
    BC,
    DE,
    HL,
    AF,
}

impl StackTarget {
    /// Decode a StackTarget from bits 5-4 of an opcode
    /// Used for PUSH/POP instructions
    pub fn from_bits(bits: u8) -> Option<Self> {
        match (bits >> 4) & 0x03 {
            0x00 => Some(StackTarget::BC),
            0x01 => Some(StackTarget::DE),
            0x02 => Some(StackTarget::HL),
            0x03 => Some(StackTarget::AF),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bits() {
        assert!(matches!(
            StackTarget::from_bits(0x00),
            Some(StackTarget::BC)
        ));
        assert!(matches!(
            StackTarget::from_bits(0x10),
            Some(StackTarget::DE)
        ));
        assert!(matches!(
            StackTarget::from_bits(0x20),
            Some(StackTarget::HL)
        ));
        assert!(matches!(
            StackTarget::from_bits(0x30),
            Some(StackTarget::AF)
        ));
    }
}
