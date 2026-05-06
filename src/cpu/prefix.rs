use super::CPU;
use crate::instructions::PrefixTarget;

pub(crate) trait PrefixOps {
    fn read_prefix_target(&mut self, target: PrefixTarget) -> u8;
    fn write_prefix_target(&mut self, target: PrefixTarget, value: u8);
}

impl PrefixOps for CPU {
    /// Read a value from a CB-prefixed instruction target (register or memory at HL).
    fn read_prefix_target(&mut self, target: PrefixTarget) -> u8 {
        if let Some(reg) = target.to_register8() {
            self.registers.read_8bit(reg)
        } else {
            self.bus.read_byte(self.registers.get_hl())
        }
    }

    /// Write a value to a CB-prefixed instruction target (register or memory at HL).
    fn write_prefix_target(&mut self, target: PrefixTarget, value: u8) {
        if let Some(reg) = target.to_register8() {
            self.registers.write_8bit(reg, value);
        } else {
            self.bus.write_byte(self.registers.get_hl(), value);
        }
    }
}
