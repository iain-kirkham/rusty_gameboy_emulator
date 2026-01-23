use crate::instructions::{
    ArithmeticTarget, IncDecTarget, Instruction, JumpTest, LoadByteSource, LoadByteTarget,
    LoadType, LoadWordSource, LoadWordTarget, StackTarget,
};
use crate::memory_bus::MemoryBus;
use crate::register::{self, Register16, Registers};

pub(crate) struct CPU {
    pub registers: register::Registers, // All general-purpose registers and flags
    pub bus: MemoryBus,                 // Memory bus
    is_halted: bool,                    // Whether the CPU is in HALT.
    pub interrupts_enabled: bool,
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
            // HALT: Stops CPU execution until an interrupt occurs.
            Instruction::HALT => {
                self.is_halted = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // DI/EI: Interrupt control instructions
            Instruction::DI => {
                self.interrupts_enabled = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::EI => {
                self.interrupts_enabled = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // Arithmetic operations on A register
            Instruction::ADD(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.add(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::ADC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.adc(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SUB(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sub(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SBC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sbc(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            // Logical operations on A register
            Instruction::AND(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.and(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::OR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.or(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::XOR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.xor(value);
                self.registers.a = new_value;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CP(target) => {
                let value = self.get_arithmetic_target(target);
                self.cp(value);
                (self.registers.pc.wrapping_add(1), 4)
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
                    let cycles = match source {
                        LoadByteSource::D8 => 8,    // LD r, d8
                        LoadByteSource::A16I => 16, // LD A,(a16)
                        LoadByteSource::A8I => 12,  // LD A,(0xFF00+o)
                        LoadByteSource::HLI => 8,   // LD r,(HL)
                        LoadByteSource::HLI_INC => 8,
                        LoadByteSource::HLI_DEC => 8,
                        LoadByteSource::BCI => 8,
                        LoadByteSource::A => 4,
                        LoadByteSource::B => 4,
                        LoadByteSource::C => 4,
                        LoadByteSource::D => 4,
                        LoadByteSource::E => 4,
                        LoadByteSource::H => 4,
                        LoadByteSource::L => 4,
                    };

                    (
                        self.registers
                            .pc
                            .wrapping_add(self.get_load_byte_pc_increment(source)),
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
                    };
                    let pc_inc = match source {
                        LoadWordSource::D16 => 3, // opcode + 2-byte immediate
                        LoadWordSource::HL | LoadWordSource::SP => 1,
                    };
                    let cycles = match source {
                        LoadWordSource::D16 => 12, // LD rr,d16
                        LoadWordSource::SP => 8,
                        LoadWordSource::HL => 8,
                    };
                    (self.registers.pc.wrapping_add(pc_inc), cycles)
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
                let cycles = if should { 20 } else { 8 };
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
                // Decimal Adjust Accumulator
                let mut a = self.registers.a;
                let mut adjust = 0;

                if self.registers.f.half_carry || (!self.registers.f.subtract && (a & 0x0F) > 9) {
                    adjust |= 0x06;
                }

                if self.registers.f.carry || (!self.registers.f.subtract && a > 0x99) {
                    adjust |= 0x60;
                    self.registers.f.carry = true;
                }

                if self.registers.f.subtract {
                    a = a.wrapping_sub(adjust);
                } else {
                    a = a.wrapping_add(adjust);
                }

                self.registers.a = a;
                self.registers.f.zero = a == 0;
                self.registers.f.half_carry = false;
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
                let offset = self.read_next_byte() as i8 as i16 as u16;
                let sp = self.registers.sp;
                let result = sp.wrapping_add(offset);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (sp & 0x0F) + (offset & 0x0F) > 0x0F;
                self.registers.f.carry = (sp & 0xFF) + (offset & 0xFF) > 0xFF;

                self.registers.sp = result;
                (self.registers.pc.wrapping_add(2), 16)
            }
            Instruction::LDHLSP => {
                // LDHLSP: Load HL with SP + signed 8-bit immediate (flags set from lower 8 bits)
                let offset = self.read_next_byte() as i8 as i16 as u16;
                let sp = self.registers.sp;
                let result = sp.wrapping_add(offset);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (sp & 0x0F) + (offset & 0x0F) > 0x0F;
                self.registers.f.carry = (sp & 0xFF) + (offset & 0xFF) > 0xFF;

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
            LoadByteSource::BCI => self.bus.read_byte(self.registers.get_bc()),
            LoadByteSource::A16I => {
                let address = self.read_next_word();
                self.bus.read_byte(address)
            }
            LoadByteSource::A8I => {
                let offset = self.read_next_byte();
                let address = 0xFF00 + offset as u16;
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
        }
    }

    /// Calculate how much the PC should advance based on the load source.
    /// Different sources consume different numbers of bytes (opcode + operands).
    fn get_load_byte_pc_increment(&self, source: LoadByteSource) -> u16 {
        match source {
            LoadByteSource::D8 => 2,   // opcode + 1 byte immediate
            LoadByteSource::A16I => 3, // opcode + 2 byte immediate
            LoadByteSource::A8I => 2,  // opcode + 1 byte offset (high RAM)
            _ => 1,
        }
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
        }
    }

    /// Perform 8-bit addition: A += value (sets all CPU flags).
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        let half_carry = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;
        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit addition with carry: A += value + carry_flag (sets all CPU flags).
    fn adc(&mut self, value: u8) -> u8 {
        let carry = if self.registers.f.carry { 1 } else { 0 };

        let (temp, overflow1) = self.registers.a.overflowing_add(value);
        let (new_value, overflow2) = temp.overflowing_add(carry);

        // Check for half carry: carry from bit 3 to bit 4
        let half_carry = ((self.registers.a & 0x0F) + (value & 0x0F) + carry) > 0x0F;
        let did_overflow = overflow1 || overflow2;

        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit subtraction: A -= value (sets all CPU flags).
    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        let half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
        self.set_arithmetic_flags(new_value, true, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit subtraction with carry: A -= value - carry_flag (sets all CPU flags).
    fn sbc(&mut self, value: u8) -> u8 {
        let carry = if self.registers.f.carry { 1 } else { 0 };

        let (temp, overflow1) = self.registers.a.overflowing_sub(value);
        let (new_value, overflow2) = temp.overflowing_sub(carry);

        // Check for half carry: borrow from bit 4 to bit 3
        let half_carry = (self.registers.a & 0x0F) < ((value & 0x0F) + carry);
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
        self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
    }

    /// Increment an 8-bit value (sets Z, N=false, H flags; doesn't affect C).
    fn inc_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;
        new_value
    }

    /// Decrement an 8-bit value (sets Z, N=true, H flags; doesn't affect C).
    fn dec_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0;
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
        if self.is_halted {
            return 0;
        }
        let mut instruction_byte = self.bus.read_byte(self.registers.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.registers.pc + 1);
        }

        let (next_pc, cycles) =
            if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
                self.execute(instruction)
            } else {
                let instruction_str = if prefixed {
                    format!("0xCB{:02X}", instruction_byte)
                } else {
                    format!("0x{:02X}", instruction_byte)
                };
                panic!(
                    "Unknown instruction {} at PC=0x{:04X}",
                    instruction_str, self.registers.pc
                );
            };

        self.registers.pc = next_pc;

        // Return the number of T-states consumed so the caller can advance timers/PPU/DMA.
        cycles
    }

    /// Check if the CPU is currently in HALT state.
    pub(crate) fn is_halted(&self) -> bool {
        self.is_halted
    }

    /// Wake the CPU from HALT state when an enabled interrupt becomes pending.
    /// TODO: Call this method when interrupt handling is fully implemented.
    #[allow(dead_code)]
    fn wake_from_halt(&mut self) {
        self.is_halted = false;
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
        if should_jump {
            let offset_byte = self.bus.read_byte(self.registers.pc + 1) as i8;
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
