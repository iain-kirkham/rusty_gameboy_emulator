// Shared target/source enums used across instruction types
use crate::register::{Register16, Register8};

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum IncDecTarget {
    Reg8(Register8),
    Reg16(Register16),
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum PrefixTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI, // Memory address pointed to by HL
}

impl PrefixTarget {
    /// Convert to Register8 if this is a register target
    pub fn to_register8(self) -> Option<Register8> {
        match self {
            PrefixTarget::A => Some(Register8::A),
            PrefixTarget::B => Some(Register8::B),
            PrefixTarget::C => Some(Register8::C),
            PrefixTarget::D => Some(Register8::D),
            PrefixTarget::E => Some(Register8::E),
            PrefixTarget::H => Some(Register8::H),
            PrefixTarget::L => Some(Register8::L),
            PrefixTarget::HLI => None,
        }
    }
}
