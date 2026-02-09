//! Memory Bus implementing the Game Boy memory map.
//!
//! Reference: [Pan Docs — Memory Map](https://gbdev.io/pandocs/Memory_Map.html)
//!
//! This module implements the emulator's memory map. See the linked pandocs page
//! for the canonical description of each memory region.

use crate::interrupts::{Interrupt, InterruptController};
use crate::ppu;
use crate::timer::Timer;

/// Memory Bus implementing the Game Boy memory map:
///
/// 0x0000-0x3FFF : ROM Bank 0 (16KB) - Fixed bank
/// 0x4000-0x7FFF : ROM Bank 1-N (16KB) - Switchable via MBC
/// 0x8000-0x9FFF : VRAM (8KB) - Video RAM
/// 0xA000-0xBFFF : External RAM (8KB) - Cartridge RAM (if present)
/// 0xC000-0xCFFF : Work RAM Bank 0 (4KB)
/// 0xD000-0xDFFF : Work RAM Bank 1 (4KB)
/// 0xE000-0xFDFF : Echo RAM (mirror of 0xC000-0xDDFF)
/// 0xFE00-0xFE9F : OAM (Object Attribute Memory) - Sprite data
/// 0xFEA0-0xFEFF : Unusable memory
/// 0xFF00-0xFF7F : I/O Registers
/// 0xFF80-0xFFFE : High RAM (HRAM) - Fast internal RAM
/// 0xFFFF        : Interrupt Enable Register
pub const MEM_SIZE: usize = 0x10000; // 64KB = 65,536 bytes (0x0000 - 0xFFFF + 1)

//Memory region boundaries
const ROM_START: usize = 0x0000;
const ROM_END: usize = 0x7FFF;

const VRAM_START: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;

const EXTERNAL_RAM_START: usize = 0xA000;
const EXTERNAL_RAM_END: usize = 0xBFFF;

const WORK_RAM_START: usize = 0xC000;
const WORK_RAM_BANK0_END: usize = 0xCFFF;
const WORK_RAM_BANK1_START: usize = 0xD000;
const WORK_RAM_BANK1_END: usize = 0xDFFF;

const ECHO_RAM_START: usize = 0xE000;
const ECHO_RAM_END: usize = 0xFDFF;

const OAM_START: usize = 0xFE00;
const OAM_END: usize = 0xFE9F;

const IO_REGISTERS_START: usize = 0xFF00;
const IO_REGISTERS_END: usize = 0xFF7F;

const HRAM_START: usize = 0xFF80;
const HRAM_END: usize = 0xFFFE;

// Specific I/O register addresses
const SERIAL_TRANSFER_DATA: usize = 0xFF01; // SB register
const SERIAL_TRANSFER_CONTROL: usize = 0xFF02; // SC register

// Memory offsets
const VRAM_OFFSET: usize = VRAM_START;
const ECHO_RAM_MIRROR_OFFSET: usize = 0x2000;

// Default values
const SERIAL_CONTROL_IDLE: u8 = 0x7E; // Bit 7 = 0 (no transfer in progress)
const UNMAPPED_MEMORY_VALUE: u8 = 0xFF;

pub struct MemoryBus {
    pub memory: [u8; MEM_SIZE],
    pub gpu: ppu::GPU,
    pub timer: Timer,
    pub interrupts: InterruptController,
    pub serial_output: Vec<u8>,
}

