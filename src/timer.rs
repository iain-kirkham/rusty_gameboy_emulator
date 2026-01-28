//! Game Boy timer module
//! Implements DIV, TIMA, TMA, and TAC with edge-detection and overflow delay.
//! Reference: <https://gbdev.io/pandocs/Timer_and_Divider_Registers.html>

pub const DIV_REGISTER: u16 = 0xFF04;
pub const TIMA_REGISTER: u16 = 0xFF05;
pub const TMA_REGISTER: u16 = 0xFF06;
pub const TAC_REGISTER: u16 = 0xFF07;

const TAC_FREQUENCY_MASK: u8 = 0b0011;
const TAC_ENABLE_MASK: u8 = 0b0100;
const TAC_READONLY_MASK: u8 = 0b1111_1000;

const TIMA_FREQ_4096HZ_BIT: u8 = 9;
const TIMA_FREQ_262144HZ_BIT: u8 = 3;
const TIMA_FREQ_65536HZ_BIT: u8 = 5;
const TIMA_FREQ_16384HZ_BIT: u8 = 7;

const TIMA_OVERFLOW_RELOAD_DELAY: u8 = 4;
const DIV_RESET_VALUE: u16 = 0x0000;

pub struct Timer {
    div: u16,                   // internal 16-bit divider
    tima: u8,                   // timer counter (0xFF05)
    tma: u8,                    // timer modulo (0xFF06)
    tac: u8,                    // timer control (0xFF07)
    prev_timer_bit: bool,       // previous selected DIV bit state
    overflow_delay: Option<u8>, // pending reload delay after overflow
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: DIV_RESET_VALUE,
            tima: 0,
            tma: 0,
            tac: 0,
            prev_timer_bit: false,
            overflow_delay: None,
        }
    }

    // Advance one T-cycle. Returns true if a timer interrupt should fire.
    pub fn tick(&mut self) -> bool {
        self.div = self.div.wrapping_add(1);
        let interrupt = self.update_overflow_delay();

        // Recompute selected bit and handle falling edge in one place
        let current_bit = self.calculate_timer_bit();
        self.handle_falling_edge_and_update_prev(current_bit);

        interrupt
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            DIV_REGISTER => self.read_div(),
            TIMA_REGISTER => self.tima,
            TMA_REGISTER => self.tma,
            TAC_REGISTER => self.read_tac(),
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            DIV_REGISTER => self.write_div(),
            TIMA_REGISTER => self.write_tima(value),
            TMA_REGISTER => self.write_tma(value),
            TAC_REGISTER => self.write_tac(value),
            _ => {}
        }
    }

    fn get_timer_bit_position(&self) -> u8 {
        match self.tac & TAC_FREQUENCY_MASK {
            0b00 => TIMA_FREQ_4096HZ_BIT,
            0b01 => TIMA_FREQ_262144HZ_BIT,
            0b10 => TIMA_FREQ_65536HZ_BIT,
            0b11 => TIMA_FREQ_16384HZ_BIT,
            _ => unreachable!(),
        }
    }

    fn is_timer_enabled(&self) -> bool {
        (self.tac & TAC_ENABLE_MASK) != 0
    }

    // Returns the current state of the selected DIV bit (false if timer disabled).
    fn calculate_timer_bit(&self) -> bool {
        if !self.is_timer_enabled() {
            return false;
        }
        let bit_pos = self.get_timer_bit_position();
        ((self.div >> bit_pos) & 1) != 0
    }

    // Handle pending reload; returns true when interrupt should be requested.
    fn update_overflow_delay(&mut self) -> bool {
        if let Some(delay) = self.overflow_delay {
            if delay == 1 {
                self.tima = self.tma;
                self.overflow_delay = None;
                return true;
            } else {
                self.overflow_delay = Some(delay - 1);
            }
        }
        false
    }

    // detect a falling edge, increment TIMA if needed,
    // then update prev_timer_bit.
    fn handle_falling_edge_and_update_prev(&mut self, new_bit: bool) {
        if self.prev_timer_bit && !new_bit {
            self.increment_tima();
        }
        self.prev_timer_bit = new_bit;
    }

    fn increment_tima(&mut self) {
        let (new_val, overflowed) = self.tima.overflowing_add(1);
        if overflowed {
            self.tima = 0x00;
            self.overflow_delay = Some(TIMA_OVERFLOW_RELOAD_DELAY);
        } else {
            self.tima = new_val;
        }
    }

    // Register helpers
    fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    fn read_tac(&self) -> u8 {
        self.tac | TAC_READONLY_MASK
    }

    // Reset divider; recalc selected bit and handle glitch edge.
    fn write_div(&mut self) {
        self.div = DIV_RESET_VALUE;
        let current_bit = self.calculate_timer_bit();
        self.handle_falling_edge_and_update_prev(current_bit);
    }

    fn write_tima(&mut self, value: u8) {
        self.tima = value;
        if self.overflow_delay.is_some() {
            self.overflow_delay = None;
        }
    }

    fn write_tma(&mut self, value: u8) {
        self.tma = value;
    }

    // Update TAC (only bits 0-2 writable), then recompute selected bit and handle glitch edge.
    fn write_tac(&mut self, value: u8) {
        self.tac = value & 0x07;
        let current_bit = self.calculate_timer_bit();
        self.handle_falling_edge_and_update_prev(current_bit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_increments_every_cycle() {
        let mut timer = Timer::new();
        for _ in 0..256 {
            timer.tick();
        }
        assert_eq!(timer.read(DIV_REGISTER), 1);
    }

    #[test]
    fn test_div_reset_on_write() {
        let mut timer = Timer::new();
        for _ in 0..1000 {
            timer.tick();
        }
        timer.write(DIV_REGISTER, 0);
        assert_eq!(timer.read(DIV_REGISTER), 0);
    }

    #[test]
    fn test_tima_increments_on_falling_edge() {
        let mut timer = Timer::new();
        timer.write(TAC_REGISTER, 0b101); // enable, fastest
        timer.div = 7;
        timer.prev_timer_bit = false;
        for _ in 0..20 {
            timer.tick();
        }
        assert!(timer.read(TIMA_REGISTER) > 0);
    }

    #[test]
    fn test_overflow_triggers_interrupt_after_delay() {
        let mut timer = Timer::new();
        timer.write(TAC_REGISTER, 0b101);
        timer.write(TMA_REGISTER, 0x42);
        timer.write(TIMA_REGISTER, 0xFF);
        timer.div = 7;
        timer.prev_timer_bit = false;

        let mut fired = false;
        for _ in 0..30 {
            if timer.tick() {
                fired = true;
                break;
            }
        }

        assert!(fired);
        assert_eq!(timer.read(TIMA_REGISTER), 0x42);
    }

    #[test]
    fn test_writing_tima_cancels_reload() {
        let mut timer = Timer::new();
        timer.write(TAC_REGISTER, 0b101);
        timer.write(TMA_REGISTER, 0x50);
        timer.write(TIMA_REGISTER, 0xFF);
        timer.div = 7;
        timer.prev_timer_bit = false;

        for _ in 0..10 {
            timer.tick();
        }

        assert_eq!(timer.read(TIMA_REGISTER), 0);

        timer.write(TIMA_REGISTER, 0x25);

        for _ in 0..10 {
            timer.tick();
        }

        assert_eq!(timer.read(TIMA_REGISTER), 0x25);
    }
}
