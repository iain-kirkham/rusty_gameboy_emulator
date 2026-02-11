# Rusty Game Boy Emulator

A work-in-progress Nintendo **Game Boy (DMG-01)** emulator written in Rust.

This project is currently has basic, **CPU + memory bus + timer + interrupts** foundation in place and is now focused on developing the PPU and cartridge support.

## Current Status

### Implemented / Working
- **CPU core**
  - 8-bit registers, `PC`, `SP`, flags, stack operations
  - The base instruction set has been implemented
  - **CB-prefixed instruction set** implemented (rotates/shifts/bit ops/`SWAP`, including `(HL)` variants)
  - Control flow: `JP`, `JR`, `CALL`, `RET`, `RETI`, `RST`, `HALT`, `DI`/`EI` (with EI-delay handling)
- **Memory bus**
  - Address-decoding scaffolding and basic read/write
  - Internal RAM handling
  - VRAM access support at the bus level (not fully implemented PPU yet)
- **Timer**
  - Timer ticking integrated into the main execution loop (ticks per T-cycle)
  - Timer interrupt request on overflow
- **Interrupt system**
  - Interrupt enable/flag management and interrupt handling in the CPU step

### Partially Implemented
- **PPU/GPU module exists** (VRAM + tile decoding + LCD register storage), but:
  - No scanline timing/state machine yet
  - No framebuffer composition
  - No window/background/sprite rendering pipeline
  - Not currently wired into a real-time renderer loop

### Not Implemented Yet
- **Cartridge / MBC**
  - ROM loading exists for local test ROM execution, but full MBC support is not complete
- **Real-time rendering loop**
  - `winit` + `pixels` are added as dependencies, but rendering is not hooked up
- **Joypad input**
- **Save states**
- **APU/audio** (Low Priority)
- **Boot ROM behaviour / full hardware accuracy** 

## Running

### Prerequisites
- Rust toolchain (stable) installed via `rustup`

### Run the current tests (Blargg CPU instruction tests)
The current `main` is set up to run a Blargg CPU test ROM and print serial output emitted by the ROM.

1. Ensure test ROMs exist at the expected path:
   - `blargg/cpu_instrs/individual/01-special.gb`

2. Run:
   - `cargo run`

The emulator loop:
- steps the CPU,
- ticks the timer **per T-cycle**, and
- prints **serial output** as soon as it appears (used by test ROMs to report PASS/FAIL).

> If you want to run a different ROM, edit the `test_roms` list in `src/main.rs`.

## Project Layout (high level)

- `src/main.rs` - current entry point / test runner loop (CPU stepping + timer ticking + serial output)
- `src/cpu.rs` - CPU implementation and instruction execution
- `src/insturctions` - instruction model defines the decoded instructions
- `src/instructions/decode` - decoding all instructions for the CPU to execute
- `src/memory_bus.rs` - bus and address mapping
- `src/timer.rs` - DIV/TIMA/TMA/TAC timer logic
- `src/interrupts.rs` - interrupt controller
- `src/ppu.rs` - early GPU/PPU scaffolding (VRAM + tile decoding + LCD registers)
- `src/instructions/` - instruction decoding/implementation details

## Roadmap

### Graphics (PPU)
- Implement PPU timing/state machine (OAM Search / Pixel Transfer / HBlank / VBlank)
- Correct LY/STAT behaviour and LCD interrupts
- Produce a framebuffer and connect it to `pixels` + `winit`

### Longer-term
- Implement proper **MBC and cartridge support** (MBC3 is the priority)
- DMA behavior and timing
- Joypad input mapping
- Audio (APU)
- Save states
- Game Boy Color (CGB) support (after DMG baseline is solid)

## Notes
- This is not yet a playable emulator. It’s currently a cpu focused core with a test ROM runner.
- The CPU timing is not currently real world and will run at unlimited speed.
- Expect behavior differences vs hardware in unimplemented areas (PPU/APU/MBC/DMA).

## References

### Documentation
- Pan Docs: https://gbdev.io/pandocs/
- Opcode table: https://meganesu.github.io/generate-gb-opcodes/
- Game Boy: Complete Technical Reference (PDF): https://gekkio.fi/files/gb-docs/gbctr.pdf
- RGBDS :https://rgbds.gbdev.io/docs/v1.0.1/gbz80.7
- Gameboy development community: https://gbdev.io/
- Realboy emulator blog: https://realboyemulator.wordpress.com/
- ASMSchool lessons: http://gameboy.mongenel.com/asmschool.html

### Test ROMs
- Blargg GB test ROMs: https://github.com/retrio/gb-test-roms
- Mooneye test suite: https://github.com/Gekkio/mooneye-test-suite
