use core::fmt;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: *mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                let buffer = unsafe { &mut *self.buffer };
                buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_at(&mut self, x: usize, y: usize, s: &str, color: ColorCode) {
        if y >= BUFFER_HEIGHT { return; }
        let mut curr_x = x;
        let buffer = unsafe { &mut *self.buffer };
        for byte in s.bytes() {
            if curr_x >= BUFFER_WIDTH { break; }
            buffer.chars[y][curr_x].write(ScreenChar {
                ascii_character: byte,
                color_code: color,
            });
            curr_x += 1;
        }
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char, color: ColorCode) {
        if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT { return; }
        let buffer = unsafe { &mut *self.buffer };
        buffer.chars[y][x].write(ScreenChar {
            ascii_character: c as u8,
            color_code: color,
        });
    }

    fn new_line(&mut self) {
        let buffer = unsafe { &mut *self.buffer };
        if self.row_position < BUFFER_HEIGHT - 2 {
            self.row_position += 1;
            self.column_position = 0;
        } else {
            // Scroll up (but keep the last 2 lines for taskbar and border)
            for row in 1..(BUFFER_HEIGHT - 2) {
                for col in 0..BUFFER_WIDTH {
                    let character = buffer.chars[row][col].read();
                    buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 3);
            self.column_position = 0;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        let buffer = unsafe { &mut *self.buffer };
        for col in 0..BUFFER_WIDTH {
            buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
    }

    pub fn set_color(&mut self, color: ColorCode) {
        self.color_code = color;
      }

    pub fn set_position(&mut self, x: usize, y: usize) {
        self.column_position = x;
        self.row_position = y;
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, border_color: ColorCode, bg_color: ColorCode) {
        let buffer = unsafe { &mut *self.buffer };
        for curr_y in y..(y + h) {
            if curr_y >= BUFFER_HEIGHT { break; }
            for curr_x in x..(x + w) {
                if curr_x >= BUFFER_WIDTH { break; }
                
                let is_border = curr_y == y || curr_y == y + h - 1 || curr_x == x || curr_x == x + w - 1;
                let c = if is_border {
                    if curr_y == y || curr_y == y + h - 1 { b'-' } else { b'|' }
                } else {
                    b' '
                };
                
                buffer.chars[curr_y][curr_x].write(ScreenChar {
                    ascii_character: c,
                    color_code: if is_border { border_color } else { bg_color },
                });
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    row_position: 0,
    color_code: ColorCode((Color::Black as u8) << 4 | (Color::White as u8)),
    buffer: 0xb8000 as *mut Buffer,
});

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
