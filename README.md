# UloOS: 100% Pure Nightly Rust Operating System

UloOS is a minimal, beginner-friendly Operating System written completely in **Nightly Rust** for x86_64 hardware. It boots on bare-metal and runs directly inside **QEMU**.

There is **HTML for the ui**. Everything—including the core systems, keyboard drivers, shell interpreter, window/state buffers, office suite, file system, and interactive DOOM minigame—is implemented in pure, bare-metal Rust code.

---

## System Architecture

- **Kernel Entry**: Bootstrapped via standard VGA memory maps (`0xb8000`), disabling standard libraries (`#![no_std]`, `#![no_main]`).
- **Hardware polling keyboard driver**: Decodes keyboard signals using core inline assembly (`inb` instructions on port `0x60` and `0x64`).
- **Taskbar with `(ToT)` Start Button**: Draws a permanent status dock at the bottom of the screen showing the `(ToT)` start logo and quick hotkeys (`F1`-`F8`).
- **Simulated Filesystem**: Integrated file records readable dynamically in-memory.

---

## Fully Integrated TUI Applications

You can switch between applications instantly by pressing keys **F1 through F8**:

1. **F1: Bash Shell** (`apps/bash.rs`)
   - An interactive command prompt. Type characters, use backspace, and execute shell instructions.
2. **F2: File Explorer** (`apps/explorer.rs`)
   - View system configuration and documentation files inside the filesystem. Navigate down/up by pressing **Enter**.
3. **F3: UloText Editor** (`apps/office.rs`)
   - Interactive document editor. Type your notes directly on the screen; use backspace to delete.
4. **F4: UloSlides Creator** (`apps/office.rs`)
   - Minimal slide presentation viewer. Press **Enter** to transition through slides.
5. **F5: UloNumbers Sheet** (`apps/office.rs`)
   - An interactive cell table grid. Navigate between cells using keys **W, A, S, D**. Press **+** to add 10 to a cell, and **-** to subtract 10.
6. **F6: UloMail Client** (`apps/office.rs`)
   - TUI email reader. Press **Enter** to switch between active emails.
7. **F7: UloBrowser Client** (`apps/explorer.rs`)
   - Text web browser showing standard page previews.
8. **F8: TUI DOOM Game** (`apps/doom.rs`)
   - A fully playable interactive retro arcade minigame!
   - Move your character `(ToT)` around the room using keys **W, A, S, D**.
   - Shoot monsters (`E`) in your line of sight by pressing **Space**.
   - Track health, ammunition count, and high scores.
*Note: The F1 - F8 key features is discontinued and will not be added in this version*
---

## How to Build and Run UloOS on QEMU

1. Install [Rustup (Nightly toolchain)](https://rustup.rs/) and [QEMU](https://www.qemu.org/) on your machine.
2. Open a terminal inside the project directory:
   ```bash
   cd uloos-kernel
   ```
3. Install the compilation boot utility:
   ```bash
   cargo install bootimage
   ```
4. Build and boot the entire operating system in QEMU:
   ```bash
   cargo run
   ```
   *Cargo compiles the kernel and runs the bootable ISO directly inside QEMU!*


**RELEASING ON 29TH OR 30TH WHEREVER U ARE! ON THAT DATE IS RELEASED**
*Note: also if you cloned the repo then u can do the steps above the text above this note*
