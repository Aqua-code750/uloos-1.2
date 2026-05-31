# UloOS: 100% Pure Nightly Rust Operating System

UloOS is a minimal, beginner-friendly Operating System written completely in **Nightly Rust** for x86_64 hardware. It boots on bare-metal and runs directly inside **QEMU** or the high-fidelity web simulator.

Everything—including the core systems, graphics drivers, mouse & keyboard controllers, shell interpreter, window/state buffers, office suite, file system, and interactive roguelike minigame—is implemented in pure, bare-metal Rust code.

---

## 🌟 What's New in Version 1.2

### 1. 🎮 Isle of Doom (Roguelike Dungeon Crawler)
A fully detailed crossover game modeled after *The Binding of Isaac* and styled like *DOOM*:
- **Room-to-Room Dungeon Crawler:** Stepping through cleared N/S/E/W doors generates random combat rooms, treasure vaults, or boss battles.
- **Heart Container HUD:** A beautiful health bar displaying pixel-art hearts (1 full heart = 2 HP, supporting half-hearts).
- **Upgrades & Pedestals:** Walk over pedestals in Treasure Rooms to collect game-changing items:
  - **`BFG-9000`**: Heavy damage, slow fire, high bullet speed.
  - **`Sad Onion`**: Massive firing speed upgrade.
  - **`Spoon Bender`**: Homing stats and speed boost.
- **Dynamic Crossover Enemies:** Battle tear-flying Cacodemons, crying Imps, and the ultimate boss: **Cyber-Monstro** (with its own boss health bar!).
- **Chests & Drops:** Slain monsters drop **Medkits** (+40 HP), **Ammo boxes** (+20 shells), **Coins**, **Bombs**, and **Keys**.
- **Retro PC Speaker Sounds:** Nostalgic square-wave tones playing on gunshot, bullet splashes, and victory tunes!

### 2. 🖥️ Premium Graphical UI & Login Lock
A gorgeous Windows-inspired desktop environment running in standard VGA Mode 13h (320x200 256-color mode):
- **Login Lock Screen:** A secure login page with user avatar, username input field, and dynamic status before the desktop opens.
- **Desktop Shortcuts & Start Menu:** Drag/click shortcuts for FILES, DOS, CODE, DOOM, STORE, and AI. A fully functional Start Menu with pinned apps and a "Restart Machine" option.
- **Taskbar Dock:** An elegant, rounded Windows 11-style dock showing active apps, clock widget, timezone offsets, and avatar.

### 3. 🔊 Retro PC Speaker Melodies
Authentic x86 PC speaker physical music chime:
- **Startup Sound:** A nostalgic ascending major-seventh chime triggered on successful login.
- **Shutdown Sound:** A descending hardware power-off melody when initiating shutdown.
- **Interactive Tones:** High-fidelity square-wave feedback on gunshot, enemy deaths, item pickups, and button clicks.

---

## 📂 Core Repository Files

- [src/main.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/main.rs): Kernel entry point, GUI desktop manager, mouse cursor coordinate loops, and keyboard event router.
- [src/apps/doom.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/apps/doom.rs): The complete *Isle of Doom* dungeon generator, teardrop collision system, upgrade pedestal logic, and enemy chasing AI.
- [src/sound.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/sound.rs): Hardware PC Speaker sound driver (ascending/descending chimes, square wave math).
- [src/allocator.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/allocator.rs): Lightweight BSS static-array memory allocator.
- [src/vga_driver.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/vga_driver.rs): Double-buffered VGA Mode 13h pixel drawer.
- [src/keyboard.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/keyboard.rs) & [src/mouse.rs](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/uloos-kernel/src/mouse.rs): Bare-metal PS/2 port handlers (`inb` / `outb`).

---

## 🛠️ Build and Run UloOS on QEMU

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

---

## 🌐 Web Simulator
You can also run the complete operating system mockup directly inside your web browser! Simply double-click [Launch_Web_Simulator.bat](file:///c:/Users/kavs1/OneDrive/Desktop/UloOS%20Minimal/Launch_Web_Simulator.bat) or open `index.html` to play with the in-browser simulator.
