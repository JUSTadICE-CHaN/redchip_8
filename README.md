<p align="center"> # RCHIP — CHIP-8 Emulator </p>

<p align="center">
  <strong>A CHIP-8 emulator written in Rust, featuring a graphical interface and real-time CPU state inspection.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-Systems%20Programming-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Architecture-CHIP--8-blueviolet" alt="CHIP-8">
  <img src="https://img.shields.io/badge/Status-Complete-green" alt="Status">
  <img src="https://img.shields.io/github/license/JUSTadICE-CHaN/redchip_8" alt="License">
</p>

---

## Overview

RCHIP is an implementation of the CHIP-8 virtual machine, written in Rust.

The project aims to explore low-level computer architecture and systems programming through the implementation of a virtual processor, its instruction set, memory, timers, display and input handling.

It includes a graphical frontend built with [Macroquad](https://github.com/not-fl3/macroquad) and [Ply](https://github.com/TheRedDeveloper/ply-engine). The interface provides real-time access to CPU state alongside a playable CHIP-8 display.

## Features

- **CHIP-8 CPU emulation** — Instruction fetching, decoding and execution.
- **Memory management** — 4 KB memory space, registers, stack and program loading.
- **Timers** — Delay and sound timer support.
- **Graphics** — 64 × 32 monochrome display.
- **Input handling** — CHIP-8 hexadecimal keypad mapped to a conventional keyboard.
- **Cycle-based execution** — Configurable CPU execution frequency.
- **Interactive interface** — Real-time display and CPU state inspection.
- **ROM management** — Load and reset ROMs through the graphical interface.
- **Error handling** — Execution errors are reported with instruction addresses and opcodes.

## Interface

The interface provides:

| Panel | Information |
|---|---|
| Display | Live CHIP-8 framebuffer |
| CPU State | Current ROM and execution status |
| Registers | General-purpose V0–VF registers |
| Special Registers | Program counter (PC) and index register (I) |
| Timers | Delay timer (DT) and sound timer (ST) |

## Controls

The CHIP-8 keypad is mapped to the following keyboard layout:

```text
CHIP-8          KEYBOARD

1  2  3  C      1  2  3  4
4  5  6  D      Q  W  E  R
7  8  9  E      A  S  D  F
A  0  B  F      Z  X  C  V
```

## Getting Started

### Requirements

- Rust toolchain (stable)
- Cargo
- A supported desktop operating system

### Build from source

Clone the repository:

```bash
git clone https://github.com/JUSTadICE-CHaN/rchip.git
cd rchip
```

Build the project:

```bash
cargo build --release
```

Run the emulator:

```bash
cargo run --release
```

### ROMs

RCHIP supports CHIP-8 ROM files (`.ch8`).

You can load ROMs through the graphical interface. The default configuration loads:

```text
roms/Pong (1 player).ch8
```

Additional ROMs can be selected using the **Load ROM** button.

Please ensure you have the appropriate rights to distribute any ROM files included with the project.

## Learning Objectives

This project explores several fundamental concepts in computer architecture and systems programming:

- Instruction set architecture (ISA) and opcode decoding.
- CPU registers, program counters and stack management.
- Memory addressing and instruction execution.
- Hardware timers and input handling.
- Cycle-based emulation.
- Modular software architecture and testing.

## License

See [LICENSE](LICENSE) for licensing information.

---

<p align="center">
  <strong>RCHIP</strong><br>
  <sub>Exploring computer architecture, one instruction at a time.</sub>
</p>
