# Rusty Game Boy Emulator

A Game Boy emulator written in Rust, currently in early development.

## 🎯 Project Goals

- Build a cycle-accurate emulator for the original Game Boy (DMG-01)
- Run Blargg’s CPU test ROMs for validation
- Learn systems programming, emulation, and low-level architecture
- Eventually add Game Boy Color support

## 🚧 Current Status

**Early Development:** The Majority of CPU operations implemented, basic memory bus, basic timer functions, no PPU/APU yet.

### ✅ Implemented Features

- **CPU Core**: 8-bit registers, program counter, stack pointer
- **Instruction Execution**:
  - Arithmetic & logic (register forms): `ADD`, `ADC`, `SUB`, `SBC`, `AND`, `XOR`, `OR`, `CP`
  - 16-bit arithmetic and related: `ADD HL, rr` (0x09 / 0x19 / 0x29 / 0x39), `ADD SP, r8` (0xE8), `LD HL, SP+r8` (0xF8), `LD SP, HL` (0xF9)
  - Control flow & miscellaneous: `NOP`, `HALT`, `DI`, `EI`, `RLCA`, `RRCA`, `RLA`, `RRA`, `DAA`, `CPL`, `SCF`, `CCF`, `JP` (including `JP (HL)`), `JR`, `CALL`, `RET`, `RETI`, `RST`
  - Load operations: register-to-register `LD r, r'` (0x40-0x7F excluding 0x76), immediate byte/word loads `LD r,d8` and `LD rr,d16`, `LD A,(HL+)`, `LD A,(HL-)`, `LD (DE),A`, `LD (a16),A`, `LD A,(a16)`, `LDH (a8),A`, `LDH A,(a8)`
  - Increment / Decrement: `INC` / `DEC` for 8-bit and 16-bit targets
  - Stack operations: `PUSH` / `POP` for BC/DE/HL/AF
  - CB-prefixed instructions: full CB set implemented - `RLC`, `RRC`, `RL`, `RR`, `SLA`, `SRA`, `SWAP`, `SRL`, `BIT`, `RES`, `SET` (all targets, including HLI)
- **Memory Bus**:
  - Basic address decoding and read/write support
  - Internal RAM and basic VRAM access

### 🔄 In Progress

- Expanding the instruction set to support Blargg test ROMs
- Memory bus improvements for full address space coverage
- ROM loading functionality
- Timer implementation

### 📋 TODO

- [ ] Complete instruction set
- [ ] Implement the Interrupt Enable/Flag system
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
