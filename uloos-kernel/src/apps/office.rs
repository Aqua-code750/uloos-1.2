use crate::vga_driver::VGA;

// ==========================================
// ULOTEXT: Text Editor
// ==========================================
pub struct UloText {
    pub buffer: [u8; 1000],
    pub len: usize,
}

impl UloText {
    pub const fn new() -> Self {
        UloText {
            buffer: [0; 1000],
            len: 0,
        }
    }

    pub fn draw(&self) {
        // High white workspace
        VGA.draw_rect(12, 28, 296, 144, 15);

        VGA.draw_string(16, 32, "UloText Document Suite", 1);
        VGA.draw_string(16, 44, "--------------------------", 8);

        // Render buffer contents dynamically inside graphic margins
        let mut curr_x = 16;
        let mut curr_y = 56;
        for i in 0..self.len {
            let byte = self.buffer[i];
            if byte == b'\n' {
                curr_x = 16;
                curr_y += 12;
                if curr_y >= 160 { break; }
            } else {
                VGA.draw_char(curr_x, curr_y, byte as char, 0);
                curr_x += 8;
                if curr_x >= 290 {
                    curr_x = 16;
                    curr_y += 12;
                    if curr_y >= 160 { break; }
                }
            }
        }
        // Blinking graphic cursor
        VGA.draw_rect(curr_x, curr_y + 6, 8, 2, 1);
    }

    pub fn handle_key(&mut self, key: char) {
        if self.len < 250 { // Graphical text length safe margin
            self.buffer[self.len] = key as u8;
            self.len += 1;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }
}

// ==========================================
// ULOSLIDES: Presentation Creator
// ==========================================
pub struct UloSlides {
    pub slides: [(&'static str, &'static str); 3],
    pub current: usize,
}

impl UloSlides {
    pub const fn new() -> Self {
        UloSlides {
            slides: [
                ("Welcome to UloSlides!", "The primary student slide deck engine written completely in Rust."),
                ("Advanced Rust Kernel", "Supports custom x86_64 target specifications with 0 compiler dependencies."),
                ("System Performance", "Optimized microkernel layout running efficiently inside QEMU."),
            ],
            current: 0,
        }
    }

    pub fn draw(&self) {
        // Gray slide stage background
        VGA.draw_rect(12, 28, 296, 144, 7);

        // Slide card inner
        VGA.draw_rect(24, 40, 272, 120, 15);
        VGA.draw_rect(24, 40, 272, 15, 14); // Yellow card header

        VGA.draw_string(28, 43, "Slide Preview", 0);

        let current_slide = self.slides[self.current];
        VGA.draw_string(32, 70, current_slide.0, 1);
        VGA.draw_string(32, 95, current_slide.1, 0);

        // Progress footer
        let mut progress = [0u8; 15];
        progress[..12].copy_from_slice(b"Slide: [ /3]");
        progress[8] = b'1' + self.current as u8;
        VGA.draw_string(110, 140, core::str::from_utf8(&progress).unwrap(), 8);
    }

    pub fn next(&mut self) {
        if self.current < 2 {
            self.current += 1;
        } else {
            self.current = 0;
        }
    }
}

// ==========================================
// ULONUMBERS: Spreadsheet Calculator
// ==========================================
pub struct UloNumbers {
    pub cells: [[i32; 5]; 5],
    pub selected_r: usize,
    pub selected_c: usize,
}

impl UloNumbers {
    pub const fn new() -> Self {
        UloNumbers {
            cells: [
                [100, 200, 300, 0, 0],
                [50, 60, 110, 0, 0],
                [25, 25, 50, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            selected_r: 0,
            selected_c: 0,
        }
    }

    pub fn draw(&self) {
        // High white workspace
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Columns headers
        VGA.draw_string(16, 34, "Row  |  A   |  B   |  C   |  D   ", 2);
        VGA.draw_rect(16, 44, 288, 1, 8); // Header divider line

        for r in 0..5 {
            let row_y = 52 + r * 16;
            
            // Draw row count label
            let mut row_label = [b' ', b' ', b'1' + r as u8, b' ', b'|', b'\0'];
            VGA.draw_string(16, row_y, core::str::from_utf8(&row_label[..5]).unwrap(), 2);

            for c in 0..4 {
                let cell_val = self.cells[r][c];
                let is_selected = self.selected_r == r && self.selected_c == c;
                
                let mut buf = [b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'\0'];
                let val_str = integer_to_str(cell_val);
                buf[1..1 + val_str.len()].copy_from_slice(val_str.as_bytes());

                let bg_color = if is_selected { 11 } else { 15 }; // Cyan highlight if active
                let fg_color = if is_selected { 0 } else { 8 };

                VGA.draw_rect(52 + c * 56, row_y - 2, 48, 12, bg_color);
                VGA.draw_string(54 + c * 56, row_y, core::str::from_utf8(&buf[..7]).unwrap(), fg_color);
            }
        }
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected_r > 0 { self.selected_r -= 1; } }
            's' | 'S' => { if self.selected_r < 4 { self.selected_r += 1; } }
            'a' | 'A' => { if self.selected_c > 0 { self.selected_c -= 1; } }
            'd' | 'D' => { if self.selected_c < 3 { self.selected_c += 1; } }
            '+' => { self.cells[self.selected_r][self.selected_c] += 10; }
            '-' => { self.cells[self.selected_r][self.selected_c] -= 10; }
            _ => {}
        }
    }
}

// ==========================================
// ULOMAIL: Student and Office Mail Client
// ==========================================
pub struct UloMail {
    pub inbox: [(&'static str, &'static str, &'static str); 2],
    pub selected: usize,
}

impl UloMail {
    pub const fn new() -> Self {
        UloMail {
            inbox: [
                ("Admin Portal", "System Setup Success", "Hello! UloOS has loaded fully on raw x86 hardware. Check text/slides apps."),
                ("Rust Dev Team", "Nightly Compiler Update", "Be sure to run 'cargo run' to start within emulated QEMU without issues."),
            ],
            selected: 0,
        }
    }

    pub fn draw(&self) {
        // High white browser background
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Sidebar list background (gray)
        VGA.draw_rect(12, 28, 80, 144, 7);

        for idx in 0..2 {
            let sender = self.inbox[idx].0;
            let is_sel = idx == self.selected;
            let bg_col = if is_sel { 1 } else { 7 };
            let fg_col = if is_sel { 15 } else { 0 };
            
            VGA.draw_rect(14, 34 + idx * 24, 76, 16, bg_col);
            VGA.draw_string(16, 38 + idx * 24, sender, fg_col);
        }

        // Selected mail content body
        let mail = self.inbox[self.selected];
        VGA.draw_string(100, 34, "Subject: ", 8);
        VGA.draw_string(164, 34, mail.1, 0);

        VGA.draw_string(100, 50, "Content Body:", 1);
        
        // Print message lines cleanly
        VGA.draw_string(100, 68, "Message:", 8);
        if mail.2.len() > 24 {
            VGA.draw_string(100, 82, &mail.2[..24], 0);
        } else {
            VGA.draw_string(100, 82, mail.2, 0);
        }
    }

    pub fn toggle(&mut self) {
        self.selected = if self.selected == 0 { 1 } else { 0 };
    }
}

fn integer_to_str(val: i32) -> &'static str {
    match val {
        0 => "0",
        25 => "25",
        50 => "50",
        60 => "60",
        100 => "100",
        110 => "110",
        200 => "200",
        300 => "300",
        _ => "Custom",
    }
}
