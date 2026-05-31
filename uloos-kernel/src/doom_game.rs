// ==========================================
// UloOS DOOM Engine — doomgeneric Platform Layer
// ==========================================
// This module implements the doomgeneric platform interface that allows
// the real id Software DOOM engine (compiled from C) to run directly
// on the UloOS bare-metal kernel.
//
// doomgeneric requires these 6 functions to be implemented:
//   DG_Init(w, h)          — Initialize display
//   DG_DrawFrame()         — Blit framebuffer to screen
//   DG_SleepMs(ms)         — Sleep for N milliseconds
//   DG_GetTicksMs()        — Get elapsed time in ms
//   DG_GetKey(pressed,key) — Poll keyboard input
//   DG_SetWindowTitle(t)   — Set window title (no-op for us)
//
// Additionally, we provide all the libc stubs that DOOM's C code needs.

use crate::vga_driver::{VGA, SCREEN_WIDTH, SCREEN_HEIGHT, BACKBUFFER};
use crate::timer;
use crate::keyboard::inb;
use crate::mouse::outb;

// ==========================================
// DOOM Screen Configuration
// ==========================================
pub const DOOM_WIDTH: usize = 320;
pub const DOOM_HEIGHT: usize = 200;

/// The DOOM framebuffer — doomgeneric writes RGBA pixels here.
/// We convert them to VGA palette indices when blitting.
static mut DG_SCREEN_BUFFER: [u32; DOOM_WIDTH * DOOM_HEIGHT] = [0u32; DOOM_WIDTH * DOOM_HEIGHT];

/// Keyboard event queue for doomgeneric
const KEY_QUEUE_SIZE: usize = 32;
static mut KEY_QUEUE: [(i32, u8); KEY_QUEUE_SIZE] = [(0, 0); KEY_QUEUE_SIZE];
static mut KEY_QUEUE_HEAD: usize = 0;
static mut KEY_QUEUE_TAIL: usize = 0;

/// Track key states for press/release events
static mut KEY_STATES: [bool; 256] = [false; 256];

/// DOOM is currently active and running
pub static mut DOOM_RUNNING: bool = false;

// ==========================================
// doomgeneric Key Constants (from doomkeys.h)
// ==========================================
pub const KEY_RIGHTARROW: u8 = 0xae;
pub const KEY_LEFTARROW: u8 = 0xac;
pub const KEY_UPARROW: u8 = 0xad;
pub const KEY_DOWNARROW: u8 = 0xaf;
pub const KEY_FIRE: u8 = 0x80 + 0x43; // CTRL key
pub const KEY_USE: u8 = b' ';
pub const KEY_ENTER: u8 = 13;
pub const KEY_ESCAPE: u8 = 27;
pub const KEY_TAB: u8 = 9;
pub const KEY_RSHIFT: u8 = 0x80 + 0x36;
pub const KEY_LSHIFT: u8 = KEY_RSHIFT;
pub const KEY_RALT: u8 = 0x80 + 0x38;

// ==========================================
// doomgeneric Platform Functions (extern "C")
// ==========================================

/// Called by doomgeneric to initialize the display.
#[no_mangle]
pub unsafe extern "C" fn DG_Init() {
    // Set the DOOM-specific 256-color VGA palette
    crate::vga_mode::set_doom_vga_palette();
    DOOM_RUNNING = true;
}

/// Called by doomgeneric every frame to blit the screen.
/// DG_ScreenBuffer contains ARGB pixels. We convert to VGA palette indices.
#[no_mangle]
pub unsafe extern "C" fn DG_DrawFrame() {
    // Poll timer to keep ticks advancing
    timer::poll_pit_ticks();

    // Convert RGBA framebuffer → 8-bit palette-indexed VGA buffer
    let mut backbuffer = BACKBUFFER.lock();
    for i in 0..(DOOM_WIDTH * DOOM_HEIGHT) {
        let rgba = DG_SCREEN_BUFFER[i];
        // Extract RGB components (format: 0x00RRGGBB)
        let r = ((rgba >> 16) & 0xFF) as u8;
        let g = ((rgba >> 8) & 0xFF) as u8;
        let b = (rgba & 0xFF) as u8;
        backbuffer[i] = rgb_to_palette(r, g, b);
    }
    drop(backbuffer);

    // Copy to VGA hardware
    VGA.swap_buffers();
}

