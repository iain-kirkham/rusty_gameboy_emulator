//! Interrupt controller for the Game Boy.
//!
//! The Game Boy has 5 interrupts with the following priority (highest to lowest):
//! 1. V-Blank  (bit 0) - Handler at 0x0040
//! 2. LCD STAT (bit 1) - Handler at 0x0048
//! 3. Timer    (bit 2) - Handler at 0x0050
//! 4. Serial   (bit 3) - Handler at 0x0058
//! 5. Joypad   (bit 4) - Handler at 0x0060
//!
//! Interrupt Flow:
//! 1. Hardware sets the corresponding bit in IF (Interrupt Flag, 0xFF0F)
//! 2. During CPU step, if IME is enabled and (IE & IF) != 0:
//!    - IME is disabled
//!    - The highest priority pending interrupt's IF bit is cleared
//!    - Current PC is pushed onto the stack
//!    - PC is set to the interrupt handler address
//!    - This process takes 5 M-cycles (20 T-cycles)
//!
//! Note: If CPU is HALTed, it wakes when (IE & IF) != 0, even if IME is disabled.
//! This is known as the "HALT bug" when IME is disabled.
//!
//! Reference: [Pan Docs — Interrupts](https://gbdev.io/pandocs/Interrupts.html)

/// The 5 Game Boy interrupts in priority order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] //Represent as u8 for easy bit manipulation
pub enum Interrupt {
    /// V-Blank interrupt - triggered at the start of V-Blank period (LY=144)
    VBlank = 0,
    /// LCD STAT interrupt - triggered by various LCD status conditions
    LcdStat = 1,
    /// Timer interrupt - triggered when TIMA overflows
    Timer = 2,
    /// Serial interrupt - triggered when serial transfer completes
    Serial = 3,
    /// Joypad interrupt - triggered on button presses
    Joypad = 4,
}

impl Interrupt {
    /// Get the bit mask for this interrupt in IF/IE registers.
    pub const fn bit_mask(self) -> u8 {
        1 << (self as u8)
    }

    /// Get the interrupt handler address for this interrupt.
    pub const fn handler_address(self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::LcdStat => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }

    /// Return the interrupt from a bit index (0-4), if valid.
    #[allow(dead_code)]
    pub const fn from_bit(bit: u8) -> Option<Interrupt> {
        match bit {
            0 => Some(Interrupt::VBlank),
            1 => Some(Interrupt::LcdStat),
            2 => Some(Interrupt::Timer),
            3 => Some(Interrupt::Serial),
            4 => Some(Interrupt::Joypad),
            _ => None,
        }
    }

    /// Iterate over all interrupts in priority order (highest first).
    pub const ALL: [Interrupt; 5] = [
        Interrupt::VBlank,
        Interrupt::LcdStat,
        Interrupt::Timer,
        Interrupt::Serial,
        Interrupt::Joypad,
    ];
}

/// Interrupt controller managing IF and IE registers.
///
/// This struct holds the interrupt flag (IF) and interrupt enable (IE) registers
/// and provides methods for interrupt handling logic.
#[derive(Debug, Clone)]
pub struct InterruptController {
    /// Interrupt Flag register (IF) - 0xFF0F
    /// Bit 0: V-Blank request
    /// Bit 1: LCD STAT request
    /// Bit 2: Timer request
    /// Bit 3: Serial request
    /// Bit 4: Joypad request
    /// Bits 5-7: Always read as 1
    pub interrupt_flag: u8,

    /// Interrupt Enable register (IE) - 0xFFFF
    /// Bit 0: V-Blank enable
    /// Bit 1: LCD STAT enable
    /// Bit 2: Timer enable
    /// Bit 3: Serial enable
    /// Bit 4: Joypad enable
    pub interrupt_enable: u8,
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptController {
    /// Create a new interrupt controller with default state.
    ///
    /// IF starts at 0xE1 on DMG after boot ROM, but we initialize to 0
    /// since the emulator skips the boot ROM.
    pub fn new() -> Self {
        InterruptController {
            interrupt_flag: 0xE0,
            interrupt_enable: 0x00,
        }
    }

    /// Read the IF register (0xFF0F).
    /// Bits 5-7 are unused and always read as 1.
    pub fn read_if(&self) -> u8 {
        self.interrupt_flag | 0xE0
    }

    /// Write to the IF register (0xFF0F).
    pub fn write_if(&mut self, value: u8) {
        // Only lower 5 bits are writable
        self.interrupt_flag = (value & 0x1F) | 0xE0;
    }

    /// Read the IE register (0xFFFF).
    pub fn read_ie(&self) -> u8 {
        self.interrupt_enable
    }

    /// Write to the IE register (0xFFFF).
    pub fn write_ie(&mut self, value: u8) {
        self.interrupt_enable = value;
    }

    /// Request an interrupt by setting its bit in IF.
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flag |= interrupt.bit_mask();
    }

