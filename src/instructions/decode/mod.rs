//! Main instruction decoder module.
//!
//! This module coordinates the decoding of Game Boy opcodes into Instruction enums.
//! It delegates to specialized decoder modules for different instruction categories.

mod arithmetic;
mod arithmetic16;
mod control_flow;
mod incdec;
mod load;
mod prefix;
mod stack;

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
        .or_else(|| arithmetic16::decode(byte))
        .or_else(|| load::decode(byte))
        .or_else(|| stack::decode(byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::StackTarget;

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

        // Test stack
        assert!(decode_instruction(0xC5, false).is_some()); // PUSH BC
        assert!(decode_instruction(0xC1, false).is_some()); // POP BC
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

    #[test]
    fn test_stack_operations() {
        // Test PUSH instructions
        assert!(matches!(
            decode_instruction(0xC5, false),
            Some(Instruction::PUSH(StackTarget::BC))
        ));
        assert!(matches!(
            decode_instruction(0xF5, false),
            Some(Instruction::PUSH(StackTarget::AF))
        ));

        // Test POP instructions
        assert!(matches!(
            decode_instruction(0xC1, false),
            Some(Instruction::POP(StackTarget::BC))
        ));
        assert!(matches!(
            decode_instruction(0xF1, false),
            Some(Instruction::POP(StackTarget::AF))
        ));
    }

    #[test]
    fn test_call_ret_operations() {
        use crate::instructions::JumpTest;

        // Test CALL instructions
        assert!(matches!(
            decode_instruction(0xCD, false),
            Some(Instruction::CALL(JumpTest::Always))
        ));
        assert!(matches!(
            decode_instruction(0xC4, false),
            Some(Instruction::CALL(JumpTest::NotZero))
        ));

        // Test RET instructions
        assert!(matches!(
            decode_instruction(0xC9, false),
            Some(Instruction::RET(JumpTest::Always))
        ));
        assert!(matches!(
            decode_instruction(0xC0, false),
            Some(Instruction::RET(JumpTest::NotZero))
        ));
    }
}
