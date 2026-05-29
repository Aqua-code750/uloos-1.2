use crate::keyboard::inb;
use crate::mouse::outb;

pub fn set_vga_mode_320x200() {
    unsafe {
        // Low-level hardware-port sequence to switch standard VGA cards to Mode 0x13 (320x200 256-color)
        
        // 1. Miscellaneous Output Register
        outb(0x3C2, 0x63);

        // 2. Sequencer Registers
        let seq_regs = [0x03, 0x01, 0x0F, 0x00, 0x0E];
        for i in 0..5 {
            outb(0x3C4, i as u8);
            outb(0x3C5, seq_regs[i]);
        }

        // 3. CRT Controller Registers
        // Unlock CRTC registers 0-7 by clearing bit 7 of register 0x11
        outb(0x3D4, 0x11);
        let temp = inb(0x3D5);
        outb(0x3D4, 0x11);
        outb(0x3D5, temp & 0x7F);

        let crtc_regs = [
            0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
            0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x9C, 0x8E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3,
            0xFF
        ];
        for i in 0..25 {
            outb(0x3D4, i as u8);
            outb(0x3D5, crtc_regs[i]);
        }

        // 4. Graphics Controller Registers
        let gc_regs = [0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F, 0xFF];
        for i in 0..9 {
            outb(0x3CE, i as u8);
            outb(0x3CF, gc_regs[i]);
        }

        // 5. Attribute Controller Registers
        let ac_regs = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
            0x41, 0x00, 0x0F, 0x00, 0x00
        ];
        for i in 0..21 {
            let _ = inb(0x3DA); // Reset Attribute Controller address/data flip-flop
            outb(0x3C0, i as u8);
            outb(0x3C0, ac_regs[i]);
        }

        // 6. Enable video signal
        let _ = inb(0x3DA);
        outb(0x3C0, 0x20);
    }
}

