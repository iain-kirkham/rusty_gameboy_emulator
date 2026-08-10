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

    fn trace_instruction(&self, prefixed: bool, opcode_byte: u8, _instruction: &Instruction) {
        // Build a readable opcode string (e.g. "0x3E" or "0xCB37")
        let _opcode_str = if prefixed {
            format!("0xCB{:02X}", opcode_byte)
        } else {
            format!("0x{:02X}", opcode_byte)
        };

        // Print a compact CPU state for debugging: PC, opcode, decoded instruction,
        // registers A,B,C,D,E,H,L, SP, HL and flags (raw F and booleans).
//         if self.registers.pc < 0x0206 || self.registers.pc > 0x020D {
//             println!(
//                 "PC={:#06X} OPCODE={} INST={:?} \
// A={:#04X} F={:02X} Z={} N={} H={} C={} \
// B={:#04X} C={:#04X} D={:#04X} E={:#04X} H={:#04X} L={:#04X} \
// SP={:#06X} HL={:#06X}",
//                 self.registers.pc,
//                 opcode_str,
//                 instruction,
//                 self.registers.a,
//                 self.registers.f.to_byte(),
//                 self.registers.f.zero,
//                 self.registers.f.subtract,
//                 self.registers.f.half_carry,
//                 self.registers.f.carry,
//                 self.registers.b,
//                 self.registers.c,
//                 self.registers.d,
//                 self.registers.e,
//                 self.registers.h,
//                 self.registers.l,
//                 self.registers.sp,
//                 self.registers.get_hl()
//             );
//         }
    }

    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.registers.pc.wrapping_add(1))
    }

    fn read_next_word(&self) -> u16 {
        let lo = self.bus.read_byte(self.registers.pc.wrapping_add(1)) as u16;
        let hi = self.bus.read_byte(self.registers.pc.wrapping_add(2)) as u16;
        (hi << 8) | lo
    }
}
