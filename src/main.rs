//! Game Boy emulator - main entry point.
//!
//! This module orchestrates the emulation loop, loading ROMs and running CPU cycles
//! with per-cycle hardware ticking (timer, GPU, etc.).

mod cpu;
mod flag_helpers;
mod gpu;
mod instructions;
mod interrupts;
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
                // Tick timer once per T-cycle. Timer interrupt is automatically
                // requested via the interrupt controller when TIMA overflows.
                cpu.bus.tick_timer();

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
