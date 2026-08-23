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

use crate::cpu::{CpuTraceState, CPU};

/// Result of running a ROM until it reports pass/fail (or times out).
pub struct HeadlessRunResult {
    /// All serial output captured during the run.
    pub serial_output: String,
    /// Total T-states executed.
    pub t_cycles: u64,
    /// True if the run stopped because it hit `max_t_cycles` without seeing
    /// a recognizable pass/fail marker in the serial output.
    pub timed_out: bool,
}

/// Observes a ROM run driven by [`run`], without owning the run loop itself.
///
/// All methods are no-ops by default, so callers only implement the ones
/// they care about (e.g. a doctor-log writer implements only `on_trace`, a
/// progress printer only `on_step`).
pub trait RunObserver {
    /// Called once per CPU step that actually executed an opcode (i.e. when
    /// [`CPU::last_step_executed_opcode`] is true), with that step's trace.
    fn on_trace(&mut self, _trace: &CpuTraceState) {}

    /// Called whenever a new chunk of serial output arrives, before it's
    /// checked against the run's stop markers.
    fn on_serial_chunk(&mut self, _chunk: &str) {}

    /// Called once per CPU step, regardless of whether it produced a trace
    /// or serial output, with the cumulative T-cycle count so far.
    fn on_step(&mut self, _cycle_count: u64) {}
}

/// A [`RunObserver`] that observes nothing; used by [`run_headless`].
struct NoopObserver;
impl RunObserver for NoopObserver {}

/// Run a ROM until its serial output contains one of `stop_markers`, or
/// `max_t_cycles` T-states have elapsed, reporting progress to `observer`.
///
/// This is the one run loop shared by the CLI binary (`main.rs`, which
/// supplies an observer that writes a Game Boy Doctor log and prints
/// progress) and headless test/tooling callers (via [`run_headless`], which
/// supplies a no-op observer).
pub fn run(
    rom_data: Vec<u8>,
    max_t_cycles: u64,
    stop_markers: &[&str],
    observer: &mut impl RunObserver,
) -> HeadlessRunResult {
    let mut cpu = CPU::new(rom_data);
    let mut t_cycles_total: u64 = 0;
    let mut serial_output = String::new();

    while t_cycles_total < max_t_cycles {
        let t_cycles = cpu.step() as usize;

        if cpu.last_step_executed_opcode() {
            if let Some(trace) = cpu.take_last_trace() {
                observer.on_trace(&trace);
            }
        }

        for _ in 0..t_cycles {
            cpu.bus.tick_timer();
            cpu.bus.tick_ppu(1);
        }

        t_cycles_total = t_cycles_total.wrapping_add(t_cycles as u64);
        observer.on_step(t_cycles_total);

        if cpu.bus.has_serial_output() {
            let chunk = cpu.bus.get_serial_output();
            cpu.bus.clear_serial_output();
            observer.on_serial_chunk(&chunk);
            serial_output.push_str(&chunk);

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

/// Run a ROM headlessly (no display, no doctor log, no progress output)
/// until its serial output contains one of `stop_markers`, or
/// `max_t_cycles` T-states have elapsed. A thin [`run`] wrapper for
/// test/tooling callers that don't need to observe the run.
pub fn run_headless(rom_data: Vec<u8>, max_t_cycles: u64, stop_markers: &[&str]) -> HeadlessRunResult {
    run(rom_data, max_t_cycles, stop_markers, &mut NoopObserver)
}
