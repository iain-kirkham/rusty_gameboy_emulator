//! Memory Bus implementing the Game Boy memory map.
//!
//! Reference: [Pan Docs — Memory Map](https://gbdev.io/pandocs/Memory_Map.html)
//!
//! This module implements the emulator's memory map. See the linked pandocs page
//! for the canonical description of each memory region.

use crate::gpu;

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

const INTERRUPT_ENABLE_REGISTER: usize = 0xFFFF;

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
    pub gpu: gpu::GPU,
    pub serial_output: Vec<u8>,
}

impl MemoryBus {
    pub fn new(rom_data: Vec<u8>) -> MemoryBus {
        let mut memory = [0u8; MEM_SIZE];
        let copy_len = std::cmp::min(rom_data.len(), MEM_SIZE);
        memory[..copy_len].copy_from_slice(&rom_data[..copy_len]);

        MemoryBus {
            memory,
            gpu: gpu::GPU::new(),
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
            SERIAL_TRANSFER_DATA => {
                // SB (Serial transfer data) - return the last byte that was written
                // Used by Blargg test ROMs to output characters
                self.serial_output.last().copied().unwrap_or(0)
            }
            SERIAL_TRANSFER_CONTROL => {
                // SC (Serial transfer control) - bit 7 indicates transfer in progress
                SERIAL_CONTROL_IDLE
            }
            IO_REGISTERS_START..=IO_REGISTERS_END => self.memory[address],
            HRAM_START..=HRAM_END => self.memory[address],
            INTERRUPT_ENABLE_REGISTER => self.memory[address],
            _ => UNMAPPED_MEMORY_VALUE,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            ROM_START..=ROM_END => {} // ROM - ignore writes
            VRAM_START..=VRAM_END => self.gpu.write_vram(address - VRAM_OFFSET, value),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.memory[address] = value,
            WORK_RAM_START..=ECHO_RAM_END => self.memory[address] = value,
            OAM_START..=OAM_END => self.memory[address] = value,
            SERIAL_TRANSFER_DATA => {
                self.serial_output.push(value);
            }
            SERIAL_TRANSFER_CONTROL => {
                // Ignore for now
            }
            IO_REGISTERS_START..=IO_REGISTERS_END => self.memory[address] = value,
            HRAM_START..=HRAM_END => self.memory[address] = value,
            INTERRUPT_ENABLE_REGISTER => self.memory[address] = value,
            _ => {} // Ignore writes to unmapped areas
        }
    }
}
