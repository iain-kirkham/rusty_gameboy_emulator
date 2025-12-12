// Main instruction decoder module
//
// This module coordinates the decoding of Game Boy opcodes into Instruction enums.
// It delegates to specialized decoder modules for different instruction categories.

mod arithmetic;
mod control_flow;
mod incdec;
mod load;
mod prefix;

use super::Instruction;

/// Main entry point for instruction decoding
///
/// Decodes a single byte opcode into an Instruction enum.
///
/// # Arguments
/// * `byte` - The opcode byte to decode
/// * `prefixed` - Whether this is a CB-prefixed instruction
///
/// # Returns
/// * `Some(Instruction)` if the opcode is recognized
/// * `None` if the opcode is invalid or not yet implemented
///
/// # Examples
/// ```
/// use rusty_gameboy_emulator::instructions::decode::decode_instruction;
///
/// // Decode NOP instruction
/// let instr = decode_instruction(0x00, false);
/// assert!(instr.is_some());
///
/// // Decode CB-prefixed RLC B instruction
/// let instr = decode_instruction(0x00, true);
/// assert!(instr.is_some());
/// ```
pub fn decode_instruction(byte: u8, prefixed: bool) -> Option<Instruction> {
    if prefixed {
        decode_prefixed(byte)
    } else {
        decode_not_prefixed(byte)
    }
}

/// Decode CB-prefixed instructions
///
/// CB-prefixed instructions are extended opcodes that provide bit manipulation,
/// rotation, and shift operations.
pub fn decode_prefixed(byte: u8) -> Option<Instruction> {
    prefix::decode(byte)
}

/// Decode non-prefixed instructions
///
/// This tries each decoder in sequence until one succeeds.
/// The order is somewhat arbitrary but generally goes from simpler
/// to more complex instruction categories.
fn decode_not_prefixed(byte: u8) -> Option<Instruction> {
    None.or_else(|| control_flow::decode(byte))
        .or_else(|| incdec::decode(byte))
        .or_else(|| arithmetic::decode(byte))
        .or_else(|| load::decode(byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_not_prefixed() {
        // Test control flow
        assert!(decode_instruction(0x00, false).is_some()); // NOP

        // Test arithmetic
        assert!(decode_instruction(0x80, false).is_some()); // ADD A, B

        // Test load
        assert!(decode_instruction(0x3E, false).is_some()); // LD A, d8

        // Test inc/dec
        assert!(decode_instruction(0x04, false).is_some()); // INC B
    }

    #[test]
    fn test_decode_prefixed() {
        // Test CB-prefixed instruction
        assert!(decode_instruction(0x00, true).is_some()); // RLC B
    }

    #[test]
    fn test_decode_invalid() {
        // Test that invalid opcodes return None
        assert!(decode_instruction(0xD3, false).is_none());
        assert!(decode_instruction(0xDB, false).is_none());
        assert!(decode_instruction(0xDD, false).is_none());
    }

    #[test]
    fn test_halt_is_control_flow_not_load() {
        // 0x76 is HALT and should be decoded as control flow
        // It's in the middle of the load range but is a special case
        let instr = decode_instruction(0x76, false);
        assert!(instr.is_some());
        assert!(matches!(instr, Some(Instruction::HALT)));
    }
}
