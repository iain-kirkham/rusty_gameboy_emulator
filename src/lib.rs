//! Game Boy emulator library.
//!
//! Exposes the CPU/memory bus/cartridge types plus a headless-run helper so
//! both the CLI binary (`src/main.rs`) and integration tests (`tests/`) can
//! drive the emulator without duplicating the run loop.

pub mod cartridge_header;
pub mod cpu;
pub mod display;
pub mod flag_helpers;
pub mod instructions;
pub mod interrupts;
pub mod memory_bus;
pub mod ppu;
pub mod register;
pub mod timer;

use crate::cpu::CPU;

/// Result of running a ROM headlessly until it reports pass/fail (or times out).
pub struct HeadlessRunResult {
    /// All serial output captured during the run.
    pub serial_output: String,
    /// Total T-states executed.
    pub t_cycles: u64,
    /// True if the run stopped because it hit `max_t_cycles` without seeing
    /// a recognizable pass/fail marker in the serial output.
    pub timed_out: bool,
}

/// Run a ROM headlessly (no display, no doctor log) until its serial output
/// contains one of `stop_markers`, or `max_t_cycles` T-states have elapsed.
///
/// This mirrors the run loop in `main.rs` (per-T-cycle timer/PPU ticking)
/// but is trimmed down for test use: no trace log file, and it stops early
/// once a pass/fail marker shows up instead of always running to the cap.
pub fn run_headless(rom_data: Vec<u8>, max_t_cycles: u64, stop_markers: &[&str]) -> HeadlessRunResult {
    let mut cpu = CPU::new(rom_data);
    let mut t_cycles_total: u64 = 0;
    let mut serial_output = String::new();

    while t_cycles_total < max_t_cycles {
        let t_cycles = cpu.step() as usize;

        for _ in 0..t_cycles {
            cpu.bus.tick_timer();
            cpu.bus.tick_ppu(1);
        }

        t_cycles_total = t_cycles_total.wrapping_add(t_cycles as u64);

        if cpu.bus.has_serial_output() {
            serial_output.push_str(&cpu.bus.get_serial_output());
            cpu.bus.clear_serial_output();

            if stop_markers.iter().any(|m| serial_output.contains(m)) {
                return HeadlessRunResult {
                    serial_output,
                    t_cycles: t_cycles_total,
                    timed_out: false,
                };
            }
        }
    }

    HeadlessRunResult {
        serial_output,
        t_cycles: t_cycles_total,
        timed_out: true,
    }
}
