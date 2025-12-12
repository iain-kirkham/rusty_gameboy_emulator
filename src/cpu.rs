use crate::instructions::{
    ArithmeticTarget, IncDecTarget, Instruction, JumpTest, LoadByteSource, LoadByteTarget,
    LoadType, LoadWordSource, LoadWordTarget, StackTarget,
};
use crate::memory_bus::MemoryBus;
use crate::register::{self, Register16, Registers};

pub(crate) struct CPU {
    pub registers: register::Registers, // All general-purpose registers and flags
    pub bus: MemoryBus,                 // Memory bus
    pub is_halted: bool,                // Whether the CPU is in HALT.
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
    /// Execute a decoded instruction and return the next program counter.
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::NOP => self.registers.pc.wrapping_add(1), // Do nothing
            // Halt: Stops CPU execution until an interrupt occurs.
            Instruction::HALT => {
                self.is_halted = true;
                self.registers.pc.wrapping_add(1)
            }
            // Interrupt control instructions
            Instruction::DI => {
                self.interrupts_enabled = false;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::EI => {
                self.interrupts_enabled = true;
                self.registers.pc.wrapping_add(1)
            }
            // Arithmetic and Logic instructions.
            Instruction::ADD(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.add(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::ADC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.adc(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::SUB(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sub(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::SBC(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sbc(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::AND(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.and(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::OR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.or(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::XOR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.xor(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::CP(target) => {
                let value = self.get_arithmetic_target(target);
                self.cp(value);
                self.registers.pc.wrapping_add(1)
            }
            // Increment and decrement instructions.
            Instruction::INC(target) => {
                match target {
                    IncDecTarget::Reg8(reg) => {
                        let value = self.registers.read_8bit(reg);
                        let new_value = self.inc_8bit(value);
                        self.registers.write_8bit(reg, new_value);
                    }
                    IncDecTarget::Reg16(reg) => {
                        self.inc_16bit(reg);
                    }
                }
                self.registers.pc.wrapping_add(1)
            }
            Instruction::DEC(target) => {
                match target {
                    IncDecTarget::Reg8(reg) => {
                        let value = self.registers.read_8bit(reg);
                        let new_value = self.dec_8bit(value);
                        self.registers.write_8bit(reg, new_value);
                    }
                    IncDecTarget::Reg16(reg) => {
                        self.dec_16bit(reg);
                    }
                }
                self.registers.pc.wrapping_add(1)
            }
            // Control flow instructions.
            Instruction::JP(test) => self.jump(self.should_jump(&test)),
            Instruction::JR(test) => self.jump_relative(self.should_jump(&test)),
            // Load instructions.
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = self.read_byte_source(source);
                    self.write_byte_target(target, source_value);
                    self.registers
                        .pc
                        .wrapping_add(self.get_load_byte_pc_increment(source))
                }

                // 16-bit load
                LoadType::Word(target, source) => {
                    let source_value = match source {
                        LoadWordSource::D16 => self.read_next_word(),
                        LoadWordSource::SP => self.registers.sp,
                    };
                    match target {
                        LoadWordTarget::HL => self.registers.set_hl(source_value),
                        LoadWordTarget::BC => self.registers.set_bc(source_value),
                        LoadWordTarget::DE => self.registers.set_de(source_value),
                        LoadWordTarget::SP => self.registers.sp = source_value,
                    };
                    self.registers.pc.wrapping_add(3) // opcode + 2-byte immediate
                }
            },
            // Stack instructions.
            Instruction::PUSH(target) => {
                let value = self.read_stack_target(target);
                self.push(value);
                self.registers.pc.wrapping_add(1)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                self.write_stack_target(target, result);
                self.registers.pc.wrapping_add(1)
            }
            Instruction::CALL(test) => self.call(self.should_jump(&test)),
            Instruction::RET(test) => self.return_(self.should_jump(&test)),
            // CB prefix instructions.
            Instruction::RLC(_) => self.registers.pc.wrapping_add(2),
        }
    }

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
        }
    }

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
        }
    }

    fn get_load_byte_pc_increment(&self, source: LoadByteSource) -> u16 {
        match source {
            LoadByteSource::D8 => 2,   // opcode + 1 byte immediate
            LoadByteSource::A16I => 3, // opcode + 2 byte immediate
            _ => 1,
        }
    }

    /// Fetches the value from a given 8-bit arithmetic target (A, B, C, D, E, H, L).
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

    /// Performs an 8-bit addition on the A register, setting flags.
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        let half_carry = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;
        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Performs an 8-bit addition with carry on the A register, setting flags.
    fn adc(&mut self, value: u8) -> u8 {
        let carry = if self.registers.f.carry { 1 } else { 0 };

        // First add the value to A
        let (temp, overflow1) = self.registers.a.overflowing_add(value);
        // Then add the carry
        let (new_value, overflow2) = temp.overflowing_add(carry);

        // Check for half carry: carry from bit 3 to bit 4
        let half_carry = ((self.registers.a & 0x0F) + (value & 0x0F) + carry) > 0x0F;

        // Carry flag is set if either addition overflowed
        let did_overflow = overflow1 || overflow2;

        self.set_arithmetic_flags(new_value, false, did_overflow, half_carry);
        new_value
    }

    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        let half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
        self.set_arithmetic_flags(new_value, true, did_overflow, half_carry);
        new_value
    }

    /// Subtract with carry
    fn sbc(&mut self, value: u8) -> u8 {
        let carry = if self.registers.f.carry { 1 } else { 0 };

        // First subtract the value from A
        let (temp, overflow1) = self.registers.a.overflowing_sub(value);
        // Then subtract the carry
        let (new_value, overflow2) = temp.overflowing_sub(carry);

        // Check for half carry: borrow from bit 4 to bit 3
        let half_carry = (self.registers.a & 0x0F) < ((value & 0x0F) + carry);

        // Carry flag is set if either subtraction underflowed
        let did_overflow = overflow1 || overflow2;

        self.set_arithmetic_flags(new_value, true, did_overflow, half_carry);
        new_value
    }

    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.set_logic_flags(new_value, true);
        new_value
    }

    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.set_logic_flags(new_value, false);
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.set_logic_flags(new_value, false);
        new_value
    }

    fn set_logic_flags(&mut self, result: u8, half_carry: bool) {
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = half_carry;
        self.registers.f.carry = false;
    }

    fn set_arithmetic_flags(&mut self, result: u8, subtract: bool, carry: bool, half_carry: bool) {
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = subtract;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = half_carry;
    }

    fn cp(&mut self, value: u8) {
        let (result, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
    }

    fn inc_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;
        new_value
    }

    fn dec_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0;
        new_value
    }

