//! CPU module implementing the Game Boy processor.
//!
//! This module contains the CPU struct and instruction execution logic,
//! managing registers, memory access, and the fetch-decode-execute cycle.

use crate::instructions::Instruction;
use crate::memory_bus::MemoryBus;
use crate::register::{self, Registers};

mod execute;
mod stack_interrupts;
mod control_flow;
mod load;
mod arithmetic;
mod fetch_trace;
mod prefix;

pub(crate) use arithmetic::ArithmeticOps;
pub(crate) use control_flow::ControlFlowOps;
pub(crate) use fetch_trace::FetchTraceOps;
pub(crate) use load::LoadOps;
pub(crate) use prefix::PrefixOps;
pub(crate) use stack_interrupts::StackInterruptOps;

pub struct CPU {
    pub registers: register::Registers,
    pub bus: MemoryBus,
    is_halted: bool,
    pub interrupts_enabled: bool,
    ei_pending: bool,
    halt_bug: bool,
    last_step_executed_opcode: bool,
    last_trace: Option<CpuTraceState>,
}

pub struct CpuTraceState {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub pcmem: [u8; 4],
}

impl CPU {
    /// Create a new CPU with the initial register state and a ROM loaded into the bus.
    pub fn new(rom_data: Vec<u8>) -> CPU {
        let bus = MemoryBus::new(rom_data);
        CPU {
            registers: Registers::new(),
            bus,
            is_halted: false,
            interrupts_enabled: false,
            ei_pending: false,
            halt_bug: false,
            last_step_executed_opcode: false,
            last_trace: None,
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
    pub fn step(&mut self) -> u16 {
        self.last_step_executed_opcode = false;
        self.last_trace = None;
        if let Some(cycles) = self.process_pre_instruction_state() {
            return cycles;
        }

        let pc = self.registers.pc;
        let pcmem = [
            self.bus.read_byte(pc),
            self.bus.read_byte(pc.wrapping_add(1)),
            self.bus.read_byte(pc.wrapping_add(2)),
            self.bus.read_byte(pc.wrapping_add(3)),
        ];
        self.last_trace = Some(CpuTraceState {
            a: self.registers.a,
            f: self.registers.f.to_byte(),
            b: self.registers.b,
            c: self.registers.c,
            d: self.registers.d,
            e: self.registers.e,
            h: self.registers.h,
            l: self.registers.l,
            sp: self.registers.sp,
            pc,
            pcmem,
        });

        let (prefixed, opcode_byte, instruction) = self.fetch_and_decode_instruction();
        self.trace_instruction(prefixed, opcode_byte, &instruction);

        self.apply_halt_bug_if_needed();
        let cycles = self.execute_and_advance_pc(instruction);
        self.last_step_executed_opcode = true;
        self.commit_ei_if_pending();

        cycles
    }

    fn process_pre_instruction_state(&mut self) -> Option<u16> {
        if self.bus.any_interrupt_pending() {
            self.wake_from_halt();
        }

        if let Some(cycles) = self.handle_interrupts() {
            return Some(cycles);
        }

        if self.is_halted {
            // CPU is halted and no interrupt to service, consume 4 T-cycles
            return Some(4);
        }

        None
    }


    fn apply_halt_bug_if_needed(&mut self) {
        // HALT bug: When set, decrement PC before execution so operand reads
        // happen at the wrong address (byte after HALT is read twice)
        if self.halt_bug {
            self.halt_bug = false;
            self.registers.pc = self.registers.pc.wrapping_sub(1);
        }
    }

    fn execute_and_advance_pc(&mut self, instruction: Instruction) -> u16 {
        // Execute the decoded instruction and advance PC
        let (next_pc, cycles) = self.execute(instruction);
        self.registers.pc = next_pc;
        cycles
    }

    fn commit_ei_if_pending(&mut self) {
        // Handle EI delay: IME is enabled after the instruction following EI completes
        if self.ei_pending {
            self.ei_pending = false;
            self.interrupts_enabled = true;
        }
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

    /// Returns whether the last executed step was an actual opcode (as opposed to a NOP during HALT).
    pub fn last_step_executed_opcode(&self) -> bool {
        self.last_step_executed_opcode
    }

    pub fn take_last_trace(&mut self) -> Option<CpuTraceState> {
        self.last_trace.take()
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{JumpTest};
    use crate::interrupts::{Interrupt, INTERRUPT_CYCLES};

    fn cpu_with_program(program: &[u8]) -> CPU {
        let mut rom = vec![0u8; 0x200];
        for (i, byte) in program.iter().enumerate() {
            rom[0x100 + i] = *byte;
        }
        CPU::new(rom)
    }

    #[test]
    fn halt_bug_reexecutes_next_opcode() {
        // HALT; INC B; NOP
        let mut cpu = cpu_with_program(&[0x76, 0x04, 0x00]);

        cpu.bus.interrupts.write_ie(Interrupt::Timer.bit_mask());
        cpu.bus.interrupts.request_interrupt(Interrupt::Timer);

        let cycles1 = cpu.step();
        assert_eq!(cycles1, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.b, 0x00);

        // First post-HALT instruction executes with HALT bug active (PC doesn't advance past opcode).
        let cycles2 = cpu.step();
        assert_eq!(cycles2, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.b, 0x01);

        // Same opcode executes again on the next step.
        let cycles3 = cpu.step();
        assert_eq!(cycles3, 4);
        assert_eq!(cpu.registers.pc, 0x0102);
        assert_eq!(cpu.registers.b, 0x02);
    }

    #[test]
    fn interrupt_service_priority_and_stack_push_order() {
        let mut cpu = cpu_with_program(&[0x00]);
        let initial_sp = cpu.registers.sp;

        cpu.interrupts_enabled = true;
        cpu.bus
            .interrupts
            .write_ie(Interrupt::VBlank.bit_mask() | Interrupt::Timer.bit_mask());
        cpu.bus.interrupts.request_interrupt(Interrupt::Timer);
        cpu.bus.interrupts.request_interrupt(Interrupt::VBlank);

        let cycles = cpu.step();
        assert_eq!(cycles, INTERRUPT_CYCLES);
        assert_eq!(cpu.registers.pc, Interrupt::VBlank.handler_address());
        assert!(!cpu.interrupts_enabled);

        assert_eq!(cpu.registers.sp, initial_sp.wrapping_sub(2));
        assert_eq!(cpu.bus.read_byte(cpu.registers.sp), 0x00);
        assert_eq!(cpu.bus.read_byte(cpu.registers.sp.wrapping_add(1)), 0x01);

        assert_eq!(
            cpu.bus.interrupts.get_pending_interrupt(),
            Some(Interrupt::Timer)
        );
    }

    #[test]
    fn jr_conditional_timing_and_pc() {
        let mut cpu = cpu_with_program(&[0x00, 0x00, 0x00]);
        cpu.registers.pc = 0x0100;
        cpu.bus.memory[0x0101] = 0x05;

        cpu.registers.f.zero = false;
        let (next_pc_not_taken, cycles_not_taken) = cpu.execute(Instruction::JR(JumpTest::Zero));
        assert_eq!(next_pc_not_taken, 0x0102);
        assert_eq!(cycles_not_taken, 8);

        cpu.registers.f.zero = true;
        let (next_pc_taken, cycles_taken) = cpu.execute(Instruction::JR(JumpTest::Zero));
        assert_eq!(next_pc_taken, 0x0107);
        assert_eq!(cycles_taken, 12);
    }

    #[test]
    fn addsp_and_ldhlsp_flags_match_expected() {
        let mut cpu = cpu_with_program(&[0x00, 0x00, 0x00]);

        cpu.registers.pc = 0x0100;
        cpu.registers.sp = 0x000F;
        cpu.bus.memory[0x0101] = 0x01;
        let (next_pc_addsp, cycles_addsp) = cpu.execute(Instruction::ADDSP);
        assert_eq!(next_pc_addsp, 0x0102);
        assert_eq!(cycles_addsp, 16);
        assert_eq!(cpu.registers.sp, 0x0010);
        assert!(!cpu.registers.f.zero);
        assert!(!cpu.registers.f.subtract);
        assert!(cpu.registers.f.half_carry);
        assert!(!cpu.registers.f.carry);

        cpu.registers.pc = 0x0100;
        cpu.registers.sp = 0x0000;
        cpu.bus.memory[0x0101] = 0xFF; // -1
        let (next_pc_ldhlsp, cycles_ldhlsp) = cpu.execute(Instruction::LDHLSP);
        assert_eq!(next_pc_ldhlsp, 0x0102);
        assert_eq!(cycles_ldhlsp, 12);
        assert_eq!(cpu.registers.get_hl(), 0xFFFF);
        assert!(!cpu.registers.f.zero);
        assert!(!cpu.registers.f.subtract);
        assert!(!cpu.registers.f.half_carry);
        assert!(!cpu.registers.f.carry);
    }

    #[test]
    fn stack_push_pop_order_round_trip() {
        let mut cpu = cpu_with_program(&[0x00]);
        cpu.registers.sp = 0xFFFE;

        cpu.push(0x1234);
        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(cpu.bus.read_byte(0xFFFD), 0x12);
        assert_eq!(cpu.bus.read_byte(0xFFFC), 0x34);

        let value = cpu.pop();
        assert_eq!(value, 0x1234);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }
}
