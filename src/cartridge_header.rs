//! Basic Game Boy cartridge header parsing utilities.
//!
//! Reference: https://gbdev.io/pandocs/The_Cartridge_Header.html

use core::fmt;

const HEADER_START: usize = 0x0100;
const HEADER_END: usize = 0x014F;

// Relevant fields we currently care about:
const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
const ROM_SIZE_ADDR: usize = 0x0148;
const RAM_SIZE_ADDR: usize = 0x0149;
const HEADER_CHECKSUM_ADDR: usize = 0x014D;

// Header checksum input range: 0x0134..=0x014C
const HEADER_CHECKSUM_RANGE_START: usize = 0x0134;
const HEADER_CHECKSUM_RANGE_END: usize = 0x014C;

/// High-level cartridge MBC family derived from cartridge type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbcKind {
    None,
    Mbc1,
    Mbc2,
    Mmm01,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    HuC1,
    HuC3,
    Unknown(u8),
}

impl fmt::Display for MbcKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MbcKind::None => write!(f, "ROM ONLY (No MBC)"),
            MbcKind::Mbc1 => write!(f, "MBC1"),
            MbcKind::Mbc2 => write!(f, "MBC2"),
            MbcKind::Mmm01 => write!(f, "MMM01"),
            MbcKind::Mbc3 => write!(f, "MBC3"),
            MbcKind::Mbc5 => write!(f, "MBC5"),
            MbcKind::Mbc6 => write!(f, "MBC6"),
            MbcKind::Mbc7 => write!(f, "MBC7"),
            MbcKind::HuC1 => write!(f, "HuC1"),
            MbcKind::HuC3 => write!(f, "HuC3"),
            MbcKind::Unknown(v) => write!(f, "Unknown (0x{v:02X})"),
        }
    }
}

/// Parsed subset of cartridge header information for emulator setup/logging.
#[derive(Debug, Clone)]
pub struct CartridgeHeader {
    /// Raw 0x0147 value.
    pub cartridge_type: u8,
    /// Human readable cartridge type.
    pub cartridge_type_name: &'static str,

    /// Derived MBC family from cartridge type.
    pub mbc_kind: MbcKind,

    /// Raw 0x0148 value.
    pub rom_size_code: u8,
    /// Number of ROM banks (16 KiB each), if known.
    pub rom_banks: Option<usize>,
    /// ROM size in bytes, if known.
    pub rom_size_bytes: Option<usize>,

    /// Raw 0x0149 value.
    pub ram_size_code: u8,
    /// Number of external RAM banks (8 KiB each), if known.
    pub ram_banks: Option<usize>,
    /// External RAM size in bytes, if known.
    pub ram_size_bytes: Option<usize>,

    /// Raw checksum value from 0x014D.
    pub header_checksum_stored: u8,
    /// Computed checksum from bytes 0x0134..=0x014C.
    pub header_checksum_computed: u8,
    /// Whether stored checksum matches computed checksum.
    pub header_checksum_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartridgeHeaderError {
    RomTooSmall { len: usize, required_min: usize },
}

impl fmt::Display for CartridgeHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CartridgeHeaderError::RomTooSmall { len, required_min } => {
                write!(
                    f,
                    "ROM too small for header parsing (len={len}, required_min={required_min})"
                )
            }
        }
    }
}

impl std::error::Error for CartridgeHeaderError {}

impl CartridgeHeader {
    /// Parse the basic cartridge fields from full ROM data.
    ///
    /// Requires at least `0x150` bytes to include header through 0x014F.
    pub fn parse(rom: &[u8]) -> Result<Self, CartridgeHeaderError> {
        Self::try_from(rom)
    }

    /// Compact one-line summary.
    pub fn summary_line(&self) -> String {
        let rom = match (self.rom_size_bytes, self.rom_banks) {
            (Some(bytes), Some(banks)) => format!("{} ({} banks)", format_bytes(bytes), banks),
            _ => format!("unknown (code 0x{:02X})", self.rom_size_code),
        };

        let ram = match (self.ram_size_bytes, self.ram_banks) {
            (Some(bytes), Some(banks)) => format!("{} ({} banks)", format_bytes(bytes), banks),
            _ => format!("unknown (code 0x{:02X})", self.ram_size_code),
        };

        format!(
            "MBC: {} | Type: {} (0x{:02X}) | ROM: {} | RAM: {} | Header checksum: {} (stored=0x{:02X}, computed=0x{:02X})",
            self.mbc_kind,
            self.cartridge_type_name,
            self.cartridge_type,
            rom,
            ram,
            if self.header_checksum_valid { "OK" } else { "BAD" },
            self.header_checksum_stored,
            self.header_checksum_computed
        )
    }
}

impl TryFrom<&[u8]> for CartridgeHeader {
    type Error = CartridgeHeaderError;