/// Called by doomgeneric to sleep for N milliseconds.
#[no_mangle]
pub unsafe extern "C" fn DG_SleepMs(ms: u32) {
    timer::delay_ms(ms);
}

/// Called by doomgeneric to get elapsed time in milliseconds.
#[no_mangle]
pub unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    timer::get_ticks_ms()
}

/// Called by doomgeneric to poll keyboard input.
/// Returns 1 if an event is available, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn DG_GetKey(pressed: *mut i32, doom_key: *mut u8) -> i32 {
    // First, poll the PS/2 keyboard and queue events
    poll_keyboard_for_doom();

    if KEY_QUEUE_HEAD != KEY_QUEUE_TAIL {
        let (p, k) = KEY_QUEUE[KEY_QUEUE_HEAD];
        KEY_QUEUE_HEAD = (KEY_QUEUE_HEAD + 1) % KEY_QUEUE_SIZE;
        *pressed = p;
        *doom_key = k;
        1
    } else {
        0
    }
}

/// Called by doomgeneric to set window title (no-op on bare metal)
#[no_mangle]
pub unsafe extern "C" fn DG_SetWindowTitle(_title: *const u8) {
    // No-op: we're on bare metal VGA
}

// ==========================================
// Keyboard Translation (PS/2 → DOOM keys)
// ==========================================

/// Enqueue a key event for doomgeneric
unsafe fn enqueue_key(pressed: bool, doom_key: u8) {
    let next_tail = (KEY_QUEUE_TAIL + 1) % KEY_QUEUE_SIZE;
    if next_tail != KEY_QUEUE_HEAD {
        KEY_QUEUE[KEY_QUEUE_TAIL] = (if pressed { 1 } else { 0 }, doom_key);
        KEY_QUEUE_TAIL = next_tail;
    }
}

/// Poll PS/2 keyboard and convert scancodes to DOOM key events
unsafe fn poll_keyboard_for_doom() {
    let status = inb(0x64);
    if (status & 1) == 0 { return; }
    if (status & 0x20) != 0 { return; } // Mouse data, skip

    let scancode = inb(0x60);
    let is_release = (scancode & 0x80) != 0;
    let code = scancode & 0x7F;

    // Convert PS/2 scancode to DOOM key
    let doom_key = match code {
        0x48 => KEY_UPARROW,    // Up arrow
        0x50 => KEY_DOWNARROW,  // Down arrow
        0x4B => KEY_LEFTARROW,  // Left arrow
        0x4D => KEY_RIGHTARROW, // Right arrow
        0x1D => KEY_FIRE,       // Left Ctrl = Fire
        0x39 => KEY_USE,        // Space = Use/Open
        0x1C => KEY_ENTER,      // Enter
        0x01 => KEY_ESCAPE,     // Escape
        0x0F => KEY_TAB,        // Tab (automap)
        0x2A | 0x36 => KEY_LSHIFT, // Shift = Run

        // WASD support (maps to arrows for movement)
        0x11 => KEY_UPARROW,    // W = Forward
        0x1F => KEY_DOWNARROW,  // S = Backward
        0x1E => KEY_LEFTARROW,  // A = Turn Left
        0x20 => KEY_RIGHTARROW, // D = Turn Right

        // Number keys for weapon selection
        0x02 => b'1', // 1
        0x03 => b'2', // 2
        0x04 => b'3', // 3
        0x05 => b'4', // 4
        0x06 => b'5', // 5
        0x07 => b'6', // 6
        0x08 => b'7', // 7

        // Y/N for prompts
        0x15 => b'y',
        0x31 => b'n',

        _ => 0,
    };

    if doom_key != 0 {
        let pressed = !is_release;
        // Only send event if state changed
        if KEY_STATES[doom_key as usize] != pressed {
            KEY_STATES[doom_key as usize] = pressed;
            enqueue_key(pressed, doom_key);
        }
    }
}

