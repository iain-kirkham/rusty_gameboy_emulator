//! Game Boy emulator - main entry point.
//!
//! This module orchestrates the emulation loop, loading ROMs and running CPU cycles
//! with per-cycle hardware ticking (timer, GPU, etc.).

use rusty_gameboy_emulator::cartridge_header::CartridgeHeader;
use rusty_gameboy_emulator::cpu::CpuTraceState;
use rusty_gameboy_emulator::display;
use rusty_gameboy_emulator::{run, RunObserver};
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;

const DEFAULT_TEST_ROM: &str = "blargg/cpu_instrs/individual/11-op a,(hl).gb";

/// Writes a Game Boy Doctor-format trace log and prints progress/serial
/// output to the console, matching the CLI's expected console behavior.
struct TestRunObserver {
    log_writer: BufWriter<fs::File>,
}

impl RunObserver for TestRunObserver {
    fn on_trace(&mut self, trace: &CpuTraceState) {
        let _ = writeln!(
            self.log_writer,
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

    fn on_serial_chunk(&mut self, chunk: &str) {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }

    fn on_step(&mut self, cycle_count: u64) {
        if cycle_count.is_multiple_of(1_000_000) {
            eprint!("\r Cycles: {}M...", cycle_count / 1_000_000);
            io::stderr().flush().unwrap();
        }
    }
}

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
        let mut observer = TestRunObserver {
            log_writer: BufWriter::new(log_file),
        };

        const MAX_CYCLES: u64 = 900_000_000; // 10 million T-states should be enough

        // Run the emulation until max cycles, or until the test ROM reports
        // a Passed/Failed result over serial.
        let result = run(rom_data, MAX_CYCLES, &["Passed", "Failed"], &mut observer);

        if result.timed_out {
            println!("\n Reached maximum cycle count ({})", MAX_CYCLES);
        }

        println!("\n==========================================\n");
    }
}
