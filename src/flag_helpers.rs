//! Helper functions for Game Boy CPU flag calculations.
//!
//! This module centralises half-carry/carry and related helpers used by
//! arithmetic operations (ADD/ADC/SUB/SBC, INC/DEC, ADD SP, r8, LD HL, SP+r8).
//!
//! The Game Boy's rules for half-carry/half-borrow and SP+r8 flag calculation
//! are subtle. These helpers encode the canonical behaviours:
//! - half-carry for 8-bit add/adc is carry from bit 3 -> 4
//! - half-borrow for 8-bit sub/sbc is borrow from bit 4 -> 3
//! - INC/DEC test H differently (based only on the operand before change)
//! - ADD SP, r8 and LD HL, SP+r8 compute H/C from low nibble/low byte where the
//!   signed immediate is treated as an unsigned byte for flag calculations.
//!
#![allow(dead_code)]

/// Half-carry from bit 3 to bit 4 for an 8-bit add (A + B).
/// Returns true if a half-carry occurred.
pub fn half_carry_add(a: u8, b: u8) -> bool {
    (((a & 0x0F) as u16) + ((b & 0x0F) as u16)) > 0x0F
}

/// Half-carry for add with carry-in: A + B + carry_in.
pub fn half_carry_add_with_carry(a: u8, b: u8, carry_in: bool) -> bool {
    let c = if carry_in { 1u16 } else { 0u16 };
    (((a & 0x0F) as u16) + ((b & 0x0F) as u16) + c) > 0x0F
}

/// Carry for 8-bit add (A + B).
pub fn carry_add(a: u8, b: u8) -> bool {
    ((a as u16) + (b as u16)) > 0xFF
}

/// Carry for add with carry-in (A + B + carry_in).
pub fn carry_add_with_carry(a: u8, b: u8, carry_in: bool) -> bool {
    let c = if carry_in { 1u16 } else { 0u16 };
    ((a as u16) + (b as u16) + c) > 0xFF
}

/// Half-borrow for 8-bit subtraction (A - B).
/// True when there's a borrow from bit 4 to bit 3.
pub fn half_borrow_sub(a: u8, b: u8) -> bool {
    ((a & 0x0F) as u16) < ((b & 0x0F) as u16)
}

/// Half-borrow for subtraction with carry-in (A - B - carry_in).
///
/// Important: this uses u16 arithmetic to avoid wrapping bugs when
/// (b_low_nibble + carry_in) would overflow u8 (0x0F + 1 -> 0x00).
pub fn half_borrow_sub_with_carry(a: u8, b: u8, carry_in: bool) -> bool {
    let c = if carry_in { 1u16 } else { 0u16 };
    ((a & 0x0F) as u16) < (((b & 0x0F) as u16) + c)
}

/// Borrow / carry for 8-bit subtraction (A - B).
pub fn borrow_sub(a: u8, b: u8) -> bool {
    (a as u16) < (b as u16)
}

/// Borrow / carry for subtraction with carry-in (A - B - carry_in).
pub fn borrow_sub_with_carry(a: u8, b: u8, carry_in: bool) -> bool {
    let c = if carry_in { 1u16 } else { 0u16 };
    (a as u16) < ((b as u16) + c)
}

/// INC (8-bit) half-carry check: set H if low nibble wraps from 0x0F -> 0x00.
/// INC does NOT affect the carry flag.
pub fn half_carry_inc(value: u8) -> bool {
    (value & 0x0F) == 0x0F
}

/// DEC (8-bit) half-borrow check: set H when low nibble borrows (value low nibble == 0x00).
/// DEC does NOT affect the carry flag.
pub fn half_borrow_dec(value: u8) -> bool {
    (value & 0x0F) == 0x00
}

/// Compute the result of `SP + r8` where `r8` is a signed 8-bit immediate.
/// The arithmetic wraps like on hardware.
///
/// This function returns the resulting SP value.
pub fn add_sp_signed(sp: u16, offset: i8) -> u16 {
    sp.wrapping_add(offset as i16 as u16)
}

/// For ADD SP,r8 and LD HL,SP+r8 the flags H and C are defined by adding
/// the low nibble / low byte of SP to the immediate interpreted as an
/// unsigned byte. The signedness only affects the final result; flags
/// treat the immediate as u8 for the check.
///
/// half_carry_add_sp: H flag (carry from bit 3)
pub fn half_carry_add_sp(sp: u16, offset: i8) -> bool {
    let off_u8 = offset as u8;
    (((sp & 0x0F) as u16) + ((off_u8 as u16) & 0x0F)) > 0x0F
}

/// carry_add_sp: C flag (carry from bit 7, i.e. low byte overflow)
pub fn carry_add_sp(sp: u16, offset: i8) -> bool {
    let off_u8 = offset as u8;
    (((sp & 0xFF) as u16) + (off_u8 as u16)) > 0xFF
}

/// Mask the F register to ensure lower 4 bits are zero (hardware invariant).
/// This is a tiny helper if you need to write raw bytes into F.
pub fn mask_f(value: u8) -> u8 {
    value & 0xF0
}

/// Rotate left circular: bit 7 becomes bit 0 and carry-out.
pub fn rotate_left_circular(value: u8) -> (u8, bool) {
    let carry = (value & 0x80) != 0;
    (value.rotate_left(1), carry)
}

