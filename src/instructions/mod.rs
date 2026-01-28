//! Game Boy Instruction Set module.
//!
//! This module defines all instruction types and provides decoding functionality
//! for the Game Boy CPU instruction set.
//!
//! The instruction set is organized into logical categories:
//! - Arithmetic/Logic operations (ADD, SUB, AND, XOR, etc.)
//! - Load/Store operations (LD)
//! - Control flow (JP, JR, CALL, RET)
//! - Stack operations (PUSH, POP)
//! - Increment/Decrement (INC, DEC)
//! - Prefix/Extended instructions (CB-prefixed opcodes)

pub mod arithmetic;
pub mod control_flow;
pub mod decode;
#[allow(non_camel_case_types)]
pub mod load;
pub mod stack;
pub mod targets;

// Re-export commonly used types for convenience
pub use arithmetic::ArithmeticTarget;
pub use control_flow::JumpTest;
pub use decode::decode_instruction;
pub use load::{LoadByteSource, LoadByteTarget, LoadType, LoadWordSource, LoadWordTarget};
pub use stack::StackTarget;
pub use targets::{IncDecTarget, PrefixTarget};

use crate::register::Register16;

/// Main instruction enum representing all Game Boy CPU instructions
///
/// Each variant corresponds to a specific instruction or instruction family.
/// Many variants contain additional data specifying operands, addressing modes,
/// or conditions.
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Instruction {
    // ===== ARITHMETIC & LOGIC =====
    /// Add to accumulator (ADD A, r)
    ADD(ArithmeticTarget),
    /// Add with carry to accumulator (ADC A, r)
    ADC(ArithmeticTarget),
    /// Subtract from accumulator (SUB A, r)
    SUB(ArithmeticTarget),
    /// Subtract with carry from accumulator (SBC A, r)
    SBC(ArithmeticTarget),
    /// Logical AND with accumulator (AND A, r)
    AND(ArithmeticTarget),
    /// Logical OR with accumulator (OR A, r)
    OR(ArithmeticTarget),
    /// Logical XOR with accumulator (XOR A, r)
    XOR(ArithmeticTarget),
    /// Compare with accumulator (CP A, r)
    CP(ArithmeticTarget),

    // ===== CONTROL FLOW =====
    /// Jump to address (JP nn or JP cc, nn)
    JP(JumpTest),
    /// Relative jump (JR r8 or JR cc, r8)
    JR(JumpTest),
    /// Call subroutine (CALL nn or CALL cc, nn)
    CALL(JumpTest),
    /// Return from subroutine (RET or RET cc)
    RET(JumpTest),
    /// Return from interrupt handler and enable interrupts (RETI)
    RETI,
    /// Restart - call to fixed address (RST vec)
    /// The target is the reset vector address (0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38)
    RST(u8),

    // ===== LOAD/STORE =====
    /// Load/store data (LD dst, src)
    LD(LoadType),

    // ===== STACK OPERATIONS =====
    /// Pop from stack into register pair (POP rr)
    POP(StackTarget),
    /// Push register pair onto stack (PUSH rr)
    PUSH(StackTarget),

    // ===== INCREMENT/DECREMENT =====
    /// Increment register or memory (INC r or INC rr)
    INC(IncDecTarget),
    /// Decrement register or memory (DEC r or DEC rr)
    DEC(IncDecTarget),

    // ===== PREFIX (CB) INSTRUCTIONS =====
    /// Rotate left circular (RLC r)
    RLC(PrefixTarget),
    /// Rotate right circular (RRC r)
    RRC(PrefixTarget),
    /// Rotate left through carry (RL r)
    RL(PrefixTarget),
    /// Rotate right through carry (RR r)
    RR(PrefixTarget),
    /// Shift left arithmetic (SLA r)
    SLA(PrefixTarget),
    /// Shift right arithmetic (SRA r)
    SRA(PrefixTarget),
    /// Swap nibbles (SWAP r)
    SWAP(PrefixTarget),
    /// Shift right logical (SRL r)
    SRL(PrefixTarget),
    /// Test bit (BIT b, r)
    BIT(u8, PrefixTarget),
    /// Reset bit (RES b, r)
    RES(u8, PrefixTarget),
    /// Set bit (SET b, r)
    SET(u8, PrefixTarget),

    // ===== CONTROL =====
    /// No operation
    NOP,
    /// Halt CPU until interrupt
    HALT,
    /// Disable interrupts (DI)
    DI,
    /// Enable interrupts (EI)
    EI,

    // ===== ROTATE ACCUMULATOR =====
    /// Rotate A left circular (RLCA)
    RLCA,
    /// Rotate A right circular (RRCA)
    RRCA,
    /// Rotate A left through carry (RLA)
    RLA,
    /// Rotate A right through carry (RRA)
    RRA,

    // ===== MISCELLANEOUS ARITHMETIC =====
    /// Decimal Adjust Accumulator (DAA)
    DAA,
    /// Complement A (CPL)
    CPL,
    /// Set Carry Flag (SCF)
    SCF,
    /// Complement Carry Flag (CCF)
    CCF,

    // ===== 16-BIT ARITHMETIC =====
    /// Add 16-bit register to HL (ADD HL, rr)
    ADDHL(Register16),
    /// Add signed 8-bit immediate to SP (ADD SP, r8)
    ADDSP,
    /// Load HL with SP + signed 8-bit immediate (LD HL, SP+r8)
    LDHLSP,

    // ===== SPECIAL JUMPS =====
    /// Jump to address in HL (JP (HL))
    JP_HL,
}

impl Instruction {
    /// Decode a byte into an Instruction
    ///
    /// This is the primary interface for instruction decoding.
    ///
    /// # Arguments
    /// * `byte` - The opcode byte to decode
    /// * `prefixed` - True if this is a CB-prefixed instruction
    ///
    /// # Returns
    /// * `Some(Instruction)` if the opcode is valid
    /// * `None` if the opcode is invalid or not yet implemented
    ///
    /// # Examples
    /// ```
    /// use rusty_gameboy_emulator::instructions::Instruction;
    ///
    /// // Decode NOP (0x00)
    /// let instr = Instruction::from_byte(0x00, false);
    /// assert!(instr.is_some());
    ///
    /// // Decode invalid opcode
    /// let instr = Instruction::from_byte(0xD3, false);
    /// assert!(instr.is_none());
    /// ```
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        decode_instruction(byte, prefixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_byte_basic() {
        // Test that from_byte delegates to decode_instruction correctly
        assert!(Instruction::from_byte(0x00, false).is_some()); // NOP
        assert!(Instruction::from_byte(0x3E, false).is_some()); // LD A, d8
        assert!(Instruction::from_byte(0x80, false).is_some()); // ADD A, B
    }

    #[test]
    fn test_from_byte_prefixed() {
        assert!(Instruction::from_byte(0x00, true).is_some()); // RLC B
    }

    #[test]
    fn test_from_byte_invalid() {
        assert!(Instruction::from_byte(0xD3, false).is_none());
    }
}
