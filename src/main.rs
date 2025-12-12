mod cpu;
mod gpu;
mod instructions;
mod memory_bus;
mod register;

use crate::cpu::CPU;
use std::fs;

fn main() {
    let test_roms = vec!["blargg/cpu_instrs/individual/01-special.gb"];

    for rom_path in test_roms {
        println!("Running test: {}", rom_path);
        let rom_data = fs::read(rom_path).expect("Failed to read ROM file");
        let mut cpu = CPU::new(rom_data);

        // Run the emulation for a fixed number of cycles or until a condition is met
        for _ in 0..1_000_000 {
            if cpu.is_halted() {
                break;
            }
            cpu.step();
        }
        println!("\n");
    }
}
