use super::CPU;
use super::FetchTraceOps;
use crate::instructions::{LoadByteSource, LoadByteTarget};

pub(crate) trait LoadOps {
    fn read_byte_source(&mut self, source: LoadByteSource) -> u8;
    fn write_byte_target(&mut self, target: LoadByteTarget, value: u8);
    fn get_load_byte_pc_increment(
        &self,
        target: LoadByteTarget,
        source: LoadByteSource,
    ) -> u16;
    fn get_load_byte_cycles(&self, target: LoadByteTarget, source: LoadByteSource) -> u16;
}

impl LoadOps for CPU {
    /// Read a byte from the specified source (register, memory location, or immediate value).
    /// Handles all LoadByteSource variants including indirect addressing modes.
    fn read_byte_source(&mut self, source: LoadByteSource) -> u8 {
        match source {
            LoadByteSource::A => self.registers.a,
            LoadByteSource::B => self.registers.b,
            LoadByteSource::C => self.registers.c,
            LoadByteSource::D => self.registers.d,
            LoadByteSource::E => self.registers.e,
            LoadByteSource::H => self.registers.h,
            LoadByteSource::L => self.registers.l,
            LoadByteSource::D8 => self.read_next_byte(),
            LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
            LoadByteSource::BCI => self.bus.read_byte(self.registers.get_bc()),
            LoadByteSource::DEI => self.bus.read_byte(self.registers.get_de()),
            LoadByteSource::HLI_INC => {
                let value = self.bus.read_byte(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                value
            }
            LoadByteSource::HLI_DEC => {
                let value = self.bus.read_byte(self.registers.get_hl());
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                value
            }
            LoadByteSource::A16I => {
                let address = self.read_next_word();
                self.bus.read_byte(address)
            }
            LoadByteSource::A8I => {
                let offset = self.read_next_byte();
                let address = 0xFF00 + offset as u16;
                self.bus.read_byte(address)
            }
            LoadByteSource::CI => {
                let address = 0xFF00 + self.registers.c as u16;
                self.bus.read_byte(address)
            }
        }
    }

    /// Write a byte to the specified target (register, memory location, or I/O address).
    /// Handles all LoadByteTarget variants including indirect addressing modes.
    fn write_byte_target(&mut self, target: LoadByteTarget, value: u8) {
        match target {
            LoadByteTarget::A => self.registers.a = value,
            LoadByteTarget::B => self.registers.b = value,
            LoadByteTarget::C => self.registers.c = value,
            LoadByteTarget::D => self.registers.d = value,
            LoadByteTarget::E => self.registers.e = value,
            LoadByteTarget::H => self.registers.h = value,
            LoadByteTarget::L => self.registers.l = value,
            LoadByteTarget::HLI => self.bus.write_byte(self.registers.get_hl(), value),
            LoadByteTarget::DEI => self.bus.write_byte(self.registers.get_de(), value),
            LoadByteTarget::BCI => self.bus.write_byte(self.registers.get_bc(), value),
            LoadByteTarget::A16I => {
                let address = self.read_next_word();
                self.bus.write_byte(address, value);
            }
            LoadByteTarget::A8I => {
                let offset = self.read_next_byte();
                let address = 0xFF00 + offset as u16;
                self.bus.write_byte(address, value);
            }
            LoadByteTarget::HLI_INC => {
                let address = self.registers.get_hl();
                self.bus.write_byte(address, value);
                self.registers.set_hl(address.wrapping_add(1));
            }
            LoadByteTarget::HLI_DEC => {
                let address = self.registers.get_hl();
                self.bus.write_byte(address, value);
                self.registers.set_hl(address.wrapping_sub(1));
            }
            LoadByteTarget::CI => {
                let address = 0xFF00 + self.registers.c as u16;
                self.bus.write_byte(address, value);
            }
        }
    }

    /// Calculate how much the PC should advance based on the load source.
    fn get_load_byte_pc_increment(
        &self,
        target: LoadByteTarget,
        source: LoadByteSource,
    ) -> u16 {
        match (target, source) {
            (LoadByteTarget::A16I, _) => 3,
            (LoadByteTarget::A8I, _) => 2,

            (_, LoadByteSource::A16I) => 3,
            (_, LoadByteSource::A8I) => 2,
            (_, LoadByteSource::D8) => 2,
            _ => 1,
        }
    }

    /// Compute T-cycle cost for LD byte operations based on both target and source.
    fn get_load_byte_cycles(&self, target: LoadByteTarget, source: LoadByteSource) -> u16 {
        let base = match source {
            LoadByteSource::D8 => 8,
            LoadByteSource::A16I => 16,
            LoadByteSource::A8I => 12,
            LoadByteSource::HLI => 8,
            LoadByteSource::HLI_INC => 8,
            LoadByteSource::HLI_DEC => 8,
            LoadByteSource::BCI => 8,
            LoadByteSource::DEI => 8,
            LoadByteSource::CI => 8,
            LoadByteSource::A => 4,
            LoadByteSource::B => 4,
            LoadByteSource::C => 4,
            LoadByteSource::D => 4,
            LoadByteSource::E => 4,
            LoadByteSource::H => 4,
            LoadByteSource::L => 4,
        };

        // Additional cost when the target is a memory location (different for
        // no-immediate, 8-bit immediate, and 16-bit immediate target addressing)
        let extra = match target {
            LoadByteTarget::HLI
            | LoadByteTarget::HLI_INC
            | LoadByteTarget::HLI_DEC
            | LoadByteTarget::BCI
            | LoadByteTarget::DEI
            | LoadByteTarget::CI => 4,
            LoadByteTarget::A8I => 8,
            LoadByteTarget::A16I => 12,
            _ => 0,
        };

        base + extra
    }
}
