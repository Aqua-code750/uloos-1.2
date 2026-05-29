use crate::keyboard::inb;

pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

pub unsafe fn mouse_wait(a_type: u8) {
    let mut timeout = 1_000;
    while timeout > 0 {
        let status = inb(0x64);
        if a_type == 0 {
            if status & 1 != 0 { return; }
        } else {
            if status & 2 == 0 { return; }
        }
        timeout -= 1;
    }
}

pub unsafe fn mouse_write(write_val: u8) {
    mouse_wait(1);
    outb(0x64, 0xD4);
    mouse_wait(1);
    outb(0x60, write_val);
}

pub unsafe fn mouse_read() -> u8 {
    mouse_wait(0);
    inb(0x60)
}

pub fn mouse_init() {
    unsafe {
        // Enable mouse auxiliary device
        mouse_wait(1);
        outb(0x64, 0xA8);

        // Read command byte
        mouse_wait(1);
        outb(0x64, 0x20);
        mouse_wait(0);
        let mut status = inb(0x60);
        status |= 2; // Enable aux interrupt (bit 1)
        status &= !0x20; // Disable aux clock lock (bit 5)

        // Write command byte
        mouse_wait(1);
        outb(0x64, 0x60);
        mouse_wait(1);
        outb(0x60, status);

        // Enable default settings
        mouse_write(0xF6);
        let _ = mouse_read(); // ACK

        // Set mouse sample rate to 200 Hz (maximum reporting frequency)
        mouse_write(0xF3);
        let _ = mouse_read(); // ACK
        mouse_write(200);
        let _ = mouse_read(); // ACK

        // Set mouse resolution to 8 counts/mm (maximum hardware precision)
        mouse_write(0xE8);
        let _ = mouse_read(); // ACK
        mouse_write(3); // 8 counts/mm (3 = 8 counts/mm, 2 = 4, 1 = 2, 0 = 1)
        let _ = mouse_read(); // ACK

        // Enable data reporting
        mouse_write(0xF4);
        let _ = mouse_read(); // ACK
    }
}

// Global mouse coordinates & click state
pub static mut MOUSE_X: i32 = 160;
pub static mut MOUSE_Y: i32 = 100;
pub static mut LEFT_CLICK: bool = false;

static mut MOUSE_CYCLE: u8 = 0;
static mut MOUSE_PACKET: [u8; 3] = [0; 3];

pub fn poll_mouse(sensitivity: usize) -> bool {
    unsafe {
        let mut packet_ready = false;
        
        // Loop to drain all currently buffered bytes from the PS/2 controller at hardware speed
        loop {
            let status = inb(0x64);
            if (status & 1) == 0 {
                break;
            }
            if (status & 0x20) == 0 {
                break; // Keyboard data, do not read!
            }
            let b = inb(0x60);
            
            if MOUSE_CYCLE == 0 {
                // The first byte of a valid PS/2 packet MUST have bit 3 set to 1!
                if (b & 0x08) == 0 {
                    // Out of sync! Skip this byte to resynchronize stream.
                    continue;
                }
                MOUSE_PACKET[0] = b;
                MOUSE_CYCLE = 1;
            } else if MOUSE_CYCLE == 1 {
                MOUSE_PACKET[1] = b;
                MOUSE_CYCLE = 2;
            } else if MOUSE_CYCLE == 2 {
                MOUSE_PACKET[2] = b;
                MOUSE_CYCLE = 0; // Reset state for next packet
                
                // Parse full aligned packet
                let bytes0 = MOUSE_PACKET[0];
                let bytes1 = MOUSE_PACKET[1];
                let bytes2 = MOUSE_PACKET[2];
                
                LEFT_CLICK = (bytes0 & 1) != 0;

                // Parse offsets
                let mut dx = bytes1 as i32;
                let mut dy = bytes2 as i32;
                
                if (bytes0 & 0x10) != 0 {
                    dx -= 256;
                }
                if (bytes0 & 0x20) != 0 {
                    dy -= 256;
                }

                // 1. Apply modern Windows ballistic acceleration curve to relative offsets!
                let abs_dx = dx.abs();
                let abs_dy = dy.abs();
                
                if abs_dx > 8 {
                    dx = dx * 3; // Fast movements are tripled
                } else if abs_dx > 3 {
                    dx = dx * 2; // Medium movements are doubled
                }
                
                if abs_dy > 8 {
                    dy = dy * 3;
                } else if abs_dy > 3 {
                    dy = dy * 2;
                }

                // 2. Apply sensitivity multiplier from System Settings to relative offsets!
                if sensitivity == 1 {
                    dx = dx * 2;
                    dy = dy * 2;
                } else if sensitivity == 2 {
                    dx = dx * 3;
                    dy = dy * 3;
                }

                // Update global coordinates directly in high-resolution graphics pixels (320x200)
                let mut x = MOUSE_X + dx;
                let mut y = MOUSE_Y - dy;

                if x < 0 { x = 0; }
                if x > 319 { x = 319; }
                if y < 0 { y = 0; }
                if y > 199 { y = 199; }

                MOUSE_X = x;
                MOUSE_Y = y;
                packet_ready = true;
            }
        }
        return packet_ready;
    }
}