// ==========================================
// RGB → Palette Index Converter
// ==========================================
// Converts a 24-bit RGB value to the nearest VGA palette index.
// Uses a fast approximate method based on DOOM's known palette structure.

fn rgb_to_palette(r: u8, g: u8, b: u8) -> u8 {
    // The DOOM palette has 256 specific colors. We use a simple nearest-match
    // lookup against the known DOOM palette (PLAYPAL lump).
    // For performance, we use a simplified color cube mapping.

    // Quick black/near-black check
    if r < 8 && g < 8 && b < 8 {
        return 0; // Black
    }

    // Map to 6-bit VGA DAC values (0-63)
    let r6 = (r >> 2) as u16;
    let g6 = (g >> 2) as u16;
    let b6 = (b >> 2) as u16;

    // Search the DOOM palette for nearest color
    let mut best_idx: u8 = 0;
    let mut best_dist: u32 = u32::MAX;

    for i in 0..256u16 {
        let (pr, pg, pb) = DOOM_PALETTE_RGB[i as usize];
        let dr = (r6 as i32 - pr as i32);
        let dg = (g6 as i32 - pg as i32);
        let db = (b6 as i32 - pb as i32);
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
            if dist == 0 { break; }
        }
    }

    best_idx
}

// ==========================================
// Pointer to the screen buffer (for doomgeneric C code)
// ==========================================

/// Exported pointer to the screen buffer so doomgeneric C code can write to it.
/// doomgeneric expects: extern uint32_t* DG_ScreenBuffer;
extern "C" {
    pub static mut DG_ScreenBuffer: *mut u32;
}

/// Initialize the screen buffer pointer. Call once before launching DOOM.
pub unsafe fn init_doom_screen_buffer() {
    DG_ScreenBuffer = DG_SCREEN_BUFFER.as_mut_ptr();
}

// ==========================================
// Launch DOOM
// ==========================================

extern "C" {
    /// Main entry point of doomgeneric (compiled from C)
    fn doomgeneric_Create(argc: i32, argv: *mut *mut u8);
    fn doomgeneric_Tick();
}

/// Start the DOOM engine. This takes over the main loop.
pub unsafe fn run_doom() {
    // Initialize screen buffer pointer
    init_doom_screen_buffer();

    // Clear screen to black and reset print Y
    crate::vga_driver::VGA.draw_rect(0, 0, DOOM_WIDTH, DOOM_HEIGHT, 0);
    crate::vga_driver::VGA.swap_buffers();
    DOOM_PRINT_Y = 0;

    // Initialize PIT timer
    timer::init_pit();

    // Call doomgeneric initialization with no arguments
    let mut arg0 = *b"doom\0";
    let mut argv: [*mut u8; 1] = [arg0.as_mut_ptr()];
    doomgeneric_Create(1, argv.as_mut_ptr());

    // Main DOOM loop — runs until ESC exits
    DOOM_RUNNING = true;
    while DOOM_RUNNING {
        // Advance PIT ticks
        timer::poll_pit_ticks();

        // Run one DOOM frame
        doomgeneric_Tick();
    }

    // Restore desktop VGA palette when DOOM exits
    let theme = crate::SYSTEM_SETTINGS.lock().active_theme;
    crate::vga_mode::set_dynamic_vga_palette(theme);
}

// ==========================================
// DOOM Palette (PLAYPAL) — 256 colors in VGA DAC format (6-bit, 0-63)
// ==========================================
// This is DOOM's actual color palette from the PLAYPAL lump.
// Each entry is (R, G, B) in 6-bit VGA DAC format (0-63).