    /// Clear an interrupt request by clearing its bit in IF.
    pub fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flag &= !interrupt.bit_mask();
    }

    /// Check if any interrupt is pending (IE & IF != 0).
    /// This is used to wake the CPU from HALT.
    pub fn any_interrupt_pending(&self) -> bool {
        (self.interrupt_flag & self.interrupt_enable & 0x1F) != 0
    }

    /// Get the highest priority pending interrupt, if any.
    /// Returns the interrupt and its handler address.
    pub fn get_pending_interrupt(&self) -> Option<Interrupt> {
        let pending = self.interrupt_flag & self.interrupt_enable & 0x1F;
        if pending == 0 {
            return None;
        }

        // Check interrupts in priority order (lowest bit = highest priority)
        for interrupt in Interrupt::ALL {
            if (pending & interrupt.bit_mask()) != 0 {
                return Some(interrupt);
            }
        }

        None
    }

    /// Service an interrupt: clear its IF bit and return the handler address.
    ///
    /// This should be called after pushing PC and disabling IME.
    /// Returns the handler address to jump to.
    pub fn service_interrupt(&mut self, interrupt: Interrupt) -> u16 {
        self.acknowledge_interrupt(interrupt);
        interrupt.handler_address()
    }
}

/// Number of T-cycles consumed when servicing an interrupt.
/// The interrupt dispatch takes 5 M-cycles = 20 T-cycles:
/// - 2 M-cycles: Internal delay (and IME = 0)
/// - 2 M-cycles: Push PC onto stack
/// - 1 M-cycle: Set PC to handler address
pub const INTERRUPT_CYCLES: u16 = 20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_bit_masks() {
        assert_eq!(Interrupt::VBlank.bit_mask(), 0b0000_0001);
        assert_eq!(Interrupt::LcdStat.bit_mask(), 0b0000_0010);
        assert_eq!(Interrupt::Timer.bit_mask(), 0b0000_0100);
        assert_eq!(Interrupt::Serial.bit_mask(), 0b0000_1000);
        assert_eq!(Interrupt::Joypad.bit_mask(), 0b0001_0000);
    }

    #[test]
    fn test_interrupt_handler_addresses() {
        assert_eq!(Interrupt::VBlank.handler_address(), 0x0040);
        assert_eq!(Interrupt::LcdStat.handler_address(), 0x0048);
        assert_eq!(Interrupt::Timer.handler_address(), 0x0050);
        assert_eq!(Interrupt::Serial.handler_address(), 0x0058);
        assert_eq!(Interrupt::Joypad.handler_address(), 0x0060);
    }

    #[test]
    fn test_if_register_upper_bits() {
        let ic = InterruptController::new();
        // Upper 3 bits should always be 1
        assert_eq!(ic.read_if() & 0xE0, 0xE0);
    }

    #[test]
    fn test_request_and_acknowledge() {
        let mut ic = InterruptController::new();
        ic.request_interrupt(Interrupt::Timer);

        assert_eq!(
            ic.read_if() & Interrupt::Timer.bit_mask(),
            Interrupt::Timer.bit_mask()
        );

        ic.acknowledge_interrupt(Interrupt::Timer);
        assert_eq!(ic.read_if() & Interrupt::Timer.bit_mask(), 0);
    }

    #[test]
    fn test_pending_interrupt_priority() {
        let mut ic = InterruptController::new();

        // Enable all interrupts
        ic.write_ie(0x1F);

        // Request Timer and VBlank (VBlank has higher priority)
        ic.request_interrupt(Interrupt::Timer);
        ic.request_interrupt(Interrupt::VBlank);

        // Should return VBlank (higher priority)
        let pending = ic.get_pending_interrupt();
        assert_eq!(pending, Some(Interrupt::VBlank));
    }

    #[test]
    fn test_interrupt_not_pending_if_disabled() {
        let mut ic = InterruptController::new();
        let timer_mask = Interrupt::Timer.bit_mask();

        // Request timer but don't enable it
        ic.request_interrupt(Interrupt::Timer);

        // Check IF specifically for the timer bit
        assert_eq!(
            ic.read_if() & timer_mask,
            timer_mask,
            "IF bit should be set"
        );
        // Check that get_pending_interrupt returns None because IE is still 0
        assert!(
            ic.get_pending_interrupt().is_none(),
            "Should not be pending if disabled"
        );
        assert!(!ic.any_interrupt_pending(), "Any pending should be false");

        // Now enable it
        ic.write_ie(timer_mask);

        // Now it should be recognized
        assert_eq!(ic.get_pending_interrupt(), Some(Interrupt::Timer));
        assert!(ic.any_interrupt_pending());
    }

    #[test]
    fn test_service_interrupt() {
        let mut ic = InterruptController::new();
        let timer_mask = Interrupt::Timer.bit_mask();

        ic.write_ie(0x1F);
        ic.request_interrupt(Interrupt::Timer);

        // service_interrupt should clear the bit in IF and return the correct address
        let handler = ic.service_interrupt(Interrupt::Timer);

        assert_eq!(handler, 0x0050);
        assert_eq!(
            ic.read_if() & timer_mask,
            0,
            "IF bit should be cleared after servicing"
        );
    }
    #[test]
    fn test_from_bit() {
        assert_eq!(Interrupt::from_bit(0), Some(Interrupt::VBlank));
        assert_eq!(Interrupt::from_bit(1), Some(Interrupt::LcdStat));
        assert_eq!(Interrupt::from_bit(2), Some(Interrupt::Timer));
        assert_eq!(Interrupt::from_bit(3), Some(Interrupt::Serial));
        assert_eq!(Interrupt::from_bit(4), Some(Interrupt::Joypad));
        assert_eq!(Interrupt::from_bit(5), None);
    }
}