impl MemoryBus {
    pub fn new(rom_data: Vec<u8>) -> MemoryBus {
        let mut memory = [0u8; MEM_SIZE];
        let copy_len = std::cmp::min(rom_data.len(), MEM_SIZE);
        memory[..copy_len].copy_from_slice(&rom_data[..copy_len]);

        // Initialize serial registers to sensible defaults so reads behave predictably.
        memory[SERIAL_TRANSFER_DATA] = 0x00;
        memory[SERIAL_TRANSFER_CONTROL] = SERIAL_CONTROL_IDLE;

        MemoryBus {
            memory,
            gpu: ppu::GPU::new(),
            timer: Timer::new(),
            interrupts: InterruptController::new(),
            serial_output: Vec::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            ROM_START..=ROM_END => self.memory[address],
            VRAM_START..=VRAM_END => self.gpu.read_vram(address - VRAM_OFFSET),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.memory[address],
            WORK_RAM_START..=WORK_RAM_BANK0_END => self.memory[address],
            WORK_RAM_BANK1_START..=WORK_RAM_BANK1_END => self.memory[address],
            ECHO_RAM_START..=ECHO_RAM_END => {
                // Echo RAM - mirrors Work RAM
                let mirror_address = address - ECHO_RAM_MIRROR_OFFSET;
                self.memory[mirror_address]
            }
            OAM_START..=OAM_END => self.memory[address],
            SERIAL_TRANSFER_DATA | SERIAL_TRANSFER_CONTROL => self.memory[address],
            // Timer registers (0xFF04-0xFF07) are handled by the timer module
            0xFF04 | 0xFF05 | 0xFF06 | 0xFF07 => self.timer.read(address as u16),
            // LCD registers (0xFF40-0xFF4B) are handled by the PPU
            0xFF40..=0xFF4B => self.gpu.read_register(address as u16),
            // Interrupt Flag register (0xFF0F)
            0xFF0F => self.interrupts.read_if(),
            IO_REGISTERS_START..=IO_REGISTERS_END => self.memory[address],
            HRAM_START..=HRAM_END => self.memory[address],
            // Interrupt Enable register (0xFFFF)
            0xFFFF => self.interrupts.read_ie(),
            _ => UNMAPPED_MEMORY_VALUE,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            ROM_START..=ROM_END => {} // ROM - ignore writes
            VRAM_START..=VRAM_END => self.gpu.write_vram(address - VRAM_OFFSET, value),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.memory[address] = value,
            WORK_RAM_START..=WORK_RAM_BANK1_END => self.memory[address] = value,
            ECHO_RAM_START..=ECHO_RAM_END => {
                self.memory[address - ECHO_RAM_MIRROR_OFFSET] = value;
            }
            OAM_START..=OAM_END => self.memory[address] = value,
            SERIAL_TRANSFER_DATA => {
                // Store the value in the SB hardware register so reads return it
                self.memory[address] = value;
            }
            SERIAL_TRANSFER_CONTROL => {
                // If bit 7 is set, start a transfer. Use `read_byte` so any special
                // behavior for reading SB (0xFF01) is preserved.
                if value & 0x80 != 0 {
                    let character = self.read_byte(0xFF01);
                    self.serial_output.push(character);

                    // Reset bit 7 to signal transfer complete while preserving other bits
                    self.memory[SERIAL_TRANSFER_CONTROL] = value & 0x7F;
                } else {
                    // For other writes, store the value in the SC register
                    self.memory[address] = value;
                }
            }
            // Timer registers (0xFF04-0xFF07) are handled by the timer module
            0xFF04 | 0xFF05 | 0xFF06 | 0xFF07 => {
                self.timer.write(address as u16, value);
            }
            // LCD registers (0xFF40-0xFF4B) are handled by the PPU
            0xFF40..=0xFF4B => self.gpu.write_register(address as u16, value),
            // Interrupt Flag register (0xFF0F)
            0xFF0F => self.interrupts.write_if(value),
            IO_REGISTERS_START..=IO_REGISTERS_END => self.memory[address] = value,
            HRAM_START..=HRAM_END => self.memory[address] = value,
            // Interrupt Enable register (0xFFFF)
            0xFFFF => self.interrupts.write_ie(value),
            _ => {} // Ignore writes to unmapped areas
        }
    }

    /// Check if there's serial output available and return it as a string
    /// This is used by Blargg test ROMs to output test results
    pub fn get_serial_output(&self) -> String {
        String::from_utf8_lossy(&self.serial_output).to_string()
    }

    /// Clear the serial output buffer
    pub fn clear_serial_output(&mut self) {
        self.serial_output.clear();
    }

    /// Check if serial output has content
    pub fn has_serial_output(&self) -> bool {
        !self.serial_output.is_empty()
    }

    /// Tick the timer by one T-cycle and request timer interrupt if needed.
    ///
    /// This must be called once per T-cycle in the emulation loop.
    /// If the timer overflows, the timer interrupt is automatically requested.
    pub fn tick_timer(&mut self) {
        if self.timer.tick() {
            self.interrupts.request_interrupt(Interrupt::Timer);
        }
    }

    /// Request an interrupt.
    #[allow(dead_code)]
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupts.request_interrupt(interrupt);
    }

    /// Check if any interrupt is pending (for HALT wake-up).
    pub fn any_interrupt_pending(&self) -> bool {
        self.interrupts.any_interrupt_pending()
    }
}