pub static DOOM_PALETTE_RGB: [(u8, u8, u8); 256] = [
    (0, 0, 0),       // 0: Black
    (31, 23, 11),     // 1: Dark brown
    (23, 15, 7),      // 2: Darker brown
    (30, 28, 25),     // 3: Gray
    (15, 15, 15),     // 4: Dark gray
    (18, 8, 0),       // 5: Dark red-brown
    (20, 4, 0),       // 6: Deep red
    (13, 0, 0),       // 7: Very dark red
    (25, 25, 25),     // 8: Medium gray
    (16, 16, 16),     // 9: Darker medium gray
    (8, 8, 8),        // 10: Very dark gray
    (4, 4, 4),        // 11: Near black gray
    (63, 63, 63),     // 12: White
    (55, 55, 55),     // 13: Light gray
    (47, 47, 47),     // 14: Gray
    (39, 39, 39),     // 15: Medium gray
    // Red/brown ramp (walls, blood)
    (63, 0, 0),       // 16: Bright red
    (59, 0, 0),       // 17
    (55, 0, 0),       // 18
    (51, 0, 0),       // 19
    (47, 0, 0),       // 20
    (43, 0, 0),       // 21
    (39, 0, 0),       // 22
    (35, 0, 0),       // 23
    (31, 0, 0),       // 24
    (27, 0, 0),       // 25
    (23, 0, 0),       // 26
    (19, 0, 0),       // 27
    (15, 0, 0),       // 28
    (11, 0, 0),       // 29
    (7, 0, 0),        // 30
    (3, 0, 0),        // 31
    // Brown/tan ramp (wood, doors)
    (47, 35, 11),     // 32
    (43, 31, 11),     // 33
    (39, 27, 11),     // 34
    (35, 23, 11),     // 35
    (31, 19, 7),      // 36
    (27, 15, 7),      // 37
    (23, 11, 7),      // 38
    (19, 7, 3),       // 39
    (55, 43, 23),     // 40
    (51, 39, 19),     // 41
    (47, 35, 15),     // 42
    (43, 27, 11),     // 43
    (39, 23, 7),      // 44
    (35, 19, 7),      // 45
    (27, 15, 3),      // 46
    (19, 7, 0),       // 47
    // Green ramp (armor, slime, tech)
    (0, 63, 0),       // 48: Bright green
    (0, 59, 0),       // 49
    (0, 55, 0),       // 50
    (0, 51, 0),       // 51
    (0, 47, 0),       // 52
    (0, 43, 0),       // 53
    (0, 39, 0),       // 54
    (0, 35, 0),       // 55
    (0, 31, 0),       // 56
    (0, 27, 0),       // 57
    (0, 23, 0),       // 58
    (0, 19, 0),       // 59
    (0, 15, 0),       // 60
    (0, 11, 0),       // 61
    (0, 7, 0),        // 62
    (0, 3, 0),        // 63
    // Blue ramp (sky, water)
    (0, 0, 63),       // 64: Bright blue
    (0, 0, 59),       // 65
    (0, 0, 55),       // 66
    (0, 0, 51),       // 67
    (0, 0, 47),       // 68
    (0, 0, 43),       // 69
    (0, 0, 39),       // 70
    (0, 0, 35),       // 71
    (0, 0, 31),       // 72
    (0, 0, 27),       // 73
    (0, 0, 23),       // 74
    (0, 0, 19),       // 75
    (0, 0, 15),       // 76
    (0, 0, 11),       // 77
    (0, 0, 7),        // 78
    (0, 0, 3),        // 79
    // Yellow/orange ramp (fire, explosions)
    (63, 63, 0),      // 80: Bright yellow
    (63, 59, 0),      // 81
    (63, 55, 0),      // 82
    (63, 51, 0),      // 83
    (63, 47, 0),      // 84
    (63, 43, 0),      // 85
    (63, 39, 0),      // 86
    (63, 35, 0),      // 87
    (63, 31, 0),      // 88
    (63, 27, 0),      // 89
    (63, 23, 0),      // 90: Orange
    (63, 19, 0),      // 91
    (63, 15, 0),      // 92
    (63, 11, 0),      // 93
    (63, 7, 0),       // 94
    (63, 3, 0),       // 95
    // Skin/flesh tones (enemies, status bar face)
    (47, 35, 23),     // 96
    (43, 31, 19),     // 97
    (39, 27, 15),     // 98
    (35, 23, 15),     // 99
    (31, 19, 11),     // 100
    (27, 15, 7),      // 101
    (23, 11, 3),      // 102
    (55, 43, 31),     // 103
    (51, 39, 27),     // 104
    (47, 35, 23),     // 105
    (43, 31, 19),     // 106
    (39, 27, 15),     // 107
    (35, 23, 11),     // 108
    (31, 19, 7),      // 109
    (27, 15, 3),      // 110
    (23, 11, 0),      // 111
    // Olive/dark green (tech, military)
    (23, 23, 0),      // 112
    (21, 21, 0),      // 113
    (19, 19, 0),      // 114
    (17, 17, 0),      // 115
    (15, 15, 0),      // 116
    (13, 13, 0),      // 117
    (11, 11, 0),      // 118
    (9, 9, 0),        // 119
    (7, 7, 0),        // 120
    (5, 5, 0),        // 121
    (3, 3, 0),        // 122
    (1, 1, 0),        // 123
    (27, 23, 0),      // 124
    (23, 19, 0),      // 125
    (19, 15, 0),      // 126
    (15, 11, 0),      // 127
    // Gray gradient (concrete, metal)
    (63, 63, 63),     // 128: White
    (59, 59, 59),     // 129
    (55, 55, 55),     // 130
    (51, 51, 51),     // 131
    (47, 47, 47),     // 132
    (43, 43, 43),     // 133
    (39, 39, 39),     // 134
    (35, 35, 35),     // 135
    (31, 31, 31),     // 136
    (27, 27, 27),     // 137
    (23, 23, 23),     // 138
    (19, 19, 19),     // 139
    (15, 15, 15),     // 140
    (11, 11, 11),     // 141
    (7, 7, 7),        // 142
    (3, 3, 3),        // 143
    // Maroon/purple tones
    (43, 0, 0),       // 144
    (39, 0, 0),       // 145
    (35, 0, 0),       // 146
    (31, 0, 7),       // 147
    (27, 0, 11),      // 148
    (23, 0, 15),      // 149
    (19, 0, 19),      // 150
    (15, 0, 23),      // 151
    (11, 0, 27),      // 152
    (7, 0, 31),       // 153
    (3, 0, 35),       // 154
    (0, 0, 39),       // 155
    (47, 0, 11),      // 156
    (43, 0, 7),       // 157
    (39, 0, 3),       // 158
    (35, 0, 0),       // 159
    // Beige/tan tones
    (63, 55, 43),     // 160
    (63, 51, 35),     // 161
    (63, 47, 27),     // 162
    (63, 43, 19),     // 163
    (63, 39, 11),     // 164
    (63, 35, 3),      // 165
    (59, 31, 0),      // 166
    (55, 27, 0),      // 167
    (51, 23, 0),      // 168
    (47, 19, 0),      // 169
    (43, 15, 0),      // 170
    (39, 11, 0),      // 171
    (35, 7, 0),       // 172
    (31, 3, 0),       // 173
    (27, 0, 0),       // 174
    (23, 0, 0),       // 175
    // Teal/cyan tones
    (0, 47, 47),      // 176
    (0, 43, 43),      // 177
    (0, 39, 39),      // 178
    (0, 35, 35),      // 179
    (0, 31, 31),      // 180
    (0, 27, 27),      // 181
    (0, 23, 23),      // 182
    (0, 19, 19),      // 183
    (0, 15, 15),      // 184
    (0, 11, 11),      // 185
    (0, 7, 7),        // 186
    (0, 3, 3),        // 187
    (51, 51, 63),     // 188
    (47, 47, 59),     // 189
    (43, 43, 55),     // 190
    (39, 39, 51),     // 191
    // Purple/magenta
    (35, 35, 47),     // 192
    (31, 31, 43),     // 193
    (27, 27, 39),     // 194
    (23, 23, 35),     // 195
    (19, 19, 31),     // 196
    (15, 15, 27),     // 197
    (11, 11, 23),     // 198
    (7, 7, 19),       // 199
    (63, 43, 27),     // 200
    (59, 39, 23),     // 201
    (55, 35, 19),     // 202
    (51, 31, 15),     // 203
    (47, 27, 11),     // 204
    (43, 23, 7),      // 205
    (39, 19, 3),      // 206
    (35, 15, 0),      // 207
    // Peach/light skin
    (63, 47, 35),     // 208
    (63, 43, 27),     // 209
    (59, 39, 23),     // 210
    (55, 35, 19),     // 211
    (51, 31, 15),     // 212
    (47, 27, 11),     // 213
    (43, 23, 7),      // 214
    (39, 19, 3),      // 215
    (55, 55, 35),     // 216
    (51, 51, 31),     // 217
    (47, 47, 27),     // 218
    (43, 43, 23),     // 219
    (39, 39, 19),     // 220
    (35, 35, 15),     // 221
    (31, 31, 11),     // 222
    (27, 27, 7),      // 223
    // Pink/hot tones
    (63, 23, 23),     // 224
    (59, 19, 19),     // 225
    (55, 15, 15),     // 226
    (51, 11, 11),     // 227
    (47, 7, 7),       // 228
    (43, 3, 3),       // 229
    (39, 0, 0),       // 230
    (35, 0, 0),       // 231
    (63, 47, 47),     // 232
    (63, 39, 39),     // 233
    (63, 31, 31),     // 234
    (63, 23, 23),     // 235
    (63, 15, 15),     // 236
    (63, 7, 7),       // 237
    (63, 0, 0),       // 238
    (42, 0, 0),       // 239
    // Special colors (status bar, HUD)
    (63, 63, 47),     // 240
    (63, 63, 35),     // 241
    (63, 63, 23),     // 242
    (63, 63, 11),     // 243
    (63, 63, 0),      // 244
    (55, 55, 0),      // 245
    (47, 47, 0),      // 246
    (39, 39, 0),      // 247
    (31, 31, 0),      // 248
    (23, 23, 0),      // 249
    (15, 15, 0),      // 250
    (7, 7, 0),        // 251
    (63, 0, 63),      // 252: Magenta (pain flash)
    (63, 0, 47),      // 253
    (63, 0, 31),      // 254
    (63, 0, 15),      // 255
];

