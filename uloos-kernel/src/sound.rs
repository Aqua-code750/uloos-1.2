use crate::keyboard::inb;
use crate::mouse::outb;

// Plays a sound on the x86 PC Speaker at a given frequency (Hz)
pub unsafe fn play_tone(frequency: u32) {
    if frequency == 0 {
        stop_speaker();
        return;
    }

    // Calculate PIT divisor
    let divisor = 1193180 / frequency;

    // Set PIT channel 2 to Mode 3 (Square Wave Generator)
    outb(0x43, 0xB6);

    // Send divisor bytes (low byte then high byte)
    outb(0x42, (divisor & 0xFF) as u8);
    outb(0x42, ((divisor >> 8) & 0xFF) as u8);

    // Enable PC Speaker output
    let current_state = inb(0x61);
    if current_state & 3 != 3 {
        outb(0x61, current_state | 3);
    }
}

// Stops the PC Speaker from playing sound
pub unsafe fn stop_speaker() {
    let current_state = inb(0x61);
    outb(0x61, current_state & 0xFC);
}

// Nostalgic retro startup sound: an ascending high-fidelity major-seventh chime
pub fn play_startup_sound() {
    unsafe {
        // C5 (523 Hz)
        play_tone(523);
        delay_ms(150);

        // E5 (659 Hz)
        play_tone(659);
        delay_ms(150);

        // G5 (784 Hz)
        play_tone(784);
        delay_ms(150);

        // B5 (988 Hz)
        play_tone(988);
        delay_ms(150);

        // C6 (1047 Hz)
        play_tone(1047);
        delay_ms(350);

        stop_speaker();
    }
}

// Simple delay function using nop loops inside standard virtual machine speeds
fn delay_ms(ms: u32) {
    // 3,000,000 nops is approx 1 second.
    // 3,000 nops is approx 1 millisecond.
    let limit = ms * 3_000;
    for _ in 0..limit {
        unsafe { core::arch::asm!("nop") }
    }
}
