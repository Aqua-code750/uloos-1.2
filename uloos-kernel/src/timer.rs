// ==========================================
// UloOS PIT Timer Driver for DOOM Engine
// ==========================================
// Uses the Programmable Interval Timer (8254 PIT) channel 0 to provide
// millisecond-accurate timing for the DOOM game engine.

use crate::keyboard::inb;
use crate::mouse::outb;
use spin::Mutex;

/// Global tick counter — incremented by PIT interrupt or polling
static TICK_COUNT: Mutex<u64> = Mutex::new(0);

/// PIT oscillator base frequency
const PIT_FREQUENCY: u32 = 1_193_182;

/// Target tick frequency (1000 Hz = 1ms per tick)
const TARGET_HZ: u32 = 1000;

/// Initialize PIT Channel 0 to fire at ~1000 Hz for millisecond timing.
/// NOTE: Without IDT/interrupts, we poll the PIT status instead.
pub fn init_pit() {
    unsafe {
        let divisor = PIT_FREQUENCY / TARGET_HZ; // ~1193

        // Channel 0, Access: lobyte/hibyte, Mode 2 (rate generator)
        outb(0x43, 0x34);

        // Send divisor low byte then high byte
        outb(0x40, (divisor & 0xFF) as u8);
        outb(0x40, ((divisor >> 8) & 0xFF) as u8);
    }
}

/// Read the current PIT channel 0 count value.
/// Returns the current countdown value (counts DOWN from the divisor).
pub fn pit_read_count() -> u16 {
    unsafe {
        // Latch channel 0 count
        outb(0x43, 0x00);
        let lo = inb(0x40) as u16;
        let hi = inb(0x40) as u16;
        (hi << 8) | lo
    }
}

/// Poll-based millisecond counter using PIT.
/// Call this regularly from the main loop to advance the tick counter.
/// Returns true if at least one tick elapsed since last call.
pub fn poll_pit_ticks() -> bool {
    // We use a simple approach: read the PIT output pin status
    // via port 0x61 bit 5, which toggles each time channel 0 wraps.
    static mut LAST_STATE: bool = false;
    let ticked;
    unsafe {
        let status = inb(0x61);
        let current_state = (status & 0x20) != 0;
        ticked = current_state != LAST_STATE;
        if ticked {
            LAST_STATE = current_state;
            let mut count = TICK_COUNT.lock();
            *count += 1;
        }
    }
    ticked
}

/// Get the current tick count in milliseconds (approximate).
/// Each tick is ~1ms with our PIT configuration.
pub fn get_ticks_ms() -> u32 {
    *TICK_COUNT.lock() as u32
}

/// Busy-wait delay for a given number of milliseconds.
pub fn delay_ms(ms: u32) {
    let start = get_ticks_ms();
    while get_ticks_ms().wrapping_sub(start) < ms {
        poll_pit_ticks();
    }
}