// ==========================================
// libc Stubs for doomgeneric C code
// ==========================================
// DOOM's C code calls these standard C library functions.
// We provide bare-metal implementations.

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    core::ptr::copy_nonoverlapping(src, dst, n);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    core::ptr::copy(src, dst, n);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, val: i32, n: usize) -> *mut u8 {
    core::ptr::write_bytes(dst, val as u8, n);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a as i32 - b as i32; }
        if a == 0 { return 0; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a as i32 - b as i32; }
        if a == 0 { return 0; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strncasecmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = to_lower(*s1.add(i));
        let b = to_lower(*s2.add(i));
        if a != b { return a as i32 - b as i32; }
        if a == 0 { return 0; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let a = to_lower(*s1.add(i));
        let b = to_lower(*s2.add(i));
        if a != b { return a as i32 - b as i32; }
        if a == 0 { return 0; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcat(dst: *mut u8, src: *const u8) -> *mut u8 {
    let dst_len = strlen(dst);
    strcpy(dst.add(dst_len), src);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const u8, c: i32) -> *const u8 {
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == c as u8 { return s.add(i); }
        if ch == 0 { return core::ptr::null(); }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const u8, c: i32) -> *const u8 {
    let mut last: *const u8 = core::ptr::null();
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == c as u8 { last = s.add(i); }
        if ch == 0 { return last; }
        i += 1;
    }
}

fn to_lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' { c + 32 } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn toupper(c: i32) -> i32 {
    if c >= 'a' as i32 && c <= 'z' as i32 { c - 32 } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn tolower(c: i32) -> i32 {
    if c >= 'A' as i32 && c <= 'Z' as i32 { c + 32 } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn atoi(s: *const u8) -> i32 {
    let mut result: i32 = 0;
    let mut i = 0;
    let mut negative = false;

    // Skip whitespace
    while *s.add(i) == b' ' || *s.add(i) == b'\t' { i += 1; }

    // Check sign
    if *s.add(i) == b'-' { negative = true; i += 1; }
    else if *s.add(i) == b'+' { i += 1; }

    // Parse digits
    while *s.add(i) >= b'0' && *s.add(i) <= b'9' {
        result = result * 10 + (*s.add(i) - b'0') as i32;
        i += 1;
    }

    if negative { -result } else { result }
}

#[no_mangle]
pub unsafe extern "C" fn atol(s: *const u8) -> i64 {
    atoi(s) as i64
}

#[no_mangle]
pub unsafe extern "C" fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

static mut DOOM_PRINT_Y: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn printf(fmt: *const u8, _: ...) -> i32 {
    puts(fmt);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fprintf(_file: *mut u8, _fmt: *const u8, _: ...) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn sprintf(buf: *mut u8, _fmt: *const u8, _: ...) -> i32 {
    *buf = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn snprintf(buf: *mut u8, _n: usize, _fmt: *const u8, _: ...) -> i32 {
    if !buf.is_null() { *buf = 0; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(buf: *mut u8, _n: usize, _fmt: *const u8, _ap: *mut u8) -> i32 {
    if !buf.is_null() { *buf = 0; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sscanf(_buf: *const u8, _fmt: *const u8, _: ...) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const u8) -> i32 {
    let mut len = 0;
    while *s.add(len) != 0 { len += 1; }
    
    // Convert C string to Rust string
    if let Ok(st) = core::str::from_utf8(core::slice::from_raw_parts(s, len)) {
        // Strip newlines for cleaner rendering
        let clean = st.trim_end_matches('\n').trim_end_matches('\r');
        if clean.is_empty() { return 0; }

        if DOOM_PRINT_Y >= 190 {
            // Screen full, clear it
            VGA.draw_rect(0, 0, DOOM_WIDTH, DOOM_HEIGHT, 0);
            DOOM_PRINT_Y = 0;
        }
        
        // Draw the text in green (10) to look like a terminal
        VGA.draw_string(2, DOOM_PRINT_Y, clean, 10);
        VGA.swap_buffers();
        DOOM_PRINT_Y += 10;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fputc(_c: i32, _stream: *mut u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn vfprintf(_stream: *mut u8, _fmt: *const u8, _ap: *mut u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn system(_command: *const u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn mkdir(_path: *const u8, _mode: u32) -> i32 { -1 }

#[no_mangle]
pub unsafe extern "C" fn strdup(s: *const u8) -> *mut u8 {
    let mut len = 0;
    while *s.add(len) != 0 { len += 1; }
    let layout = core::alloc::Layout::from_size_align(len + 1, 1).unwrap();
    let ptr = alloc::alloc::alloc(layout);
    if !ptr.is_null() {
        core::ptr::copy_nonoverlapping(s, ptr, len + 1);
    }
    ptr
}

// Missing DOOM specific symbols that were not compiled or defined
#[no_mangle]
pub static mut drone: i32 = 0;

#[no_mangle]
pub static mut net_client_connected: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn W_ParseCommandLine() {}

// File I/O stubs — DOOM WAD loading needs to be redirected to in-memory buffer
// doomgeneric already handles this via its WAD embedding, but we stub the rest.
#[no_mangle]
pub unsafe extern "C" fn fopen(_path: *const u8, _mode: *const u8) -> *mut u8 {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn fclose(_file: *mut u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn fread(_buf: *mut u8, _size: usize, _count: usize, _file: *mut u8) -> usize { 0 }

#[no_mangle]
pub unsafe extern "C" fn fwrite(_buf: *const u8, _size: usize, _count: usize, _file: *mut u8) -> usize { 0 }

#[no_mangle]
pub unsafe extern "C" fn fseek(_file: *mut u8, _offset: i64, _whence: i32) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn ftell(_file: *mut u8) -> i64 { 0 }

#[no_mangle]
pub unsafe extern "C" fn fflush(_file: *mut u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn feof(_file: *mut u8) -> i32 { 1 }

#[no_mangle]
pub unsafe extern "C" fn ferror(_file: *mut u8) -> i32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn fgets(_buf: *mut u8, _n: i32, _file: *mut u8) -> *mut u8 {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn remove(_path: *const u8) -> i32 { -1 }

#[no_mangle]
pub unsafe extern "C" fn rename(_old: *const u8, _new: *const u8) -> i32 { -1 }

// Standard global variables that C code expects
#[no_mangle]
pub static mut stdin: *mut u8 = core::ptr::null_mut();
#[no_mangle]
pub static mut stdout: *mut u8 = core::ptr::null_mut();
#[no_mangle]
pub static mut stderr: *mut u8 = core::ptr::null_mut();

// Process control stubs
#[no_mangle]
pub unsafe extern "C" fn exit(_code: i32) -> ! {
    // Exit DOOM gracefully - set flag and loop forever
    // The main kernel loop should check DOOM_RUNNING
    DOOM_RUNNING = false;
    loop {
        core::arch::asm!("hlt");
    }
}

#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    exit(1);
}

#[no_mangle]
pub unsafe extern "C" fn atexit(_func: *const u8) -> i32 { 0 }

// Math stubs (DOOM uses fixed-point, but some utility code uses these)
#[no_mangle]
pub unsafe extern "C" fn floor(x: f64) -> f64 { x as i64 as f64 }

#[no_mangle]
pub unsafe extern "C" fn ceil(x: f64) -> f64 {
    let i = x as i64;
    if x > i as f64 { (i + 1) as f64 } else { i as f64 }
}

#[no_mangle]
pub unsafe extern "C" fn sqrt(x: f64) -> f64 {
    // Newton's method approximation
    if x <= 0.0 { return 0.0; }
    let mut guess = x;
    for _ in 0..20 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
}

#[no_mangle]
pub unsafe extern "C" fn sin(x: f64) -> f64 {
    // Taylor series
    let pi = 3.14159265358979323846;
    let mut v = x;
    while v > pi { v -= 2.0 * pi; }
    while v < -pi { v += 2.0 * pi; }
    let x2 = v * v;
    v - (v * x2 / 6.0) + (v * x2 * x2 / 120.0) - (v * x2 * x2 * x2 / 5040.0)
}

#[no_mangle]
pub unsafe extern "C" fn cos(x: f64) -> f64 {
    sin(x + 1.5707963267948966)
}

// Time stubs
#[no_mangle]
pub unsafe extern "C" fn time(_t: *mut i64) -> i64 {
    timer::get_ticks_ms() as i64
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> i64 {
    timer::get_ticks_ms() as i64
}

// Misc stubs
#[no_mangle]
pub unsafe extern "C" fn qsort(
    base: *mut u8,
    num: usize,
    size: usize,
    compar: unsafe extern "C" fn(*const u8, *const u8) -> i32,
) {
    // Simple bubble sort (not fast but correct and simple)
    if num <= 1 { return; }
    let mut swapped = true;
    while swapped {
        swapped = false;
        for i in 0..(num - 1) {
            let a = base.add(i * size);
            let b = base.add((i + 1) * size);
            if compar(a, b) > 0 {
                // Swap elements
                for j in 0..size {
                    let tmp = *a.add(j);
                    *a.add(j) = *b.add(j);
                    *b.add(j) = tmp;
                }
                swapped = true;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rand() -> i32 {
    // Simple LCG random number generator
    static mut SEED: u32 = 12345;
    SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
    ((SEED >> 16) & 0x7FFF) as i32
}

#[no_mangle]
pub unsafe extern "C" fn srand(seed: u32) {
    // Set random seed - ignored for simplicity
    let _ = seed;
}

#[no_mangle]
pub unsafe extern "C" fn getenv(_name: *const u8) -> *mut u8 {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strerror(_errnum: i32) -> *const u8 {
    b"Unknown error\0".as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn signal(_sig: i32, _handler: *const u8) -> *const u8 {
    core::ptr::null()
}

// errno (global error number used by C code)
#[no_mangle]
pub static mut errno: i32 = 0;

// __stack_chk_fail (stack smashing protection — disabled but symbol needed)
#[no_mangle]
pub unsafe extern "C" fn __stack_chk_fail() {
    panic!("Stack smashing detected!");
}
