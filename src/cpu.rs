//! CPU module implementing the Game Boy processor.
//!
//! This module contains the CPU struct and instruction execution logic,
//! managing registers, memory access, and the fetch-decode-execute cycle.

use crate::flag_helpers as fh;
use crate::instructions::{
    ArithmeticTarget, IncDecTarget, Instruction, JumpTest, LoadByteSource, LoadByteTarget,
    LoadType, LoadWordSource, LoadWordTarget, StackTarget,
};
use crate::interrupts::INTERRUPT_CYCLES;
use crate::memory_bus::MemoryBus;
use crate::register::{self, Register16, Registers};

pub(crate) struct CPU {
    pub registers: register::Registers,
    pub bus: MemoryBus,
    is_halted: bool,
    pub interrupts_enabled: bool,
    ei_pending: bool,
    halt_bug: bool,
}

impl CPU {
    /// Create a new CPU with the initial register state and a ROM loaded into the bus.
    pub(crate) fn new(rom_data: Vec<u8>) -> CPU {
        let bus = MemoryBus::new(rom_data);
        CPU {
            registers: Registers::new(),
            bus,
            is_halted: false,
            interrupts_enabled: false,
            ei_pending: false,
            halt_bug: false,
        }
    }

    /// Execute a decoded instruction and return (next_pc, cycles_in_tstates).
    ///
    /// Cycle counts here are provided so the rest of the emulator can
    /// step timers/PPU/DMA appropriately. These values are the T-states for
    /// the instruction.
    fn execute(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::NOP => (self.registers.pc.wrapping_add(1), 4),
            // STOP: Stops CPU and LCD execution until a button press occurs.
            Instruction::STOP => {
                self.is_halted = true;
                (self.registers.pc.wrapping_add(2), 4)
            }
            // HALT: Stops CPU execution until an interrupt occurs.
            // HALT bug: When IME=0 and there's a pending interrupt, PC fails to increment
            Instruction::HALT => {
                if !self.interrupts_enabled && self.bus.any_interrupt_pending() {
                    // HALT bug condition: don't halt, but set flag to skip PC increment on next fetch
                    self.halt_bug = true;
                } else {
                    self.is_halted = true;
                }
                (self.registers.pc.wrapping_add(1), 4)
            }
            // DI/EI: Interrupt control instructions
            Instruction::DI => {
                self.interrupts_enabled = false;
                self.ei_pending = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::EI => {
                self.ei_pending = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // Arithmetic operations on A register
            Instruction::ADD(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.add(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::ADC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.adc(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::SUB(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sub(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::SBC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sbc(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            // Logical operations on A register
            Instruction::AND(target) => {
                let value = self.get_arithmetic_target(target);
                self.registers.a &= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::OR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.or(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::XOR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.xor(value);
                self.registers.a = new_value;
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            Instruction::CP(target) => {
                let value = self.get_arithmetic_target(target);
                self.cp(value);
                let (pc_inc, cycles) = match target {
                    ArithmeticTarget::D8 => (2, 8),
                    ArithmeticTarget::HLI => (1, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc_inc), cycles)
            }
            // Increment/Decrement instructions (8-bit and 16-bit)
            Instruction::INC(target) => match target {
                IncDecTarget::Reg8(reg) => {
                    let value = self.registers.read_8bit(reg);
                    let new_value = self.inc_8bit(value);
                    self.registers.write_8bit(reg, new_value);
                    (self.registers.pc.wrapping_add(1), 4)
                }
                IncDecTarget::Reg16(reg) => {
                    self.inc_16bit(reg);
                    (self.registers.pc.wrapping_add(1), 8)
                }
                IncDecTarget::HLI => {
                    let address = self.registers.get_hl();
                    let value = self.bus.read_byte(address);
                    let new_value = self.inc_8bit(value);
                    self.bus.write_byte(address, new_value);
                    (self.registers.pc.wrapping_add(1), 12)
                }
            },
            Instruction::DEC(target) => match target {
                IncDecTarget::Reg8(reg) => {
                    let value = self.registers.read_8bit(reg);
                    let new_value = self.dec_8bit(value);
                    self.registers.write_8bit(reg, new_value);
                    (self.registers.pc.wrapping_add(1), 4)
                }
                IncDecTarget::Reg16(reg) => {
                    self.dec_16bit(reg);
                    (self.registers.pc.wrapping_add(1), 8)
                }
                IncDecTarget::HLI => {
                    let address = self.registers.get_hl();
                    let value = self.bus.read_byte(address);
                    let new_value = self.dec_8bit(value);
                    self.bus.write_byte(address, new_value);
                    (self.registers.pc.wrapping_add(1), 12)
                }
            },
            // Control flow: Jumps and relative jumps
            Instruction::JP(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.jump(should);
                let cycles = if should { 16 } else { 12 };
                (next_pc, cycles)
            }
            Instruction::JR(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.jump_relative(should);
                let cycles = if should { 12 } else { 8 };
                (next_pc, cycles)
            }
            // Data transfers: Load byte/word operations
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = self.read_byte_source(source);
                    self.write_byte_target(target, source_value);
                    let cycles = self.get_load_byte_cycles(target, source);

                    (
                        self.registers
                            .pc
                            .wrapping_add(self.get_load_byte_pc_increment(target, source)),
                        cycles,
                    )
                }
                // 16-bit load
                LoadType::Word(target, source) => {
                    let source_value = match source {
                        LoadWordSource::D16 => self.read_next_word(),
                        LoadWordSource::SP => self.registers.sp,
                        LoadWordSource::HL => self.registers.get_hl(),
                    };

                    match target {
                        LoadWordTarget::HL => self.registers.set_hl(source_value),
                        LoadWordTarget::BC => self.registers.set_bc(source_value),
                        LoadWordTarget::DE => self.registers.set_de(source_value),
                        LoadWordTarget::SP => self.registers.sp = source_value,
                        LoadWordTarget::A16I => {
                            let address = self.read_next_word();
                            self.bus.write_byte(address, (source_value & 0xFF) as u8);
                            self.bus
                                .write_byte(address.wrapping_add(1), (source_value >> 8) as u8);
                        }
                    };
                    let (length, cycles) = match (target, source) {
                        (LoadWordTarget::A16I, _) => (3, 20), // Opcode 0x08
                        (_, LoadWordSource::D16) => (3, 12),  // LD rr, d16
                        _ => (1, 8),                          // LD SP, HL, etc.
                    };
                    (self.registers.pc.wrapping_add(length), cycles)
                }
            },
            // Stack operations: Push/Pop/Call/Return
            Instruction::PUSH(target) => {
                let value = self.read_stack_target(target);
                self.push(value);
                (self.registers.pc.wrapping_add(1), 16)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                self.write_stack_target(target, result);
                (self.registers.pc.wrapping_add(1), 12)
            }
            Instruction::CALL(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.call(should);
                let cycles = if should { 24 } else { 12 };
                (next_pc, cycles)
            }
            Instruction::RET(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.return_(should);
                let cycles = match test {
                    JumpTest::Always => 16,
                    _ => {
                        if should {
                            20
                        } else {
                            8
                        }
                    }
                };
                (next_pc, cycles)
            }
            // RETI: Return from interrupt handler (pops PC and enables interrupts)
            Instruction::RETI => {
                self.interrupts_enabled = true;
                (self.pop(), 16)
            }
            // RST: Restart (push next PC and jump to reset vector)
            Instruction::RST(vec) => {
                let next_pc = self.registers.pc.wrapping_add(1);
                self.push(next_pc);
                (vec as u16, 16)
            }
            // Rotate accumulator instructions (A register only)
            Instruction::RLCA => {
                // RLCA: Rotate A left (bit 7 wraps to bit 0)
                let carry = (self.registers.a & 0x80) >> 7;
                self.registers.a = (self.registers.a << 1) | carry;
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RRCA => {
                // RRCA: Rotate A right (bit 0 wraps to bit 7)
                let carry = self.registers.a & 0x01;
                self.registers.a = (self.registers.a >> 1) | (carry << 7);
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RLA => {
                // RLA: Rotate A left through carry flag
                let old_carry = if self.registers.f.carry { 1 } else { 0 };
                let new_carry = (self.registers.a & 0x80) >> 7;
                self.registers.a = (self.registers.a << 1) | old_carry;
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = new_carry == 1;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RRA => {
                // RRA: Rotate A right through carry flag
                let old_carry = if self.registers.f.carry { 1 } else { 0 };
                let new_carry = self.registers.a & 0x01;
                self.registers.a = (self.registers.a >> 1) | (old_carry << 7);
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = new_carry == 1;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // Miscellaneous special arithmetic operations
            Instruction::DAA => {
                // Decimal Adjust Accumulator (DAA)
                let mut a = self.registers.a;
                let mut adjust: u8 = 0;
                let mut carry = self.registers.f.carry;

                if !self.registers.f.subtract {
                    if self.registers.f.half_carry || (a & 0x0F) > 9 {
                        adjust |= 0x06;
                    }
                    if carry || a > 0x99 {
                        adjust |= 0x60;
                        carry = true;
                    }
                    a = a.wrapping_add(adjust);
                } else {
                    if self.registers.f.half_carry {
                        adjust |= 0x06;
                    }
                    if carry {
                        adjust |= 0x60;
                    }
                    a = a.wrapping_sub(adjust);
                }

                self.registers.a = a;
                self.registers.f.zero = a == 0;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CPL => {
                // Complement A (bitwise NOT)
                self.registers.a = !self.registers.a;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SCF => {
                // Set Carry Flag
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CCF => {
                // Complement Carry Flag
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = !self.registers.f.carry;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // 16-bit arithmetic: ADD HL and SP operations
            Instruction::ADDHL(reg) => {
                let hl = self.registers.get_hl();
                let value = match reg {
                    Register16::BC => self.registers.get_bc(),
                    Register16::DE => self.registers.get_de(),
                    Register16::HL => self.registers.get_hl(),
                    Register16::SP => self.registers.sp,
                };

                let (result, carry) = hl.overflowing_add(value);
                // Half carry for 16-bit: carry from bit 11 to bit 12
                let half_carry = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;

                self.registers.set_hl(result);
                self.registers.f.subtract = false;
                self.registers.f.half_carry = half_carry;
                self.registers.f.carry = carry;
                (self.registers.pc.wrapping_add(1), 8)
            }
            Instruction::ADDSP => {
                // ADDSP: Add signed 8-bit immediate to SP (flags set from lower 8 bits)
                let offset_signed = self.read_next_byte() as i8;
                let sp = self.registers.sp;
                let result = fh::add_sp_signed(sp, offset_signed);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = fh::half_carry_add_sp(sp, offset_signed);
                self.registers.f.carry = fh::carry_add_sp(sp, offset_signed);

                self.registers.sp = result;
                (self.registers.pc.wrapping_add(2), 16)
            }
            Instruction::LDHLSP => {
                // LDHLSP: Load HL with SP + signed 8-bit immediate (flags set from lower 8 bits)
                let offset_signed = self.read_next_byte() as i8;
                let sp = self.registers.sp;
                let result = fh::add_sp_signed(sp, offset_signed);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = fh::half_carry_add_sp(sp, offset_signed);
                self.registers.f.carry = fh::carry_add_sp(sp, offset_signed);

                self.registers.set_hl(result);
                (self.registers.pc.wrapping_add(2), 12)
            }
            // JP_HL: Jump to address stored in HL
            Instruction::JP_HL => (self.registers.get_hl(), 4),
            // CB prefix instructions: Rotations, Shifts, and Bit operations
            Instruction::RLC(target) => {
                let value = self.read_prefix_target(target);
                let carry = (value & 0x80) >> 7;
                let result = (value << 1) | carry;
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::RRC(target) => {
                let value = self.read_prefix_target(target);
                let carry = value & 0x01;
                let result = (value >> 1) | (carry << 7);
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::RL(target) => {
                let value = self.read_prefix_target(target);
                let old_carry = if self.registers.f.carry { 1 } else { 0 };
                let new_carry = (value & 0x80) >> 7;
                let result = (value << 1) | old_carry;
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = new_carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::RR(target) => {
                let value = self.read_prefix_target(target);
                let old_carry = if self.registers.f.carry { 1 } else { 0 };
                let new_carry = value & 0x01;
                let result = (value >> 1) | (old_carry << 7);
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = new_carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::SLA(target) => {
                let value = self.read_prefix_target(target);
                let carry = (value & 0x80) >> 7;
                let result = value << 1;
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::SRA(target) => {
                let value = self.read_prefix_target(target);
                let carry = value & 0x01;
                let msb = value & 0x80;
                let result = (value >> 1) | msb;
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::SWAP(target) => {
                let value = self.read_prefix_target(target);
                let result = ((value & 0x0F) << 4) | ((value & 0xF0) >> 4);
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            Instruction::SRL(target) => {
                let value = self.read_prefix_target(target);
                let carry = value & 0x01;
                let result = value >> 1;
                self.write_prefix_target(target, result);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry == 1;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            // BIT test: Check if a bit is set (zero flag = NOT bit)
            Instruction::BIT(bit, target) => {
                let value = self.read_prefix_target(target);
                let result = value & (1 << bit);
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            // RES: Clear (reset) a bit
            Instruction::RES(bit, target) => {
                let value = self.read_prefix_target(target);
                let result = value & !(1 << bit);
                self.write_prefix_target(target, result);
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
            // SET: Set a bit to 1
            Instruction::SET(bit, target) => {
                let value = self.read_prefix_target(target);
                let result = value | (1 << bit);
                self.write_prefix_target(target, result);
                let cycles = Self::get_prefix_cycles(&target);
                (self.registers.pc.wrapping_add(2), cycles)
            }
        }
    }

    /// Calculate T-state cycles for CB-prefixed instructions.
    /// Returns 8 cycles for register targets or 16 cycles for memory (HL) targets.
    fn get_prefix_cycles(target: &crate::instructions::targets::PrefixTarget) -> u16 {
        match target.to_register8() {
            Some(_) => 8,
            None => 16,
        }
    }

    /// Read a byte from the specified source (register, memory location, or immediate value).
    /// Handles all LoadByteSource variants including indirect addressing modes.
    fn read_byte_source(&mut self, source: LoadByteSource) -> u8 {
        match source {
            LoadByteSource::A => self.registers.a,
            LoadByteSource::B => self.registers.b,
            LoadByteSource::C => self.registers.c,
            LoadByteSource::D => self.registers.d,
            LoadByteSource::E => self.registers.e,
            LoadByteSource::H => self.registers.h,
            LoadByteSource::L => self.registers.l,
            LoadByteSource::D8 => self.read_next_byte(),
            LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
            LoadByteSource::BCI => self.bus.read_byte(self.registers.get_bc()),
            LoadByteSource::DEI => self.bus.read_byte(self.registers.get_de()),
            LoadByteSource::HLI_INC => {
                let value = self.bus.read_byte(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                value
            }
            LoadByteSource::HLI_DEC => {
                let value = self.bus.read_byte(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                value
            }
            LoadByteSource::A16I => {
                let address = self.read_next_word();
                self.bus.read_byte(address)
            }
            LoadByteSource::A8I => {
                let offset = self.read_next_byte();
                let address = 0xFF00 + offset as u16;
                self.bus.read_byte(address)
            }
            LoadByteSource::CI => {
                let address = 0xFF00 + self.registers.c as u16;
                self.bus.read_byte(address)
            }
        }
    }

    /// Write a byte to the specified target (register, memory location, or I/O address).
    /// Handles all LoadByteTarget variants including indirect addressing modes.
    fn write_byte_target(&mut self, target: LoadByteTarget, value: u8) {
        match target {
            LoadByteTarget::A => self.registers.a = value,
            LoadByteTarget::B => self.registers.b = value,
            LoadByteTarget::C => self.registers.c = value,
            LoadByteTarget::D => self.registers.d = value,
            LoadByteTarget::E => self.registers.e = value,
            LoadByteTarget::H => self.registers.h = value,
            LoadByteTarget::L => self.registers.l = value,
            LoadByteTarget::HLI => self.bus.write_byte(self.registers.get_hl(), value),
            LoadByteTarget::DEI => self.bus.write_byte(self.registers.get_de(), value),
            LoadByteTarget::BCI => self.bus.write_byte(self.registers.get_bc(), value),
            LoadByteTarget::A16I => {
                let address = self.read_next_word();
                self.bus.write_byte(address, value);
            }
            LoadByteTarget::A8I => {
                let offset = self.read_next_byte();
                let address = 0xFF00 + offset as u16;
                self.bus.write_byte(address, value);
            }
            LoadByteTarget::HLI_INC => {
                let address = self.registers.get_hl();
                self.bus.write_byte(address, value);
                self.registers.set_hl(address.wrapping_add(1));
            }
            LoadByteTarget::HLI_DEC => {
                let address = self.registers.get_hl();
                self.bus.write_byte(address, value);
                self.registers.set_hl(address.wrapping_sub(1));
            }
            LoadByteTarget::CI => {
                let address = 0xFF00 + self.registers.c as u16;
                self.bus.write_byte(address, value);
            }
        }
    }

    /// Calculate how much the PC should advance based on the load source.
    fn get_load_byte_pc_increment(&self, target: LoadByteTarget, source: LoadByteSource) -> u16 {
        match (target, source) {
            (LoadByteTarget::A16I, _) => 3,
            (LoadByteTarget::A8I, _) => 2,

            (_, LoadByteSource::A16I) => 3,
            (_, LoadByteSource::A8I) => 2,
            (_, LoadByteSource::D8) => 2,
            _ => 1,
        }
    }

    /// Compute T-cycle cost for LD byte operations based on both target and source.
    fn get_load_byte_cycles(&self, target: LoadByteTarget, source: LoadByteSource) -> u16 {
        let base = match source {
            LoadByteSource::D8 => 8,
            LoadByteSource::A16I => 16,
            LoadByteSource::A8I => 12,
            LoadByteSource::HLI => 8,
            LoadByteSource::HLI_INC => 8,
            LoadByteSource::HLI_DEC => 8,
            LoadByteSource::BCI => 8,
            LoadByteSource::DEI => 8,
            LoadByteSource::CI => 8,
            LoadByteSource::A => 4,
            LoadByteSource::B => 4,
            LoadByteSource::C => 4,
            LoadByteSource::D => 4,
            LoadByteSource::E => 4,
            LoadByteSource::H => 4,
            LoadByteSource::L => 4,
        };

        // Additional cost when the target is a memory location (different for
        // no-immediate, 8-bit immediate, and 16-bit immediate target addressing)
        let extra = match target {
            LoadByteTarget::HLI
            | LoadByteTarget::HLI_INC
            | LoadByteTarget::HLI_DEC
            | LoadByteTarget::BCI
            | LoadByteTarget::DEI
            | LoadByteTarget::CI => 4,
            LoadByteTarget::A8I => 8,
            LoadByteTarget::A16I => 12,
            _ => 0,
        };

        base + extra
    }

    /// Fetch the value from an 8-bit arithmetic target register.
    /// Used by arithmetic operations (ADD, SUB, AND, OR, XOR, CP) to get the operand.
    fn get_arithmetic_target(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.registers.a,
            ArithmeticTarget::B => self.registers.b,
            ArithmeticTarget::C => self.registers.c,
            ArithmeticTarget::D => self.registers.d,
            ArithmeticTarget::E => self.registers.e,
            ArithmeticTarget::H => self.registers.h,
            ArithmeticTarget::L => self.registers.l,
            ArithmeticTarget::HLI => self.bus.read_byte(self.registers.get_hl()),
            ArithmeticTarget::D8 => self.bus.read_byte(self.registers.pc.wrapping_add(1)),
        }
    }

    /// Perform 8-bit addition: A += value (sets all CPU flags).
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        let half_carry = fh::half_carry_add(self.registers.a, value);
        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit addition with carry: A += value + carry_flag (sets all CPU flags).
    fn adc(&mut self, value: u8) -> u8 {
        let carry_in = self.registers.f.carry;

        let (temp, overflow1) = self.registers.a.overflowing_add(value);
        let (new_value, overflow2) = temp.overflowing_add(if carry_in { 1 } else { 0 });

        // Check for half carry: carry from bit 3 to bit 4
        let half_carry = fh::half_carry_add_with_carry(self.registers.a, value, carry_in);
        let did_overflow = overflow1 || overflow2;

        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit subtraction: A -= value (sets all CPU flags).
    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        let half_carry = fh::half_borrow_sub(self.registers.a, value);
        self.set_arithmetic_flags(new_value, true, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit subtraction with carry: A -= value - carry_flag (sets all CPU flags).
    fn sbc(&mut self, value: u8) -> u8 {
        let carry_in = self.registers.f.carry;

        let (temp, overflow1) = self.registers.a.overflowing_sub(value);
        let (new_value, overflow2) = temp.overflowing_sub(if carry_in { 1 } else { 0 });

        // Check for half carry (borrow from bit 4 to bit 3) using helper to avoid wrapping issues.
        let half_carry = fh::half_borrow_sub_with_carry(self.registers.a, value, carry_in);
        let did_overflow = overflow1 || overflow2;

        self.set_arithmetic_flags(new_value, true, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit bitwise AND: A &= value (sets Z and H flags, clears N and C).
    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.set_logic_flags(new_value, true);
        new_value
    }

    /// Perform 8-bit bitwise OR: A |= value (sets Z flag, clears N, H, and C).
    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.set_logic_flags(new_value, false);
        new_value
    }

    /// Perform 8-bit bitwise XOR: A ^= value (sets Z flag, clears N, H, and C).
    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.set_logic_flags(new_value, false);
        new_value
    }

    /// Set CPU flags for logical operations (AND/OR/XOR).
    /// Clears N and C flags; sets Z if result is zero; sets H based on operation.
    fn set_logic_flags(&mut self, result: u8, half_carry: bool) {
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = half_carry;
        self.registers.f.carry = false;
    }

    /// Set CPU flags for arithmetic operations (ADD/SUB/INC/DEC).
    /// Updates Z, N, H, and C flags based on operation results.
    fn set_arithmetic_flags(&mut self, result: u8, subtract: bool, carry: bool, half_carry: bool) {
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = subtract;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = half_carry;
    }

    /// Compare operation: Perform A - value and set flags without modifying A.
    fn cp(&mut self, value: u8) {
        let (result, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = fh::half_borrow_sub(self.registers.a, value);
    }

    /// Increment an 8-bit value (sets Z, N=false, H flags; doesn't affect C).
    fn inc_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = fh::half_carry_inc(value);
        new_value
    }

    /// Decrement an 8-bit value (sets Z, N=true, H flags; doesn't affect C).
    fn dec_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = fh::half_borrow_dec(value);
        new_value
    }

    /// Increment a 16-bit register (doesn't affect any CPU flags).
    fn inc_16bit(&mut self, reg: Register16) {
        match reg {
            Register16::SP => self.registers.sp = self.registers.sp.wrapping_add(1),
            _ => {
                let value = self.registers.read_16bit(reg);
                self.registers.write_16bit(reg, value.wrapping_add(1));
            }
        }
    }

    /// Decrement a 16-bit register (doesn't affect any CPU flags).
    fn dec_16bit(&mut self, reg: Register16) {
        match reg {
            Register16::SP => self.registers.sp = self.registers.sp.wrapping_sub(1),
            _ => {
                let value = self.registers.read_16bit(reg);
                self.registers.write_16bit(reg, value.wrapping_sub(1));
            }
        }
    }

    /// Execute a single CPU step and return the number of T-states consumed.
    ///
    /// Decodes the instruction at PC, executes it, and updates PC and cycle count.
    /// Returns 0 cycles if the CPU is halted (HALT mode waiting for interrupt).
    ///
    /// # HALT Behavior
    /// When a HALT instruction is encountered, the CPU sets `is_halted = true`.
    /// The CPU remains halted until an interrupt becomes pending.
    /// Call `wake_from_halt()` when implementing interrupt handling.
    ///
    /// # Unknown Instructions
    /// Panics on unknown opcodes to make missing implementations obvious during development.
    pub(crate) fn step(&mut self) -> u16 {
        if self.bus.any_interrupt_pending() {
            self.wake_from_halt();
        }

        if let Some(cycles) = self.handle_interrupts() {
            return cycles;
        }

        if self.is_halted {
            // CPU is halted and no interrupt to service, consume 4 T-cycles
            return 4;
        }

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
        let decoded = Instruction::from_byte(opcode_byte, prefixed);

        if let Some(instruction) = decoded {
            // Build a readable opcode string (e.g. "0x3E" or "0xCB37")
            let opcode_str = if prefixed {
                format!("0xCB{:02X}", opcode_byte)
            } else {
                format!("0x{:02X}", opcode_byte)
            };

            // Print a compact CPU state for debugging: PC, opcode, decoded instruction,
            // registers A,B,C,D,E,H,L, SP, HL and flags (raw F and booleans).
            if self.registers.pc < 0x0206 || self.registers.pc > 0x020D {
                println!(
                    "PC={:#06X} OPCODE={} INST={:?} \
    A={:#04X} F={:02X} Z={} N={} H={} C={} \
    B={:#04X} C={:#04X} D={:#04X} E={:#04X} H={:#04X} L={:#04X} \
    SP={:#06X} HL={:#06X}",
                    self.registers.pc,
                    opcode_str,
                    &instruction,
                    self.registers.a,
                    self.registers.f.to_byte(),
                    self.registers.f.zero,
                    self.registers.f.subtract,
                    self.registers.f.half_carry,
                    self.registers.f.carry,
                    self.registers.b,
                    self.registers.c,
                    self.registers.d,
                    self.registers.e,
                    self.registers.h,
                    self.registers.l,
                    self.registers.sp,
                    self.registers.get_hl()
                );
            }

            // HALT bug: When set, decrement PC before execution so operand reads
            // happen at the wrong address (byte after HALT is read twice)
            let halt_bug_active = self.halt_bug;
            if halt_bug_active {
                self.halt_bug = false;
                self.registers.pc = self.registers.pc.wrapping_sub(1);
            }

            // Execute the decoded instruction and advance PC
            let (next_pc, cycles) = self.execute(instruction);
            self.registers.pc = next_pc;

            // Handle EI delay: IME is enabled after the instruction following EI completes
            if self.ei_pending {
                self.ei_pending = false;
                self.interrupts_enabled = true;
            }

            cycles
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

    /// Check if the CPU is currently in HALT state.
    pub(crate) fn is_halted(&self) -> bool {
        self.is_halted
    }

    /// Wake the CPU from HALT state when an enabled interrupt becomes pending.
    fn wake_from_halt(&mut self) {
        self.is_halted = false;
    }

    /// Handle pending interrupts if IME is enabled.
    ///
    /// If an interrupt is pending and IME is set:
    /// 1. Disable IME
    /// 2. Push current PC onto stack
    /// 3. Clear the interrupt flag bit
    /// 4. Jump to interrupt handler
    ///
    /// Returns Some(cycles) if an interrupt was serviced, None otherwise.
    fn handle_interrupts(&mut self) -> Option<u16> {
        // Only service interrupts if IME is enabled
        if !self.interrupts_enabled {
            return None;
        }

        // Get the highest priority pending interrupt
        let interrupt = self.bus.interrupts.get_pending_interrupt()?;

        // Disable IME
        self.interrupts_enabled = false;

        // Push current PC onto stack
        self.push(self.registers.pc);

        // Service the interrupt (clears IF bit) and get handler address
        let handler_address = self.bus.interrupts.service_interrupt(interrupt);

        // Jump to handler
        self.registers.pc = handler_address;

        // Return the number of cycles consumed
        Some(INTERRUPT_CYCLES)
    }

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
        //
        // Using read_next_byte() keeps operand access consistent with other
        // places that read immediates (e.g. LD, ADDSP) and avoids subtle
        // mistakes where callers might also advance PC incorrectly.
        let offset_byte = self.read_next_byte() as i8;
        if should_jump {
            (self.registers.pc.wrapping_add(2) as i16).wrapping_add(offset_byte as i16) as u16
        } else {
            self.registers.pc.wrapping_add(2)
        }
    }

    /// Read a 16-bit value from a stack target register pair (BC, DE, HL, or AF).
    fn read_stack_target(&self, target: StackTarget) -> u16 {
        match target {
            StackTarget::BC => self.registers.get_bc(),
            StackTarget::DE => self.registers.get_de(),
            StackTarget::HL => self.registers.get_hl(),
            StackTarget::AF => self.registers.get_af(),
        }
    }

    /// Write a 16-bit value to a stack target register pair (BC, DE, HL, or AF).
    fn write_stack_target(&mut self, target: StackTarget, value: u16) {
        match target {
            StackTarget::BC => self.registers.set_bc(value),
            StackTarget::DE => self.registers.set_de(value),
            StackTarget::HL => self.registers.set_hl(value),
            StackTarget::AF => self.registers.set_af(value),
        }
    }
    /// Push a 16-bit value onto the stack (decrements SP twice, MSB written first).
    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus
            .write_byte(self.registers.sp, ((value & 0xFF00) >> 8) as u8);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus.write_byte(self.registers.sp, (value & 0xFF) as u8);
    }

    /// Pop a 16-bit value from the stack (increments SP twice, LSB read first).
    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    /// Execute a CALL instruction: conditionally push return address and jump.
    /// Returns target address if should_jump is true, otherwise PC+3 (skip instruction).
    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.registers.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    /// Read the next byte from memory at PC+1 (typically an immediate operand).
    fn read_next_byte(&mut self) -> u8 {
        self.bus.read_byte(self.registers.pc + 1)
    }

    /// Read the next word (16-bit value) from memory at PC+1 (little-endian: LSB at PC+1, MSB at PC+2).
    fn read_next_word(&mut self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.registers.pc + 1) as u16;
        let most_significant_byte = self.bus.read_byte(self.registers.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    }

    /// Execute a RET instruction: conditionally pop return address from stack.
    /// Returns the popped address if should_jump is true, otherwise PC+1 (skip instruction).
    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.registers.pc.wrapping_add(1)
        }
    }

    /// Read a value from a CB-prefixed instruction target (register or memory at HL).
    fn read_prefix_target(&mut self, target: crate::instructions::targets::PrefixTarget) -> u8 {
        if let Some(reg) = target.to_register8() {
            self.registers.read_8bit(reg)
        } else {
            self.bus.read_byte(self.registers.get_hl())
        }
    }

    /// Write a value to a CB-prefixed instruction target (register or memory at HL).
    fn write_prefix_target(
        &mut self,
        target: crate::instructions::targets::PrefixTarget,
        value: u8,
    ) {
        if let Some(reg) = target.to_register8() {
            self.registers.write_8bit(reg, value);
        } else {
            self.bus.write_byte(self.registers.get_hl(), value);
        }
    }
}
