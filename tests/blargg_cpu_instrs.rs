//! Integration tests that run each individual Blargg `cpu_instrs` test ROM
//! headlessly and assert it reports "Passed" over serial.
//!
//! ROMs live in `blargg/cpu_instrs/individual/`. Other Blargg test suites
//! (instr_timing, mem_timing, halt_bug, etc.) can be added the same way
//! later once the PPU/timing support they need is in place.

use rusty_gameboy_emulator::run_headless;
use std::fs;

/// Individual test ROMs run in well under a second of emulated CPU work;
/// this cap is only meant to catch a genuine hang/infinite loop.
const MAX_T_CYCLES: u64 = 200_000_000;

/// Run one Blargg `cpu_instrs` ROM to completion and assert it reports Passed.
fn run_blargg_cpu_instrs_rom(rom_path: &str) {
    let rom_data = fs::read(rom_path)
        .unwrap_or_else(|e| panic!("failed to read ROM at {rom_path}: {e}"));

    let result = run_headless(rom_data, MAX_T_CYCLES, &["Passed", "Failed"]);

    assert!(
        !result.timed_out,
        "{rom_path}: timed out after {} T-cycles without a Passed/Failed result; serial output so far:\n{}",
        result.t_cycles, result.serial_output
    );
    assert!(
        result.serial_output.contains("Passed"),
        "{rom_path}: test ROM reported failure; serial output:\n{}",
        result.serial_output
    );
}

macro_rules! blargg_cpu_instrs_test {
    ($test_name:ident, $rom_file:expr) => {
        #[test]
        fn $test_name() {
            run_blargg_cpu_instrs_rom(concat!(
                "blargg/cpu_instrs/individual/",
                $rom_file
            ));
        }
    };
}

blargg_cpu_instrs_test!(cpu_instrs_01_special, "01-special.gb");
blargg_cpu_instrs_test!(cpu_instrs_02_interrupts, "02-interrupts.gb");
blargg_cpu_instrs_test!(cpu_instrs_03_op_sp_hl, "03-op sp,hl.gb");
blargg_cpu_instrs_test!(cpu_instrs_04_op_r_imm, "04-op r,imm.gb");
blargg_cpu_instrs_test!(cpu_instrs_05_op_rp, "05-op rp.gb");
blargg_cpu_instrs_test!(cpu_instrs_06_ld_r_r, "06-ld r,r.gb");
blargg_cpu_instrs_test!(cpu_instrs_07_jr_jp_call_ret_rst, "07-jr,jp,call,ret,rst.gb");
blargg_cpu_instrs_test!(cpu_instrs_08_misc_instrs, "08-misc instrs.gb");
blargg_cpu_instrs_test!(cpu_instrs_09_op_r_r, "09-op r,r.gb");
blargg_cpu_instrs_test!(cpu_instrs_10_bit_ops, "10-bit ops.gb");
blargg_cpu_instrs_test!(cpu_instrs_11_op_a_hl, "11-op a,(hl).gb");
