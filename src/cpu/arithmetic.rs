use super::CPU;
use crate::flag_helpers as fh;
use crate::instructions::ArithmeticTarget;
use crate::register::Register16;

pub(crate) trait ArithmeticOps {
    fn get_arithmetic_target(&self, target: ArithmeticTarget) -> u8;
    fn add(&mut self, value: u8) -> u8;
    fn adc(&mut self, value: u8) -> u8;
    fn sub(&mut self, value: u8) -> u8;
    fn sbc(&mut self, value: u8) -> u8;
    fn and(&mut self, value: u8) -> u8;
    fn or(&mut self, value: u8) -> u8;
    fn xor(&mut self, value: u8) -> u8;
    fn cp(&mut self, value: u8);
    fn inc_8bit(&mut self, value: u8) -> u8;
    fn dec_8bit(&mut self, value: u8) -> u8;
    fn inc_16bit(&mut self, reg: Register16);
    fn dec_16bit(&mut self, reg: Register16);
}

impl ArithmeticOps for CPU {
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
        self.registers.f.apply_arithmetic(new_value, false, did_overflow, half_carry);
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

        self.registers.f.apply_arithmetic(new_value, false, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit subtraction: A -= value (sets all CPU flags).
    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        let half_carry = fh::half_borrow_sub(self.registers.a, value);
        self.registers.f.apply_arithmetic(new_value, true, did_overflow, half_carry);
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

        self.registers.f.apply_arithmetic(new_value, true, did_overflow, half_carry);
        new_value
    }

    /// Perform 8-bit bitwise AND: A &= value (sets Z and H flags, clears N and C).
    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.registers.f.apply_logic(new_value, true);
        new_value
    }

    /// Perform 8-bit bitwise OR: A |= value (sets Z flag, clears N, H, and C).
    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.registers.f.apply_logic(new_value, false);
        new_value
    }

    /// Perform 8-bit bitwise XOR: A ^= value (sets Z flag, clears N, H, and C).
    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.registers.f.apply_logic(new_value, false);
        new_value
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
}
