use super::CPU;
use crate::instructions::StackTarget;
use crate::interrupts::INTERRUPT_CYCLES;

pub(crate) trait StackInterruptOps {
    fn wake_from_halt(&mut self);
    fn handle_interrupts(&mut self) -> Option<u16>;
    fn read_stack_target(&self, target: StackTarget) -> u16;
    fn write_stack_target(&mut self, target: StackTarget, value: u16);
    fn push(&mut self, value: u16);
    fn pop(&mut self) -> u16;
}

impl StackInterruptOps for CPU {
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
}
