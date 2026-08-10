use super::CPU;
use crate::instructions::Instruction;

pub(crate) trait FetchTraceOps {
    fn fetch_and_decode_instruction(&self) -> (bool, u8, Instruction);
    fn trace_instruction(&self, prefixed: bool, opcode_byte: u8, instruction: &Instruction);
    fn read_next_byte(&self) -> u8;
    fn read_next_word(&self) -> u16;
}

impl FetchTraceOps for CPU {
    fn fetch_and_decode_instruction(&self) -> (bool, u8, Instruction) {
        // Read first opcode byte and determine if it's a CB-prefix
        let first_byte = self.bus.read_byte(self.registers.pc);
        let prefixed = first_byte == 0xCB;

        // For prefixed instructions, opcode byte is the second byte; otherwise use first.
        let opcode_byte = if prefixed {
            self.bus.read_byte(self.registers.pc + 1)
        } else {
            first_byte
        };

        // Decode instruction
        if let Some(instruction) = Instruction::from_byte(opcode_byte, prefixed) {
            (prefixed, opcode_byte, instruction)
        } else {
            let instruction_str = if prefixed {
                format!("0xCB{:02X}", opcode_byte)
            } else {
                format!("0x{:02X}", opcode_byte)
            };
            panic!(
                "Unknown instruction {} at PC=0x{:04X}",
                instruction_str, self.registers.pc
            );
        }
    }

    fn trace_instruction(&self, _prefixed: bool, _opcode_byte: u8, _instruction: &Instruction) {}

    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.registers.pc.wrapping_add(1))
    }

    fn read_next_word(&self) -> u16 {
        let lo = self.bus.read_byte(self.registers.pc.wrapping_add(1)) as u16;
        let hi = self.bus.read_byte(self.registers.pc.wrapping_add(2)) as u16;
        (hi << 8) | lo
    }
}
