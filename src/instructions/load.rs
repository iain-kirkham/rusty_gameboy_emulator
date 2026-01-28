//! Load and store instructions module.
//!
//! This module defines the target and source types for load/store instructions,
//! and provides decoding utilities for register and memory operands.

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,  // Memory address pointed to by HL register pair
    DEI,  // Memory address pointed to by DE register pair
    BCI,  // Memory address pointed to by BC register pair
    A16I, // Memory address given by immediate 16-bit value (next two bytes)
    A8I,  // High RAM: Memory address 0xFF00 + immediate 8-bit offset
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,      // Immediate 8-bit value (next byte in instruction stream)
    HLI,     // Memory address pointed to by HL register pair
    HLI_INC, // Memory address at HL, then increment HL (HL+)
    HLI_DEC, // Memory address at HL, then decrement HL (HL-)
    BCI,     // Memory address pointed to by BC register pair
    A16I,    // Memory address given by immediate 16-bit value (next two bytes)
    A8I,     // High RAM: Memory address 0xFF00 + immediate 8-bit offset
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum LoadWordTarget {
    HL,
    BC,
    DE,
    SP,
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum LoadWordSource {
    D16, // Immediate 16-bit value (next two bytes in instruction stream)
    SP,
    HL, // HL register pair
}

#[derive(Debug, Copy, Clone)]
pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
}

impl LoadByteTarget {
    /// Decode a LoadByteTarget from the upper 3 bits (bits 5-3) of an opcode
    /// Used for LD dst, src instructions in the 0x40-0x7F range
    pub fn from_upper_bits(bits: u8) -> Option<Self> {
        match (bits >> 3) & 0x07 {
            0x00 => Some(LoadByteTarget::B),
            0x01 => Some(LoadByteTarget::C),
            0x02 => Some(LoadByteTarget::D),
            0x03 => Some(LoadByteTarget::E),
            0x04 => Some(LoadByteTarget::H),
            0x05 => Some(LoadByteTarget::L),
            0x06 => Some(LoadByteTarget::HLI),
            0x07 => Some(LoadByteTarget::A),
            _ => None,
        }
    }
}

impl LoadByteSource {
    /// Decode a LoadByteSource from the lower 3 bits (bits 2-0) of an opcode
    /// Used for LD dst, src instructions in the 0x40-0x7F range
    pub fn from_lower_bits(bits: u8) -> Option<Self> {
        match bits & 0x07 {
            0x00 => Some(LoadByteSource::B),
            0x01 => Some(LoadByteSource::C),
            0x02 => Some(LoadByteSource::D),
            0x03 => Some(LoadByteSource::E),
            0x04 => Some(LoadByteSource::H),
            0x05 => Some(LoadByteSource::L),
            0x06 => Some(LoadByteSource::HLI),
            0x07 => Some(LoadByteSource::A),
            _ => None,
        }
    }
}

impl LoadWordTarget {
    /// Decode a LoadWordTarget from bits 5-4 of an opcode
    /// Used for 16-bit LD instructions
    pub fn from_bits(bits: u8) -> Option<Self> {
        match (bits >> 4) & 0x03 {
            0x00 => Some(LoadWordTarget::BC),
            0x01 => Some(LoadWordTarget::DE),
            0x02 => Some(LoadWordTarget::HL),
            0x03 => Some(LoadWordTarget::SP),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_byte_target_from_upper_bits() {
        // Test decoding destination register from opcode
        assert!(matches!(
            LoadByteTarget::from_upper_bits(0x40),
            Some(LoadByteTarget::B)
        )); // 0x40 = 0b01000000, bits 5-3 = 000
        assert!(matches!(
            LoadByteTarget::from_upper_bits(0x78),
            Some(LoadByteTarget::A)
        )); // 0x78 = 0b01111000, bits 5-3 = 111
    }

    #[test]
    fn test_load_byte_source_from_lower_bits() {
        // Test decoding source register from opcode
        assert!(matches!(
            LoadByteSource::from_lower_bits(0x41),
            Some(LoadByteSource::C)
        )); // 0x41 = 0b01000001, bits 2-0 = 001
        assert!(matches!(
            LoadByteSource::from_lower_bits(0x47),
            Some(LoadByteSource::A)
        )); // 0x47 = 0b01000111, bits 2-0 = 111
    }

    #[test]
    fn test_load_word_target_from_bits() {
        assert!(matches!(
            LoadWordTarget::from_bits(0x01),
            Some(LoadWordTarget::BC)
        ));
        assert!(matches!(
            LoadWordTarget::from_bits(0x31),
            Some(LoadWordTarget::SP)
        ));
    }
}
