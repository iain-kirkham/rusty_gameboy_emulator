# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A work-in-progress Nintendo Game Boy (DMG-01) emulator written in Rust. CPU core, memory bus, timer, and interrupts are implemented and passing Blargg CPU instruction test ROMs. PPU is scaffolded (VRAM + tile decode + LCD registers) but has no scanline timing or framebuffer yet. No cartridge/MBC, joypad, audio, or real-time rendering loop yet.

## Common commands

- Build: `cargo build`
- Run (headless, default): `cargo run` — runs `DEFAULT_TEST_ROM` in `src/main.rs`, or `cargo run -- <rom-path>` for a specific `.gb` file. Output is Game Boy serial output (test ROM PASS/FAIL text) printed to stdout, plus a Game Boy Doctor-format trace log written to `doctor_<rom_stem>.log` in the working directory.
- Run (display mode): `cargo run -- --display` — opens a `minifb` window at GB screen resolution (160x144, scaled 4x) showing a static test pattern (`src/display.rs`); not yet wired to real PPU/CPU framebuffer output.
- Test: `cargo test`
- Run a single test: `cargo test <test_name>` (e.g. `cargo test halt_bug_reexecutes_next_opcode`)
- Lint: `cargo clippy`

Tests are colocated with implementation in `#[cfg(test)] mod tests` blocks throughout `src/`, not in a separate `tests/` directory.

## Architecture

### Instruction pipeline

Two-layer split between instruction *representation* and instruction *execution*:

- `src/instructions/` — the `Instruction` enum (in `instructions/mod.rs`) and its operand-target types (`ArithmeticTarget`, `LoadType`/`LoadByteSource`/`LoadByteTarget`/etc., `StackTarget`, `IncDecTarget`, `PrefixTarget`, `JumpTest`). `instructions/decode/` (split by instruction family: `arithmetic.rs`, `arithmetic16.rs`, `control_flow.rs`, `incdec.rs`, `load.rs`, `prefix.rs`, `stack.rs`) turns raw opcode bytes into `Instruction` values. `Instruction::from_byte(byte, prefixed)` is the entry point; returns `None` for unimplemented/invalid opcodes.
- `src/cpu/` — the `CPU` struct and everything that executes a decoded `Instruction` against register/bus state. Split by concern (see `docs/cpu_modules.md`, though file names there are stale — the actual files are `execute.rs`, `control_flow.rs`, `stack_interrupts.rs`, `load.rs`, `arithmetic.rs`, `fetch_trace.rs`, `prefix.rs`, all under `src/cpu/`, with the `CPU` struct itself in `src/cpu/mod.rs`). Each submodule exposes a `*Ops` trait (`ArithmeticOps`, `ControlFlowOps`, `FetchTraceOps`, `LoadOps`, `PrefixOps`, `StackInterruptOps`) implemented on `CPU` and re-exported via `pub(crate) use` in `cpu/mod.rs`.

Visibility convention inside `src/cpu/`: prefer `pub(super)` for helpers shared across CPU submodules; keep helpers private if used in only one file; reserve `pub(crate)` for the external CPU API (`new`, `step`, `is_halted`, trace accessors).

### Behavioural invariants (do not change without explicit reason)

These are load-bearing for passing the Blargg/Mooneye test ROMs — preserve them exactly when touching `src/cpu/`:

- Interrupt service order: `IME` disable → push PC to stack → clear interrupt flag → jump to handler.
- HALT bug: when triggered, PC is decremented before executing the next opcode so the byte after `HALT` is fetched/executed twice.
- `EI` has delayed effect: `IME` becomes true only after the instruction *following* `EI` completes (`ei_pending` in `CPU`).
- Stack push order is high byte then low byte; pop is low byte then high byte.
- Per-instruction T-cycle counts and PC advancement must stay exact — the CPU is stepped and hardware (timer, PPU) is ticked once per T-cycle in the main loop (`src/main.rs`), so cycle counts drive timing-sensitive peripherals.

### Execution loop

`CPU::step()` (in `src/cpu/mod.rs`) does: service any pending interrupt or stay halted → fetch/decode → apply HALT-bug PC adjustment if needed → execute instruction and advance PC → commit any pending `EI`. It returns T-states consumed. Callers (currently `src/main.rs`) are expected to tick the timer and PPU that many individual T-cycles afterward — hardware ticking is per-T-cycle, not per-instruction, to keep timing accurate.

### Other modules

- `src/memory_bus.rs` — address decoding and read/write dispatch to RAM/VRAM/timer/interrupts/etc.
- `src/timer.rs` — DIV/TIMA/TMA/TAC, ticked per T-cycle, requests the Timer interrupt on overflow.
- `src/interrupts.rs` — interrupt enable/flag register handling and interrupt vector lookup.
- `src/ppu.rs` — VRAM storage, tile decoding, LCD register storage (no timing/rendering pipeline yet).
- `src/cartridge_header.rs` — parses the Game Boy cartridge header for logging/info.
- `src/register.rs` — 8/16-bit register model and flag register.
- `src/flag_helpers.rs` — shared flag-computation helpers used by arithmetic instructions.

## Notes

- Unknown/unimplemented opcodes panic on decode/execute rather than silently no-opping — this is intentional, to surface gaps during development.
- Emulation currently runs unthrottled (no real-time pacing to actual Game Boy clock speed).
