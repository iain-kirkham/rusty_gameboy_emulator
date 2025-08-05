# Rusty Game Boy Emulator

A Game Boy emulator written in Rust, currently in early development.

## 🎯 Project Goals

- Build a cycle-accurate emulator for the original Game Boy (DMG-01)
- Run Blargg’s CPU test ROMs for validation
- Learn systems programming, emulation, and low-level architecture
- Eventually add Game Boy Color support

## 🚧 Current Status

**Early Development:** Core CPU operations and memory mapping implemented

### ✅ Implemented Features

- **CPU Core**: 8-bit registers, program counter, stack pointer
- **Instruction Execution**:
  - Arithmetic: `ADD`, `SUB`, `INC`, `DEC`
  - Control flow: `JP`, `CALL`, `RET`, `HALT`, `NOP`
  - Stack: `PUSH`, `POP`
  - Partial load instructions (`LD`)
- **Memory Bus**:
  - Basic address decoding and read/write support
  - Internal RAM and basic VRAM access

### 🔄 In Progress

- Expanding the instruction set to support Blargg test ROMs
- Memory bus improvements for full address space coverage
- ROM loading functionality

### 📋 TODO

- [ ] Complete instruction set
- [ ]  Implement the Interrupt Enable/Flag system
- [ ] Timer implementation
- [ ] Add PPU (Picture Processing Unit) for graphics
- [ ] Add Audio Processing Unit (APU)
- [ ] Joypad input via keyboard
- [ ] ROM loading and cartridge support
- [ ] Save state functionality


## 📚 Reference Material
### Game Boy Documentation
- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive Game Boy technical reference
- [Interactive Game Boy Opcode Table](https://meganesu.github.io/generate-gb-opcodes/)
- [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
- [Real boy Emulator blog](https://realboyemulator.wordpress.com/)
- [ASMSchool lessons](http://gameboy.mongenel.com/asmschool.html)

### Test ROMs
- [Blargg's Test ROMs](https://github.com/retrio/gb-test-roms)
- [Mooneye Test Suite](https://github.com/Gekkio/mooneye-test-suite)
