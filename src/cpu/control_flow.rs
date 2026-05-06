use super::CPU;
use super::FetchTraceOps;
use crate::instructions::JumpTest;

pub(crate) trait ControlFlowOps {
    fn should_jump(&self, test: &JumpTest) -> bool;
    fn jump(&mut self, should_jump: bool) -> u16;
    fn jump_relative(&mut self, should_jump: bool) -> u16;
    fn call(&mut self, should_jump: bool) -> u16;
    fn return_(&mut self, should_jump: bool) -> u16;
}

impl ControlFlowOps for CPU {
    /// Evaluate a jump condition based on CPU flags.
    fn should_jump(&self, test: &JumpTest) -> bool {
        match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        }
    }

    /// Execute an absolute jump to a 16-bit address (JP instruction).
    /// Returns the target address if should_jump is true, otherwise PC+3 (skip instruction).
    fn jump(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.read_next_word()
        } else {
            self.registers.pc.wrapping_add(3)
        }
    }

    fn jump_relative(&mut self, should_jump: bool) -> u16 {
        // Fetch the signed 8-bit offset using read_next_byte().
        // This centralises operand reads and makes the intent explicit:
        // the offset byte is the operand at PC+1 and the relative jump is
        // calculated from the address after the instruction (PC + 2).
        let offset_byte = self.read_next_byte() as i8;
        if should_jump {
            (self.registers.pc.wrapping_add(2) as i16).wrapping_add(offset_byte as i16) as u16
        } else {
            self.registers.pc.wrapping_add(2)
        }
    }

    /// Execute a CALL instruction: conditionally push return address and jump.
    /// Returns target address if should_jump is true, otherwise PC+3 (skip instruction).
    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.registers.pc.wrapping_add(3);
        if should_jump {
            // Push return address in the same order as stack helper: high byte, then low byte.
            self.registers.sp = self.registers.sp.wrapping_sub(1);
            self.bus
                .write_byte(self.registers.sp, ((next_pc & 0xFF00) >> 8) as u8);
            self.registers.sp = self.registers.sp.wrapping_sub(1);
            self.bus.write_byte(self.registers.sp, (next_pc & 0xFF) as u8);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    /// Execute a RET instruction: conditionally pop return address from stack.
    /// Returns the popped address if should_jump is true, otherwise PC+1 (skip instruction).
    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            let lsb = self.bus.read_byte(self.registers.sp) as u16;
            self.registers.sp = self.registers.sp.wrapping_add(1);
            let msb = self.bus.read_byte(self.registers.sp) as u16;
            self.registers.sp = self.registers.sp.wrapping_add(1);
            (msb << 8) | lsb
        } else {
            self.registers.pc.wrapping_add(1)
        }
    }
}