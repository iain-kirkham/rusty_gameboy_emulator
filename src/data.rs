use crate::register::Registers;
use crate::{gpu, register};

enum Instruction {
    ADD(ArithmeticTarget),
    SUB(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget),
    JP(JumpTest),
    LD(LoadType),
    POP(StackTarget),
    PUSH(StackTarget),
    CALL(JumpTest),
    RLC(PrefixTarget),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    RET(JumpTest),
    NOP,
    HALT,
}

enum StackTarget {
    BC,
    DE,
    HL,
    AF,
}

enum PrefixTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    SP,
}

enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}
enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLI,
}
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
}

struct CPU {
    registers: register::Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
    is_halted: bool,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
    gpu: gpu::GPU,
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => self.memory[address], // ROM
            0x8000..=0x9FFF => self.gpu.read_vram(address - 0x8000), // VRAM
            0xA000..=0xBFFF => self.memory[address], // External RAM
            0xC000..=0xFDFF => self.memory[address], // Work RAM
            0xFE00..=0xFE9F => self.memory[address], // OAM
            0xFF00..=0xFF7F => self.memory[address], // I/O Registers
            0xFF80..=0xFFFE => self.memory[address], // High RAM
            0xFFFF => self.memory[address],          // Interrupt Enable Register
            _ => 0xFF,                               // Unmapped areas return 0xFF
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => {} // ROM ignore writes
            0x8000..=0x9FFF => self.gpu.write_vram(address - 0x8000, value),
            0xA000..=0xBFFF => self.memory[address] = value,
            0xC000..=0xFDFF => self.memory[address] = value,
            0xFE00..=0xFE9F => self.memory[address] = value,
            0xFF00..=0xFF7F => self.memory[address] = value,
            0xFF80..=0xFFFE => self.memory[address] = value,
            0xFFFF => self.memory[address] = value,
            _ => {} // Ignore writes to unmapped areas
        }
    }
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            _ =>
            /* TODO: Add mapping for rest of instructions */
            {
                None
            }
        }
    }

    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x76 => Some(Instruction::HALT),

            //INC 8bit
            0x04 => Some(Instruction::INC(IncDecTarget::B)),
            0x0C => Some(Instruction::INC(IncDecTarget::C)),
            0x14 => Some(Instruction::INC(IncDecTarget::D)),
            0x1C => Some(Instruction::INC(IncDecTarget::E)),
            0x24 => Some(Instruction::INC(IncDecTarget::H)),
            0x2C => Some(Instruction::INC(IncDecTarget::L)),
            0x3C => Some(Instruction::INC(IncDecTarget::A)),

            //INC 16bit
            0x03 => Some(Instruction::INC(IncDecTarget::BC)),
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x23 => Some(Instruction::INC(IncDecTarget::HL)),
            0x33 => Some(Instruction::INC(IncDecTarget::SP)),

            0x05 => Some(Instruction::DEC(IncDecTarget::B)),
            0x0D => Some(Instruction::DEC(IncDecTarget::C)),
            0x15 => Some(Instruction::DEC(IncDecTarget::D)),
            0x1D => Some(Instruction::DEC(IncDecTarget::E)),
            0x25 => Some(Instruction::DEC(IncDecTarget::H)),
            0x2D => Some(Instruction::DEC(IncDecTarget::L)),
            0x3D => Some(Instruction::DEC(IncDecTarget::A)),

            0x0B => Some(Instruction::DEC(IncDecTarget::BC)),
            0x1B => Some(Instruction::DEC(IncDecTarget::DE)),
            0x2B => Some(Instruction::DEC(IncDecTarget::HL)),
            0x3B => Some(Instruction::DEC(IncDecTarget::SP)),


            //ADD A instructions
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),

            //SUB A instructions
            0x90 => Some(Instruction::SUB(ArithmeticTarget::B)),
            0x91 => Some(Instruction::SUB(ArithmeticTarget::C)),
            0x92 => Some(Instruction::SUB(ArithmeticTarget::D)),
            0x93 => Some(Instruction::SUB(ArithmeticTarget::E)),
            0x94 => Some(Instruction::SUB(ArithmeticTarget::H)),
            0x95 => Some(Instruction::SUB(ArithmeticTarget::L)),
            0x97 => Some(Instruction::SUB(ArithmeticTarget::A)),

            0xA0 => Some(Instruction::AND(ArithmeticTarget::B)),
            0xA1 => Some(Instruction::AND(ArithmeticTarget::C)),
            0xA2 => Some(Instruction::AND(ArithmeticTarget::D)),
            0xA3 => Some(Instruction::AND(ArithmeticTarget::E)),
            0xA4 => Some(Instruction::AND(ArithmeticTarget::H)),
            0xA5 => Some(Instruction::AND(ArithmeticTarget::L)),
            0xA7 => Some(Instruction::AND(ArithmeticTarget::A)),

            0xB0 => Some(Instruction::OR(ArithmeticTarget::B)),
            0xB1 => Some(Instruction::OR(ArithmeticTarget::C)),
            0xB2 => Some(Instruction::OR(ArithmeticTarget::D)),
            0xB3 => Some(Instruction::OR(ArithmeticTarget::E)),
            0xB4 => Some(Instruction::OR(ArithmeticTarget::H)),
            0xB5 => Some(Instruction::OR(ArithmeticTarget::L)),
            0xB7 => Some(Instruction::OR(ArithmeticTarget::A)),

            0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
            0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
            0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
            0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
            0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
            0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
            0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),

            0xB8 => Some(Instruction::CP(ArithmeticTarget::B)),
            0xB9 => Some(Instruction::CP(ArithmeticTarget::C)),
            0xBA => Some(Instruction::CP(ArithmeticTarget::D)),
            0xBB => Some(Instruction::CP(ArithmeticTarget::E)),
            0xBC => Some(Instruction::CP(ArithmeticTarget::H)),
            0xBD => Some(Instruction::CP(ArithmeticTarget::L)),
            0xBF => Some(Instruction::CP(ArithmeticTarget::A)),

            _ =>
            /* TODO: Add mapping for rest of instructions */
            {
                None
            }
        }
    }
}
impl CPU {
    fn new(bus: MemoryBus) -> CPU {
        CPU {
            registers: Registers::new(),
            pc: 0x0100,
            sp: 0xFFFE,
            bus,
            is_halted: false,
        }
    }
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::NOP => self.pc.wrapping_add(1),
            Instruction::HALT => {
                self.is_halted = true;
                self.pc.wrapping_add(1)
            }
            Instruction::ADD(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.add(value);
                self.registers.a = new_value;
                self.pc.wrapping_add(1)
            }
            Instruction::SUB(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.sub(value);
                self.registers.a = new_value;
                self.pc.wrapping_add(1)
            }
            Instruction::AND(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.and(value);
                self.registers.a = new_value;
                self.pc.wrapping_add(1)
            }
            Instruction::XOR(target) => {
                let value = self.get_arithmetic_target(target);
                let new_value = self.xor(value);
                self.registers.a = new_value;
                self.pc.wrapping_add(1)
            }
            Instruction::CP(target) => {
                let value = self.get_arithmetic_target(target);
                self.cp(value);
                self.pc.wrapping_add(1)
            }
            Instruction::INC(target) => {
                match target {
                    // 8-bit INC operations
                    IncDecTarget::A => self.registers.a = self.inc_8bit(self.registers.a),
                    IncDecTarget::B => self.registers.b = self.inc_8bit(self.registers.b),
                    IncDecTarget::C => self.registers.c = self.inc_8bit(self.registers.c),
                    IncDecTarget::D => self.registers.d = self.inc_8bit(self.registers.d),
                    IncDecTarget::E => self.registers.e = self.inc_8bit(self.registers.e),
                    IncDecTarget::H => self.registers.h = self.inc_8bit(self.registers.h),
                    IncDecTarget::L => self.registers.l = self.inc_8bit(self.registers.l),

                    // 16-bit INC operations
                    IncDecTarget::BC => {
                        let value = self.registers.get_bc().wrapping_add(1);
                        self.registers.set_bc(value);
                    }
                    IncDecTarget::DE => {
                        let value = self.registers.get_de().wrapping_add(1);
                        self.registers.set_de(value);
                    }
                    IncDecTarget::HL => {
                        let value = self.registers.get_hl().wrapping_add(1);
                        self.registers.set_hl(value);
                    }
                    IncDecTarget::SP => {
                        self.sp = self.sp.wrapping_add(1);
                    }
                }
                self.pc.wrapping_add(1)
            }