/// Rotate right circular: bit 0 becomes bit 7 and carry-out.
pub fn rotate_right_circular(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    (value.rotate_right(1), carry)
}

/// Rotate left through carry: old carry enters bit 0, bit 7 becomes carry-out.
pub fn rotate_left_through_carry(value: u8, carry_in: bool) -> (u8, bool) {
    let carry_out = (value & 0x80) != 0;
    let result = (value << 1) | u8::from(carry_in);
    (result, carry_out)
}

/// Rotate right through carry: old carry enters bit 7, bit 0 becomes carry-out.
pub fn rotate_right_through_carry(value: u8, carry_in: bool) -> (u8, bool) {
    let carry_out = (value & 0x01) != 0;
    let result = (value >> 1) | (u8::from(carry_in) << 7);
    (result, carry_out)
}

/// Shift left arithmetic: bit 0 becomes 0, bit 7 becomes carry-out.
pub fn shift_left_arithmetic(value: u8) -> (u8, bool) {
    let carry = (value & 0x80) != 0;
    (value << 1, carry)
}

/// Shift right arithmetic: keeps bit 7, bit 0 becomes carry-out.
pub fn shift_right_arithmetic(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    ((value >> 1) | (value & 0x80), carry)
}

/// Shift right logical: bit 7 becomes 0, bit 0 becomes carry-out.
pub fn shift_right_logical(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    (value >> 1, carry)
}

/// Swap upper and lower nibbles.
pub fn swap_nibbles(value: u8) -> u8 {
    value.rotate_left(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_carry_add() {
        assert!(half_carry_add(0x0F, 0x01));
        assert!(!half_carry_add(0x07, 0x01));
    }

    #[test]
    fn test_half_carry_add_with_carry() {
        assert!(half_carry_add_with_carry(0x0F, 0x00, true));
        assert!(!half_carry_add_with_carry(0x08, 0x01, false));
    }

    #[test]
    fn test_carry_add() {
        assert!(carry_add(0xFF, 0x01));
        assert!(!carry_add(0x7F, 0x01));
    }

    #[test]
    fn test_half_borrow_sub_edge_case() {
        // The SBC trap: b_low_nibble = 0x0F, carry_in = 1 => (b_low + carry) would wrap in u8
        // Our helper must still detect half-borrow correctly.
        let a = 0x00u8;
        let b = 0x0Fu8;
        assert!(half_borrow_sub_with_carry(a, b, true)); // true because 0x00 < (0x0F + 1)
    }

    #[test]
    fn test_borrow_sub_with_carry() {
        assert!(borrow_sub_with_carry(0x01, 0x02, false));
        assert!(borrow_sub_with_carry(0x00, 0xFF, true)); // 0 < 0xFF + 1
    }

    #[test]
    fn test_inc_dec_half_helpers() {
        assert!(half_carry_inc(0x0F));
        assert!(!half_carry_inc(0x0E));
        assert!(half_borrow_dec(0x10)); // low nibble zero => DEC will borrow from bit 4
    }

    #[test]
    fn test_add_sp_signed_and_flags() {
        // Positive offset
        let sp = 0xFFF8u16;
        let off: i8 = 8;
        let result = add_sp_signed(sp, off);
        assert_eq!(result, 0x0000u16); // 0xFFF8 + 8 == 0x10000 -> wraps to 0x0000
        assert!(carry_add_sp(sp, off)); // low byte overflow
        assert!(half_carry_add_sp(0x000F, 0x01)); // low nibble half-carry

        // Negative offset (e.g., -1)
        let sp2 = 0x0000u16;
        let off2: i8 = -1;
        let res2 = add_sp_signed(sp2, off2);
        assert_eq!(res2, 0xFFFFu16);
        // Flags are computed with offset treated as u8 (0xFF)
        assert!(!half_carry_add_sp(sp2, off2)); // (0x0 + 0xF) <= 0x0F => no half carry
        assert!(!carry_add_sp(sp2, off2)); // (0x00 + 0xFF) <= 0xFF => no full carry
    }

    #[test]
    fn test_mask_f() {
        assert_eq!(mask_f(0xFF), 0xF0);
        assert_eq!(mask_f(0x0F), 0x00);
    }

    #[test]
    fn test_rotate_and_shift_helpers() {
        assert_eq!(rotate_left_circular(0x81), (0x03, true));
        assert_eq!(rotate_right_circular(0x01), (0x80, true));

        assert_eq!(rotate_left_through_carry(0x80, false), (0x00, true));
        assert_eq!(rotate_left_through_carry(0x00, true), (0x01, false));

        assert_eq!(rotate_right_through_carry(0x01, false), (0x00, true));
        assert_eq!(rotate_right_through_carry(0x00, true), (0x80, false));

        assert_eq!(shift_left_arithmetic(0x81), (0x02, true));
        assert_eq!(shift_right_arithmetic(0x81), (0xC0, true));
        assert_eq!(shift_right_logical(0x81), (0x40, true));

        assert_eq!(swap_nibbles(0xF0), 0x0F);
        assert_eq!(swap_nibbles(0xAB), 0xBA);
    }
}
