use super::CPU;
use super::HaltState;
use super::{
    ArithmeticOps, ControlFlowOps, FetchTraceOps, LoadOps, PrefixOps, StackInterruptOps,
};
use crate::flag_helpers as fh;
use crate::instructions::{
    ArithmeticTarget, IncDecTarget, Instruction, JumpTest, LoadType, LoadWordSource,
    LoadWordTarget, PrefixTarget,
};
use crate::register::Register16;

impl CPU {
    /// Execute a decoded instruction and return (next_pc, cycles_in_tstates).
    ///
    /// Cycle counts here are provided so the rest of the emulator can
    /// step timers/PPU/DMA appropriately. These values are the T-states for
    /// the instruction.
    pub(super) fn execute(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::NOP
            | Instruction::STOP
            | Instruction::HALT
            | Instruction::DI
            | Instruction::EI => self.execute_core_control(instruction),
            Instruction::ADD(_)
            | Instruction::ADC(_)
            | Instruction::SUB(_)
            | Instruction::SBC(_)
            | Instruction::AND(_)
            | Instruction::OR(_)
            | Instruction::XOR(_)
            | Instruction::CP(_) => self.execute_arithmetic_instruction(instruction),
            Instruction::INC(_) | Instruction::DEC(_) => self.execute_inc_dec_instruction(instruction),
            Instruction::JP(_)
            | Instruction::JR(_)
            | Instruction::CALL(_)
            | Instruction::RET(_)
            | Instruction::RETI
            | Instruction::RST(_)
            | Instruction::JP_HL => self.execute_control_flow_instruction(instruction),
            Instruction::LD(_) => self.execute_load_instruction(instruction),
            Instruction::PUSH(_) | Instruction::POP(_) => self.execute_stack_instruction(instruction),
            Instruction::RLCA
            | Instruction::RRCA
            | Instruction::RLA
            | Instruction::RRA
            | Instruction::DAA
            | Instruction::CPL
            | Instruction::SCF
            | Instruction::CCF
            | Instruction::ADDHL(_)
            | Instruction::ADDSP
            | Instruction::LDHLSP => self.execute_misc_instruction(instruction),
            Instruction::RLC(_)
            | Instruction::RRC(_)
            | Instruction::RL(_)
            | Instruction::RR(_)
            | Instruction::SLA(_)
            | Instruction::SRA(_)
            | Instruction::SWAP(_)
            | Instruction::SRL(_)
            | Instruction::BIT(_, _)
            | Instruction::RES(_, _)
            | Instruction::SET(_, _) => self.execute_prefixed_instruction(instruction),
        }
    }

    fn execute_core_control(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::NOP => self.instruction_result(1, 4),
            Instruction::STOP => {
                self.halt_state = HaltState::Halted;
                self.instruction_result(2, 4)
            }
            Instruction::HALT => {
                if !self.interrupts_enabled && self.bus.any_interrupt_pending() {
                    self.halt_state = HaltState::HaltBugPending;
                } else {
                    self.halt_state = HaltState::Halted;
                }
                self.instruction_result(1, 4)
            }
            Instruction::DI => {
                self.interrupts_enabled = false;
                self.ei_pending = false;
                self.instruction_result(1, 4)
            }
            Instruction::EI => {
                self.ei_pending = true;
                self.instruction_result(1, 4)
            }
            _ => unreachable!("execute_core_control called with non-control instruction"),
        }
    }

    fn execute_arithmetic_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        enum ArithmeticOp {
            Add,
            Adc,
            Sub,
            Sbc,
            And,
            Or,
            Xor,
            Cp,
        }

        let (target, op) = match instruction {
            Instruction::ADD(target) => (target, ArithmeticOp::Add),
            Instruction::ADC(target) => (target, ArithmeticOp::Adc),
            Instruction::SUB(target) => (target, ArithmeticOp::Sub),
            Instruction::SBC(target) => (target, ArithmeticOp::Sbc),
            Instruction::AND(target) => (target, ArithmeticOp::And),
            Instruction::OR(target) => (target, ArithmeticOp::Or),
            Instruction::XOR(target) => (target, ArithmeticOp::Xor),
            Instruction::CP(target) => (target, ArithmeticOp::Cp),
            _ => unreachable!("execute_arithmetic_instruction called with non-arithmetic instruction"),
        };

        let value = self.get_arithmetic_target(target);

        match op {
            ArithmeticOp::Add => self.registers.a = self.add(value),
            ArithmeticOp::Adc => self.registers.a = self.adc(value),
            ArithmeticOp::Sub => self.registers.a = self.sub(value),
            ArithmeticOp::Sbc => self.registers.a = self.sbc(value),
            ArithmeticOp::And => self.registers.a = self.and(value),
            ArithmeticOp::Or => self.registers.a = self.or(value),
            ArithmeticOp::Xor => self.registers.a = self.xor(value),
            ArithmeticOp::Cp => {
                self.cp(value);
            }
        }

        let (pc_inc, cycles) = Self::arithmetic_timing(target);
        self.instruction_result(pc_inc, cycles)
    }

    fn arithmetic_timing(target: ArithmeticTarget) -> (u16, u16) {
        match target {
            ArithmeticTarget::D8 => (2, 8),
            ArithmeticTarget::HLI => (1, 8),
            _ => (1, 4),
        }
    }

    fn instruction_result(&self, pc_increment: u16, cycles: u16) -> (u16, u16) {
        (self.registers.pc.wrapping_add(pc_increment), cycles)
    }

    fn absolute_result(next_pc: u16, cycles: u16) -> (u16, u16) {
        (next_pc, cycles)
    }

    fn branch_cycles(taken: bool, taken_cycles: u16, not_taken_cycles: u16) -> u16 {
        if taken {
            taken_cycles
        } else {
            not_taken_cycles
        }
    }

    fn compute_sp_plus_offset(sp: u16, offset: i8) -> (u16, bool, bool) {
        let result = fh::add_sp_signed(sp, offset);
        let half_carry = fh::half_carry_add_sp(sp, offset);
        let carry = fh::carry_add_sp(sp, offset);
        (result, half_carry, carry)
    }

    fn prefixed_result_for_target(&self, target: &PrefixTarget, memory_cycles: u16) -> (u16, u16) {
        match target.to_register8() {
            Some(_) => self.instruction_result(2, 8),
            None => self.instruction_result(2, memory_cycles),
        }
    }

    fn execute_inc_dec_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        enum IncDecOp {
            Inc,
            Dec,
        }

        let (target, op) = match instruction {
            Instruction::INC(target) => (target, IncDecOp::Inc),
            Instruction::DEC(target) => (target, IncDecOp::Dec),
            _ => unreachable!("execute_inc_dec_instruction called with non INC/DEC instruction"),
        };

        match target {
            IncDecTarget::Reg8(reg) => {
                let value = self.registers.read_8bit(reg);
                let new_value = match op {
                    IncDecOp::Inc => self.inc_8bit(value),
                    IncDecOp::Dec => self.dec_8bit(value),
                };
                self.registers.write_8bit(reg, new_value);
                self.instruction_result(1, 4)
            }
            IncDecTarget::Reg16(reg) => {
                match op {
                    IncDecOp::Inc => self.inc_16bit(reg),
                    IncDecOp::Dec => self.dec_16bit(reg),
                }
                self.instruction_result(1, 8)
            }
            IncDecTarget::HLI => {
                let address = self.registers.get_hl();
                let value = self.bus.read_byte(address);
                let new_value = match op {
                    IncDecOp::Inc => self.inc_8bit(value),
                    IncDecOp::Dec => self.dec_8bit(value),
                };
                self.bus.write_byte(address, new_value);
                self.instruction_result(1, 12)
            }
        }
    }

    fn execute_control_flow_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::JP(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.jump(should);
                let cycles = Self::branch_cycles(should, 16, 12);
                Self::absolute_result(next_pc, cycles)
            }
            Instruction::JR(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.jump_relative(should);
                let cycles = Self::branch_cycles(should, 12, 8);
                Self::absolute_result(next_pc, cycles)
            }
            Instruction::CALL(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.call(should);
                let cycles = Self::branch_cycles(should, 24, 12);
                Self::absolute_result(next_pc, cycles)
            }
            Instruction::RET(test) => {
                let should = self.should_jump(&test);
                let next_pc = self.return_(should);
                let cycles = match test {
                    JumpTest::Always => 16,
                    _ => Self::branch_cycles(should, 20, 8),
                };
                Self::absolute_result(next_pc, cycles)
            }
            Instruction::RETI => {
                self.interrupts_enabled = true;
                Self::absolute_result(self.pop(), 16)
            }
            Instruction::RST(vec) => {
                let next_pc = self.registers.pc.wrapping_add(1);
                self.push(next_pc);
                Self::absolute_result(vec as u16, 16)
            }
            Instruction::JP_HL => Self::absolute_result(self.registers.get_hl(), 4),
            _ => unreachable!("execute_control_flow_instruction called with non-control-flow instruction"),
        }
    }

    fn execute_load_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = self.read_byte_source(source);
                    self.write_byte_target(target, source_value);
                    let cycles = self.get_load_byte_cycles(target, source);
                    let pc_increment = self.get_load_byte_pc_increment(target, source);
                    self.instruction_result(pc_increment, cycles)
                }
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
                        (LoadWordTarget::A16I, _) => (3, 20),
                        (_, LoadWordSource::D16) => (3, 12),
                        _ => (1, 8),
                    };
                    self.instruction_result(length, cycles)
                }
            },
            _ => unreachable!("execute_load_instruction called with non-LD instruction"),
        }
    }

    fn execute_stack_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::PUSH(target) => {
                let value = self.read_stack_target(target);
                self.push(value);
                self.instruction_result(1, 16)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                self.write_stack_target(target, result);
                self.instruction_result(1, 12)
            }
            _ => unreachable!("execute_stack_instruction called with non-stack instruction"),
        }
    }

    fn execute_misc_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::RLCA => {
                let (result, carry) = fh::rotate_left_circular(self.registers.a);
                self.registers.a = result;
                self.registers.f.apply_rotate_accumulator(carry);
                self.instruction_result(1, 4)
            }
            Instruction::RRCA => {
                let (result, carry) = fh::rotate_right_circular(self.registers.a);
                self.registers.a = result;
                self.registers.f.apply_rotate_accumulator(carry);
                self.instruction_result(1, 4)
            }
            Instruction::RLA => {
                let (result, carry) =
                    fh::rotate_left_through_carry(self.registers.a, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f.apply_rotate_accumulator(carry);
                self.instruction_result(1, 4)
            }
            Instruction::RRA => {
                let (result, carry) =
                    fh::rotate_right_through_carry(self.registers.a, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f.apply_rotate_accumulator(carry);
                self.instruction_result(1, 4)
            }
            Instruction::DAA => {
                let mut a = self.registers.a;
                let mut adjust: u8 = 0;
                let mut carry = self.registers.f.carry;

                if !self.registers.f.subtract {
                    if self.registers.f.half_carry || (a & 0x0F) > 9 {
                        adjust |= 0x06;
                    }
                    if self.registers.f.carry || a > 0x99 {
                        adjust |= 0x60;
                        carry = true;
                    }
                    a = a.wrapping_add(adjust);
                } else {
                    if self.registers.f.half_carry {
                        adjust |= 0x06;
                    }
                    if self.registers.f.carry {
                        adjust |= 0x60;
                        carry = true;
                    }
                    a = a.wrapping_sub(adjust);
                }

                self.registers.a = a;
                self.registers.f.zero = a == 0;
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry;
                self.instruction_result(1, 4)
            }
            Instruction::CPL => {
                self.registers.a = !self.registers.a;
                self.registers.f.apply_nh(true, true);
                self.instruction_result(1, 4)
            }
            Instruction::SCF => {
                self.registers.f.apply_nh(false, false);
                self.registers.f.carry = true;
                self.instruction_result(1, 4)
            }
            Instruction::CCF => {
                self.registers.f.apply_nh(false, false);
                self.registers.f.carry = !self.registers.f.carry;
                self.instruction_result(1, 4)
            }
            Instruction::ADDHL(reg) => {
                let hl = self.registers.get_hl();
                let value = match reg {
                    Register16::BC => self.registers.get_bc(),
                    Register16::DE => self.registers.get_de(),
                    Register16::HL => self.registers.get_hl(),
                    Register16::SP => self.registers.sp,
                };

                let (result, carry) = hl.overflowing_add(value);
                let half_carry = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;

                self.registers.set_hl(result);
                self.registers.f.subtract = false;
                self.registers.f.half_carry = half_carry;
                self.registers.f.carry = carry;
                self.instruction_result(1, 8)
            }
            Instruction::ADDSP => {
                let offset_signed = self.read_next_byte() as i8;
                let sp = self.registers.sp;
                let (result, half_carry, carry) = Self::compute_sp_plus_offset(sp, offset_signed);

                self.registers.f.apply_sp_offset(half_carry, carry);
                self.registers.sp = result;
                self.instruction_result(2, 16)
            }
            Instruction::LDHLSP => {
                let offset_signed = self.read_next_byte() as i8;
                let sp = self.registers.sp;
                let (result, half_carry, carry) = Self::compute_sp_plus_offset(sp, offset_signed);

                self.registers.f.apply_sp_offset(half_carry, carry);
                self.registers.set_hl(result);
                self.instruction_result(2, 12)
            }
            _ => unreachable!("execute_misc_instruction called with unsupported instruction"),
        }
    }

    fn execute_prefixed_instruction(&mut self, instruction: Instruction) -> (u16, u16) {
        match instruction {
            Instruction::RLC(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::rotate_left_circular(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::RRC(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::rotate_right_circular(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::RL(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::rotate_left_through_carry(value, self.registers.f.carry);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::RR(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::rotate_right_through_carry(value, self.registers.f.carry);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::SLA(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::shift_left_arithmetic(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::SRA(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::shift_right_arithmetic(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::SWAP(target) => {
                let value = self.read_prefix_target(target);
                let result = fh::swap_nibbles(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, false);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::SRL(target) => {
                let value = self.read_prefix_target(target);
                let (result, carry) = fh::shift_right_logical(value);
                self.write_prefix_target(target, result);
                self.registers.f.apply_rotate_shift(result, carry);
                self.prefixed_result_for_target(&target, 16)
            }
            Instruction::BIT(bit, target) => self.execute_prefixed_bit(bit, target),
            Instruction::RES(bit, target) => self.execute_prefixed_res_set(bit, target, false),
            Instruction::SET(bit, target) => self.execute_prefixed_res_set(bit, target, true),
            _ => unreachable!("execute_prefixed_instruction called with non-prefixed instruction"),
        }
    }

    fn execute_prefixed_bit(&mut self, bit: u8, target: PrefixTarget) -> (u16, u16) {
        let value = self.read_prefix_target(target);
        let result = value & (1 << bit);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.prefixed_result_for_target(&target, 12)
    }

    fn execute_prefixed_res_set(
        &mut self,
        bit: u8,
        target: PrefixTarget,
        set: bool,
    ) -> (u16, u16) {
        let value = self.read_prefix_target(target);
        let result = if set { value | (1 << bit) } else { value & !(1 << bit) };
        self.write_prefix_target(target, result);
        self.prefixed_result_for_target(&target, 16)
    }

}