            Instruction::DEC(target) => {
                match target {
                    // 8-bit DEC operations
                    IncDecTarget::A => self.registers.a = self.dec_8bit(self.registers.a),
                    IncDecTarget::B => self.registers.b = self.dec_8bit(self.registers.b),
                    IncDecTarget::C => self.registers.c = self.dec_8bit(self.registers.c),
                    IncDecTarget::D => self.registers.d = self.dec_8bit(self.registers.d),
                    IncDecTarget::E => self.registers.e = self.dec_8bit(self.registers.e),
                    IncDecTarget::H => self.registers.h = self.dec_8bit(self.registers.h),
                    IncDecTarget::L => self.registers.l = self.dec_8bit(self.registers.l),

                    // 16-bit DEC operations
                    IncDecTarget::BC => {
                        let value = self.registers.get_bc().wrapping_sub(1);
                        self.registers.set_bc(value);
                    }
                    IncDecTarget::DE => {
                        let value = self.registers.get_de().wrapping_sub(1);
                        self.registers.set_de(value);
                    }
                    IncDecTarget::HL => {
                        let value = self.registers.get_hl().wrapping_sub(1);
                        self.registers.set_hl(value);
                    }
                    IncDecTarget::SP => {
                        self.sp = self.sp.wrapping_sub(1);
                    }
                }
                self.pc.wrapping_add(1)
            }
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                        _ => {
                            panic!("TODO: implement other sources")
                        }
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                        _ => {
                            panic!("TODO: implement other targets")
                        }
                    };
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                _ => {
                    panic!("TODO: implement other load types")
                }
            },
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::BC => self.registers.get_bc(),
                    _ => {
                        panic!("TODO: support more targets")
                    }
                };
                self.push(value);
                self.pc.wrapping_add(1)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::BC => self.registers.set_bc(result),
                    _ => {
                        panic!("TODO: support more targets")
                    }
                };
                self.pc.wrapping_add(1)
            }
            Instruction::CALL(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    _ => {
                        panic!("TODO: support more conditions")
                    }
                };
                self.call(jump_condition)
            }
            Instruction::RET(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    _ => {
                        panic!("TODO: support more conditions")
                    }
                };
                self.return_(jump_condition)
            }
            _ => {
                /* TODO: support more instructions */
                self.pc.wrapping_add(1)
            }
        }
    }

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

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        // Half carry occurs when borrowing from bit 4
        self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
        new_value
    }

    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true; // Always set for AND
        self.registers.f.carry = false;
        new_value
    }

    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }

    fn cp(&mut self, value: u8) {
        let (result, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
        // Note: We don't store the result in A register for CP
    }

    fn inc_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        // Half carry occurs when carry from bit 3 to bit 4
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;
        // Carry flag is not affected by INC
        new_value
    }

    fn dec_8bit(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        // Half carry occurs when borrowing from bit 4
        self.registers.f.half_carry = (value & 0x0F) == 0;
        // Carry flag is not affected by DEC
        new_value
    }


    fn step(&mut self) {
        // If CPU is halted, don't execute instructions until an interrupt occurs
        if self.is_halted {
            //implement interrupts here
            return;
        }
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            let description = format!(
                "0x{}{:x}",
                if prefixed { "cb" } else { "" },
                instruction_byte
            );
            panic!("Unknown instruction found for: {}", description)
        };

        self.pc = next_pc;
    }

    fn is_halted(&self) -> bool {
        self.is_halted
    }

    fn wake_from_halt(&mut self) {
        self.is_halted = false;
    }
    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // Gameboy is little endian so read pc + 2 as most significant bit
            // and pc + 1 as least significant bit
            let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
            (most_significant_byte << 8) | least_significant_byte
        } else {
            // If we don't jump we need to still move the program
            // counter forward by 3 since the jump instruction is
            // 3 bytes wide (1 byte for tag and 2 bytes for jump address)
            self.pc.wrapping_add(3)
        }
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    fn read_next_word(&self) -> u16 {
        // Game Boy is little endian, so least significant byte comes first
        let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
        let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    }

    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }
}