    fn inc_16bit(&mut self, reg: Register16) {
        match reg {
            Register16::SP => self.registers.sp = self.registers.sp.wrapping_add(1),
            _ => {
                let value = self.registers.read_16bit(reg);
                self.registers.write_16bit(reg, value.wrapping_add(1));
            }
        }
    }

    fn dec_16bit(&mut self, reg: Register16) {
        match reg {
            Register16::SP => self.registers.sp = self.registers.sp.wrapping_sub(1),
            _ => {
                let value = self.registers.read_16bit(reg);
                self.registers.write_16bit(reg, value.wrapping_sub(1));
            }
        }
    }

    /// Execute a single CPU step.
    ///
    /// - HALT handling: the CPU will set `is_halted = true` on a HALT instruction.
    ///   When interrupts are implemented, HALT should be cleared when any enabled
    ///   interrupt becomes pending. `wake_from_halt()` exists for that purpose.
    /// - Unknown opcode: currently this function panics on an unknown/unsupported
    ///   opcode to make missing implementations obvious during development.
    pub(crate) fn step(&mut self) {
        if self.is_halted {
            return;
        }
        let mut instruction_byte = self.bus.read_byte(self.registers.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.registers.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            panic!(
                "Unknown instruction 0x{}{:02X} at PC=0x{:04X}",
                if prefixed { "CB" } else { "" },
                instruction_byte,
                self.registers.pc
            );
        };

        self.registers.pc = next_pc;
    }

    pub(crate) fn is_halted(&self) -> bool {
        self.is_halted
    }

    /// Wakes the CPU from HALT state when an interrupt occurs.
    /// TODO: Call this when interrupt handling is implemented.
    #[allow(dead_code)]
    fn wake_from_halt(&mut self) {
        self.is_halted = false;
    }

    fn should_jump(&self, test: &JumpTest) -> bool {
        match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        }
    }
    fn jump(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.read_next_word()
        } else {
            self.registers.pc.wrapping_add(3)
        }
    }

    fn jump_relative(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            let offset_byte = self.bus.read_byte(self.registers.pc + 1) as i8; // Read as signed 8-bit
                                                                               // PC is already at the instruction start, add 2 (opcode + offset) and then the signed offset
            (self.registers.pc.wrapping_add(2) as i16).wrapping_add(offset_byte as i16) as u16
        } else {
            self.registers.pc.wrapping_add(2) // 1 byte opcode + 1 byte offset
        }
    }

    fn read_stack_target(&self, target: StackTarget) -> u16 {
        match target {
            StackTarget::BC => self.registers.get_bc(),
            StackTarget::DE => self.registers.get_de(),
            StackTarget::HL => self.registers.get_hl(),
            StackTarget::AF => self.registers.get_af(),
        }
    }

    fn write_stack_target(&mut self, target: StackTarget, value: u16) {
        match target {
            StackTarget::BC => self.registers.set_bc(value),
            StackTarget::DE => self.registers.set_de(value),
            StackTarget::HL => self.registers.set_hl(value),
            StackTarget::AF => self.registers.set_af(value),
        }
    }
    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus
            .write_byte(self.registers.sp, ((value & 0xFF00) >> 8) as u8);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus.write_byte(self.registers.sp, (value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.registers.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.registers.pc + 1);
        byte
    }

    fn read_next_word(&mut self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.registers.pc + 1) as u16;
        let most_significant_byte = self.bus.read_byte(self.registers.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    }

    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.registers.pc.wrapping_add(1)
        }
    }
}
