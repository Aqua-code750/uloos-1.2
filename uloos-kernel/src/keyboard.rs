pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedKey {
    Raw(u8),
    Ascii(char),
    Backspace,
    Enter,
    Escape,
    Tab,
    F1, F2, F3, F4, F5, F6, F7, F8,
}

pub static mut IS_SHIFT_PRESSED: bool = false;

pub fn get_key() -> Option<DecodedKey> {
    unsafe {
        let status = inb(0x64);
        if status & 1 != 0 {
            if status & 0x20 != 0 {
                return None;
            }
            let scancode = inb(0x60);
            
            // Check shift press
            if scancode == 0x2A || scancode == 0x36 {
                IS_SHIFT_PRESSED = true;
                return None;
            }
            // Check shift release
            if scancode == 0xAA || scancode == 0xB6 {
                IS_SHIFT_PRESSED = false;
                return None;
            }

            // We ignore key releases (scan codes with bit 7 set, i.e., >= 0x80)
            if scancode >= 0x80 {
                return None;
            }

            return match scancode {
                0x01 => Some(DecodedKey::Escape),
                0x0E => Some(DecodedKey::Backspace),
                0x0F => Some(DecodedKey::Tab),
                0x1C => Some(DecodedKey::Enter),
                0x39 => Some(DecodedKey::Ascii(' ')),
                0x3B => Some(DecodedKey::F1),
                0x3C => Some(DecodedKey::F2),
                0x3D => Some(DecodedKey::F3),
                0x3E => Some(DecodedKey::F4),
                0x3F => Some(DecodedKey::F5),
                0x40 => Some(DecodedKey::F6),
                0x41 => Some(DecodedKey::F7),
                0x42 => Some(DecodedKey::F8),
                _ => {
                    let mut c = translate_scancode(scancode);
                    if c != '\0' {
                        if IS_SHIFT_PRESSED {
                            c = make_uppercase(c);
                        }
                        Some(DecodedKey::Ascii(c))
                    } else {
                        Some(DecodedKey::Raw(scancode))
                    }
                }
            };
        }
    }
    None
}

fn make_uppercase(c: char) -> char {
    match c {
        'a'..='z' => (c as u8 - 32) as char,
        '1' => '!', '2' => '@', '3' => '#', '4' => '$',
        '5' => '%', '6' => '^', '7' => '&', '8' => '*',
        '9' => '(', '0' => ')',
        '-' => '_', '=' => '+', '[' => '{', ']' => '}',
        ';' => ':', '\'' => '"', ',' => '<', '.' => '>',
        '/' => '?', '\\' => '|',
        _ => c,
    }
}

fn translate_scancode(scancode: u8) -> char {
    match scancode {
        0x02 => '1', 0x03 => '2', 0x04 => '3', 0x05 => '4',
        0x06 => '5', 0x07 => '6', 0x08 => '7', 0x09 => '8',
        0x0A => '9', 0x0B => '0',
        0x10 => 'q', 0x11 => 'w', 0x12 => 'e', 0x13 => 'r',
        0x14 => 't', 0x15 => 'y', 0x16 => 'u', 0x17 => 'i',
        0x18 => 'o', 0x19 => 'p',
        0x1E => 'a', 0x1F => 's', 0x20 => 'd', 0x21 => 'f',
        0x22 => 'g', 0x23 => 'h', 0x24 => 'j', 0x25 => 'k',
        0x26 => 'l',
        0x2C => 'z', 0x2D => 'x', 0x2E => 'c', 0x2F => 'v',
        0x30 => 'b', 0x31 => 'n', 0x32 => 'm',
        0x33 => ',', 0x34 => '.', 0x35 => '/',
        0x0C => '-', 0x0D => '=', 0x1A => '[', 0x1B => ']',
        0x27 => ';', 0x28 => '\'', 0x2B => '\\',
        _ => '\0',
    }
}
