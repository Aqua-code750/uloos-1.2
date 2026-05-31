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

        // 7. Load premium modern-era color palette
        set_dynamic_vga_palette(0);
    }
}

pub fn set_dynamic_vga_palette(theme: usize) {
    unsafe {
        let custom_palette: [(u8, u8, u8, u8); 16] = match theme {
            1 => [ // 1: Cyber Purple (Neon Violet)
                (0, 8, 6, 12),     // 0: Navy Black
                (1, 15, 20, 40),   // 1: Indigo Blue
                (2, 12, 45, 24),   // 2: Emerald Green
                (3, 10, 35, 45),   // 3: Dark Teal
                (4, 48, 15, 18),   // 4: Soft Coral Red
                (5, 45, 10, 60),   // 5: Premium Lavender / Purple
                (6, 40, 25, 12),   // 6: Warm Copper Brown
                (7, 50, 50, 52),   // 7: Sleek Silver Gray
                (8, 16, 14, 22),   // 8: Medium Purple-Gray
                (9, 45, 10, 60),   // 9: Bright Purple Accent
                (10, 18, 56, 30),  // 10: Neon Mint Green
                (11, 48, 22, 58),  // 11: Elegant Neon Violet
                (12, 58, 20, 24),  // 12: Vibrant Rose Red
                (13, 48, 22, 58),  // 13: Elegant Neon Violet
                (14, 63, 48, 12),  // 14: Amber Gold
                (15, 61, 61, 63),  // 15: Crisp Pure White
            ],
            2 => [ // 2: Emerald Mint (Linux Mint)
                (0, 6, 12, 8),     // 0: Forest Black
                (1, 15, 20, 40),   // 1: Indigo Blue
                (2, 15, 56, 28),   // 2: Emerald Green
                (3, 10, 35, 45),   // 3: Dark Teal
                (4, 48, 15, 18),   // 4: Soft Coral Red
                (5, 38, 15, 52),   // 5: Premium Lavender
                (6, 40, 25, 12),   // 6: Warm Copper Brown
                (7, 50, 50, 52),   // 7: Sleek Silver Gray
                (8, 14, 20, 16),   // 8: Forest Charcoal
                (9, 15, 56, 28),   // 9: Neon Mint Green Accent
                (10, 18, 56, 30),  // 10: Neon Mint Green
                (11, 10, 56, 56),  // 11: Glowing Neon Cyan
                (12, 58, 20, 24),  // 12: Vibrant Rose Red
                (13, 48, 22, 58),  // 13: Elegant Neon Violet
                (14, 63, 48, 12),  // 14: Amber Gold
                (15, 61, 61, 63),  // 15: Crisp Pure White
            ],
            3 => [ // 3: Coral Sunset (Aura Sunset)
                (0, 14, 10, 10),   // 0: Charcoal Black
                (1, 15, 20, 40),   // 1: Indigo Blue
                (2, 12, 45, 24),   // 2: Emerald Green
                (3, 10, 35, 45),   // 3: Dark Teal
                (4, 56, 18, 18),   // 4: Coral Rose Accent
                (5, 38, 15, 52),   // 5: Premium Lavender
                (6, 40, 25, 12),   // 6: Warm Copper Brown
                (7, 50, 50, 52),   // 7: Sleek Silver Gray
                (8, 24, 18, 18),   // 8: Sunset Slate
                (9, 56, 18, 18),   // 9: Coral Rose Accent
                (10, 18, 56, 30),  // 10: Neon Mint Green
                (11, 10, 56, 56),  // 11: Glowing Neon Cyan
                (12, 58, 20, 24),  // 12: Vibrant Rose Red
                (13, 48, 22, 58),  // 13: Elegant Neon Violet
                (14, 63, 48, 12),  // 14: Amber Gold
                (15, 63, 60, 58),  // 15: Warm White
            ],
            4 => [ // 4: Classic Light Mode
                (0, 58, 58, 60),   // 0: Silver White Background
                (1, 10, 15, 30),   // 1: Deep Indigo (darker)
                (2, 10, 38, 20),   // 2: Forest Green
                (3, 10, 30, 40),   // 3: Dark Teal
                (4, 45, 10, 12),   // 4: Deep Red
                (5, 30, 10, 40),   // 5: Deep Purple
                (6, 35, 20, 10),   // 6: Dark Brown
                (7, 40, 40, 42),   // 7: Dark Silver Gray
                (8, 48, 48, 50),   // 8: Soft Platinum
                (9, 10, 25, 45),   // 9: Deep Sky Blue Accent
                (10, 12, 45, 20),  // 10: Dark Mint Green
                (11, 10, 45, 45),  // 11: Dark Cyan
                (12, 50, 15, 18),  // 12: Dark Rose
                (13, 40, 15, 50),  // 13: Dark Violet
                (14, 50, 38, 10),  // 14: Dark Amber Folder
                (15, 5, 5, 8),     // 15: Deep Ink Black Text
            ],
            _ => [ // 0: Sky Blue (Fluent Default)
                (0, 10, 12, 18),   // 0: Deep Slate Black
                (1, 15, 20, 40),   // 1: Indigo Blue
                (2, 12, 45, 24),   // 2: Emerald Green
                (3, 10, 35, 45),   // 3: Dark Teal
                (4, 48, 15, 18),   // 4: Soft Coral Red
                (5, 38, 15, 52),   // 5: Premium Lavender
                (6, 40, 25, 12),   // 6: Warm Copper Brown
                (7, 50, 50, 52),   // 7: Sleek Silver Gray
                (8, 20, 22, 28),   // 8: Charcoal Dark Gray
                (9, 10, 38, 63),   // 9: Fluent Sky Blue Accent
                (10, 18, 56, 30),  // 10: Neon Mint Green
                (11, 10, 56, 56),  // 11: Glowing Neon Cyan
                (12, 58, 20, 24),  // 12: Vibrant Rose Red
                (13, 48, 22, 58),  // 13: Elegant Neon Violet
                (14, 63, 48, 12),  // 14: Amber Gold
                (15, 61, 61, 63),  // 15: Crisp Pure White
            ]
        };

        for &(index, r, g, b) in custom_palette.iter() {
            outb(0x3C8, index);
            outb(0x3C9, r);
            outb(0x3C9, g);
            outb(0x3C9, b);
        }
    }
}

/// Programs the VGA DAC with DOOM's full 256-color palette.
/// This uses the DOOM engine's actual PLAYPAL colors so the game
/// renders with authentic visuals on bare-metal VGA Mode 13h.
#[cfg(not(no_doom_engine))]
pub fn set_doom_vga_palette() {
    unsafe {
        // Import the DOOM palette from the doom_game module
        let palette = &crate::doom_game::DOOM_PALETTE_RGB;
        for (i, &(r, g, b)) in palette.iter().enumerate() {
            outb(0x3C8, i as u8);
            outb(0x3C9, r);
            outb(0x3C9, g);
            outb(0x3C9, b);
        }
    }
}


