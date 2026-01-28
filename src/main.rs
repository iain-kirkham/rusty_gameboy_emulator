//! Game Boy emulator - main entry point.
//!
//! This module orchestrates the emulation loop, loading ROMs and running CPU cycles
//! with per-cycle hardware ticking (timer, GPU, etc.).

mod cpu;
mod gpu;
mod instructions;
mod memory_bus;
mod register;
mod timer;

use crate::cpu::CPU;
use std::fs;
use std::io::{self, Write};

fn main() {
    let test_roms = vec!["blargg/cpu_instrs/individual/01-special.gb"];

    for rom_path in test_roms {
        println!("==========================================");
        println!("Running test: {}", rom_path);
        println!("==========================================\n");

        let rom_data = match fs::read(rom_path) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to read ROM file: {}", e);
                println!("Make sure the ROM exists at: {}\n", rom_path);
                continue;
            }
        };

        let mut cpu = CPU::new(rom_data);
        let mut cycle_count: u64 = 0;
        const MAX_CYCLES: u64 = 10_000_000; // 10 million T-states should be enough

        // Run the emulation until max cycles or until CPU halts
        while cycle_count < MAX_CYCLES {
            if cpu.is_halted() {
                println!("\n CPU halted after {} cycles", cycle_count);
                break;
            }

            let t_cycles = cpu.step() as usize;

            // Advance per-T-cycle hardware (Timer, GPU/PPU, DMA, etc.)
            for _ in 0..t_cycles {
                // Tick timer once per T-cycle. Returns true when the TIMA->TMA reload finished,
                // which indicates the timer interrupt should be requested.
                let timer_interrupt = cpu.bus.tick_timer();

                if timer_interrupt {
                    // Request Timer interrupt by setting IF bit 2 (0xFF0F bit mask 0b0000_0100).
                    // Replace this with interrupt controller.
                    let if_addr: u16 = 0xFF0F;
                    let current_if = cpu.bus.read_byte(if_addr);
                    cpu.bus.write_byte(if_addr, current_if | 0b0000_0100);
                }

                // TODO: Tick other per-T-cycle systems here (GPU/PPU, DMA timing, etc.)
            }

            cycle_count = cycle_count.wrapping_add(t_cycles as u64);

            // Check for serial output and print it immediately
            if cpu.bus.has_serial_output() {
                let output = cpu.bus.get_serial_output();
                print!("{}", output);
                io::stdout().flush().unwrap();
                cpu.bus.clear_serial_output();
            }

            // Print progress every million cycles
            if cycle_count % 1_000_000 == 0 {
                eprint!("\r Cycles: {}M...", cycle_count / 1_000_000);
                io::stderr().flush().unwrap();
            }
        }

        if cycle_count >= MAX_CYCLES {
            println!("\n Reached maximum cycle count ({})", MAX_CYCLES);
        }

        // Print any remaining serial output
        if cpu.bus.has_serial_output() {
            let output = cpu.bus.get_serial_output();
            print!("{}", output);
        }

        println!("\n==========================================\n");
    }
}
