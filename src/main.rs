//! Game Boy emulator - main entry point.
//!
//! This module orchestrates the emulation loop, loading ROMs and running CPU cycles
//! with per-cycle hardware ticking (timer, GPU, etc.).

use rusty_gameboy_emulator::cartridge_header::CartridgeHeader;
use rusty_gameboy_emulator::cpu::{CpuTraceState, CPU};
use rusty_gameboy_emulator::display;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;

fn write_doctor_log_line(trace: &CpuTraceState, writer: &mut BufWriter<fs::File>) {
    let _ = writeln!(
        writer,
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
        trace.a,
        trace.f,
        trace.b,
        trace.c,
        trace.d,
        trace.e,
        trace.h,
        trace.l,
        trace.sp,
        trace.pc,
        trace.pcmem[0],
        trace.pcmem[1],
        trace.pcmem[2],
        trace.pcmem[3]
    );
}

const DEFAULT_TEST_ROM: &str = "blargg/cpu_instrs/individual/11-op a,(hl).gb";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--display") {
        // Basic minifb window scaffold - displays a static test pattern at
        // GB screen resolution. Not yet wired to real PPU/CPU output.
        display::run_test_screen();
        return;
    }

    // Headless mode (default): run CPU test ROMs with no window, printing
    // serial output and writing a Game Boy Doctor-format trace log.
    let rom_paths: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--")).collect();
    let rom_paths = if rom_paths.is_empty() {
        vec![DEFAULT_TEST_ROM.to_string()]
    } else {
        rom_paths
    };

    run_cpu_test_roms(&rom_paths);
}

fn run_cpu_test_roms(test_roms: &[String]) {
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

        match CartridgeHeader::parse(&rom_data) {
            Ok(header) => {
                println!("{}", header.summary_line());
            }
            Err(e) => {
                println!("Could not parse cartridge header: {e}");
            }
        }

        let log_name = Path::new(rom_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("doctor");
        let log_path = format!("doctor_{}.log", log_name);
        let log_file = fs::File::create(&log_path).expect("Failed to create doctor log file");
        let mut log_writer = BufWriter::new(log_file);

        let mut cpu = CPU::new(rom_data);
        let mut cycle_count: u64 = 0;
        const MAX_CYCLES: u64 = 900_000_000; // 10 million T-states should be enough
        let mut serial_output = String::new();
        let mut result_seen = false;

        // Run the emulation until max cycles, until CPU halts, or until the
        // test ROM reports a Passed/Failed result over serial.
        while cycle_count < MAX_CYCLES && !result_seen {

            let t_cycles = cpu.step() as usize;

            if cpu.last_step_executed_opcode() {
                if let Some(trace) = cpu.take_last_trace() {
                    write_doctor_log_line(&trace, &mut log_writer);
                }
            }

            // Advance per-T-cycle hardware (Timer, GPU/PPU, DMA, etc.)
            for _ in 0..t_cycles {
                // Tick timer once per T-cycle. Timer interrupt is automatically
                // requested via the interrupt controller when TIMA overflows.
                cpu.bus.tick_timer();

                // Advance PPU timing one T-cycle at a time.
                cpu.bus.gpu.tick(1);

                // TODO: Tick other per-T-cycle systems here (GPU/PPU, DMA timing, etc.)
            }

            cycle_count = cycle_count.wrapping_add(t_cycles as u64);

            // Check for serial output and print it immediately
            if cpu.bus.has_serial_output() {
                let output = cpu.bus.get_serial_output();
                print!("{}", output);
                io::stdout().flush().unwrap();
                serial_output.push_str(&output);
                cpu.bus.clear_serial_output();

                if serial_output.contains("Passed") || serial_output.contains("Failed") {
                    result_seen = true;
                }
            }

            // Print progress every million cycles
            if cycle_count.is_multiple_of(1_000_000) {
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
