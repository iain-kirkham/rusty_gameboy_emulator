/// The F register contains 4 flags in the upper nibble (bits 7-4).
/// Bits 3-0 are always zero on real hardware.
///
/// - Bit 7: Zero (Z) - Set when the result of an operation is zero
/// - Bit 6: Subtract (N) - Set if the last operation was a subtraction
/// - Bit 5: Half Carry (H) - Set if carry occurred from bit 3 to bit 4
/// - Bit 4: Carry (C) - Set if carry occurred from bit 7 or borrow occurred
#[derive(Debug, Clone, Copy, Default)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Register16 {
    BC,
    DE,
    HL,
    SP,
}

/// Convert flags struct to the F register byte.
///
/// Important: on real Game Boy hardware, bits 0-3 of the F register are
/// always zero. We only store the four flag bits in bits 7-4; lower-nibble
/// bits are intentionally never set by this function.
impl FlagsRegister {
    pub fn to_byte(&self) -> u8 {
        (if self.zero { 0x80 } else { 0 })
            | (if self.subtract { 0x40 } else { 0 })
            | (if self.half_carry { 0x20 } else { 0 })
            | (if self.carry { 0x10 } else { 0 })
    }

    /// Build a FlagsRegister from a raw F-register byte.
    ///
    /// Note: incoming bytes may have garbage in the lower nibble (bits 0-3).
    /// We mask with `0xF0` to ensure bits 0-3 are cleared, matching real hardware.
    /// This guarantees the lower nibble invariant regardless of caller-provided bytes. (should program code illegaly change them)
    pub fn from_byte(byte: u8) -> FlagsRegister {
        // On real Game Boy hardware, bits 0-3 of the F register are always 0 (use bit mask to ensure they are always 0)
        let byte = byte & 0xF0;
        FlagsRegister {
            zero: byte & 0x80 != 0,
            subtract: byte & 0x40 != 0,
            half_carry: byte & 0x20 != 0,
            carry: byte & 0x10 != 0,
        }
    }
}

pub struct Registers {
    pub a: u8,            // Accumulator register
    pub b: u8,            // General purpose register
    pub c: u8,            // General purpose register
    pub d: u8,            // General purpose register
    pub e: u8,            // General purpose register
    pub f: FlagsRegister, // FlagsRegister for F
    pub h: u8,            // High byte of the HL register pair
    pub l: u8,            // Low byte of the HL register pair
    pub sp: u16,          // Stack pointer
    pub pc: u16,          // Program counter
}

impl Registers {
    /// Creates a new Registers struct with post-boot ROM values.
    ///
    /// These are the register values immediately after the boot ROM finishes execution
    /// on a real DMG-01 Game Boy. The boot ROM:
    /// - Scrolls the Nintendo logo
    /// - Plays the startup sound
    /// - Validates the cartridge header
    /// - Initializes hardware registers
    ///
    /// PC starts at 0x0100 (first instruction of the cartridge ROM)
    /// SP starts at 0xFFFE (top of High RAM)
    pub fn new() -> Registers {
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: FlagsRegister::from_byte(0xC0),
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x100,
        }
    }

    // Combined 16-bit register pairs getters and setters
    /// Gets the 16-bit AF register pair.
    /// The A register forms the high byte, and the F (Flags) register forms the low byte.
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.to_byte() as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from_byte((value & 0x00FF) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    /// Read an 8-bit register value
    pub fn read_8bit(&self, reg: Register8) -> u8 {
        match reg {
            Register8::A => self.a,
            Register8::B => self.b,
            Register8::C => self.c,
            Register8::D => self.d,
            Register8::E => self.e,
            Register8::H => self.h,
            Register8::L => self.l,
        }
    }

    /// Write to an 8-bit register
    pub fn write_8bit(&mut self, reg: Register8, value: u8) {
        match reg {
            Register8::A => self.a = value,
            Register8::B => self.b = value,
            Register8::C => self.c = value,
            Register8::D => self.d = value,
            Register8::E => self.e = value,
            Register8::H => self.h = value,
            Register8::L => self.l = value,
        }
    }

    /// Read a 16-bit register value
    pub fn read_16bit(&self, reg: Register16) -> u16 {
        match reg {
            Register16::BC => self.get_bc(),
            Register16::DE => self.get_de(),
            Register16::HL => self.get_hl(),
            Register16::SP => self.sp,
        }
    }

    /// Write to a 16-bit register
    pub fn write_16bit(&mut self, reg: Register16, value: u16) {
        match reg {
            Register16::BC => self.set_bc(value),
            Register16::DE => self.set_de(value),
            Register16::HL => self.set_hl(value),
            Register16::SP => self.sp = value,
        }
    }
}