    fn try_from(rom: &[u8]) -> Result<Self, Self::Error> {
        if rom.len() <= HEADER_END {
            return Err(CartridgeHeaderError::RomTooSmall {
                len: rom.len(),
                required_min: HEADER_END + 1,
            });
        }

        let cart_type = rom[CARTRIDGE_TYPE_ADDR];
        let mbc_kind = mbc_from_cartridge_type(cart_type);

        let rom_size_code = rom[ROM_SIZE_ADDR];
        let ram_size_code = rom[RAM_SIZE_ADDR];

        let (rom_banks, rom_size_bytes) = decode_rom_size(rom_size_code);
        let (mut ram_banks, mut ram_size_bytes) = decode_ram_size(ram_size_code);

        // MBC2 has built-in RAM in the controller: 512 x 4-bit values (stored in 512 bytes).
        // This is not external cartridge RAM decoded from 0x0149.
        if mbc_kind == MbcKind::Mbc2 {
            ram_banks = Some(1);
            ram_size_bytes = Some(512);
        }

        let stored = rom[HEADER_CHECKSUM_ADDR];
        let computed = compute_header_checksum(rom);
        let valid = stored == computed;

        Ok(Self {
            cartridge_type: cart_type,
            cartridge_type_name: cartridge_type_name(cart_type),
            mbc_kind,
            rom_size_code,
            rom_banks,
            rom_size_bytes,
            ram_size_code,
            ram_banks,
            ram_size_bytes,
            header_checksum_stored: stored,
            header_checksum_computed: computed,
            header_checksum_valid: valid,
        })
    }
}

pub fn compute_header_checksum(rom: &[u8]) -> u8 {
    if rom.len() <= HEADER_CHECKSUM_RANGE_END {
        return 0;
    }

    let mut x: u8 = 0;
    for &b in &rom[HEADER_CHECKSUM_RANGE_START..=HEADER_CHECKSUM_RANGE_END] {
        x = x.wrapping_sub(b).wrapping_sub(1);
    }
    x
}

fn mbc_from_cartridge_type(code: u8) -> MbcKind {
    match code {
        0x00 | 0x08 | 0x09 => MbcKind::None,

        0x01..=0x03 => MbcKind::Mbc1,
        0x05 | 0x06 => MbcKind::Mbc2,
        0x0B..=0x0D => MbcKind::Mmm01,
        0x0F..=0x13 => MbcKind::Mbc3,
        0x19..=0x1E => MbcKind::Mbc5,
        0x20 => MbcKind::Mbc6,
        0x22 => MbcKind::Mbc7,
        0xFE => MbcKind::HuC3,
        0xFF => MbcKind::HuC1,

        _ => MbcKind::Unknown(code),
    }
}

fn cartridge_type_name(code: u8) -> &'static str {
    match code {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+BATTERY",
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        0x1C => "MBC5+RUMBLE",
        0x1D => "MBC5+RUMBLE+RAM",
        0x1E => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xFC => "POCKET CAMERA",
        0xFD => "BANDAI TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+BATTERY",
        _ => "UNKNOWN",
    }
}

fn decode_rom_size(code: u8) -> (Option<usize>, Option<usize>) {
    // Banks are 16 KiB each.
    let banks = match code {
        0x00..=0x08 => Some(2usize << code),
        0x52 => Some(72), // 1.1 MiB
        0x53 => Some(80), // 1.2 MiB
        0x54 => Some(96), // 1.5 MiB
        _ => None,
    };

    let bytes = banks.map(|b| b * 16 * 1024);
    (banks, bytes)
}

fn decode_ram_size(code: u8) -> (Option<usize>, Option<usize>) {
    // External RAM size codes from cartridge header (0x0149).
    // Note: MBC2 uses internal 512 x 4-bit RAM and is handled separately.
    match code {
        0x00 => (Some(0), Some(0)),
        0x01 => (Some(1), Some(2 * 1024)), // 2 KiB (legacy/homebrew)
        0x02 => (Some(1), Some(8 * 1024)), // 8 KiB
        0x03 => (Some(4), Some(32 * 1024)), // 32 KiB
        0x04 => (Some(16), Some(128 * 1024)), // 128 KiB
        0x05 => (Some(8), Some(64 * 1024)), // 64 KiB
        _ => (None, None),
    }
}

fn format_bytes(bytes: usize) -> String {
    const KIB: usize = 1024;
    const MIB: usize = 1024 * 1024;

    if bytes >= MIB {
        if bytes % MIB == 0 {
            format!("{} MiB", bytes / MIB)
        } else {
            format!("{:.2} MiB", bytes as f64 / MIB as f64)
        }
    } else if bytes >= KIB {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[allow(dead_code)]
fn _header_present(rom: &[u8]) -> bool {
    rom.len() > HEADER_START
}
