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
                ("1. Microkernel Architecture", "Featuring multi-threaded scheduler loops, CMOS sync, and IST offset timezone clocks."),
                ("2. Graphical VGA Mode 13h", "Rendering beautiful 256-color flat carbon-grid designs on bare-metal systems."),
                ("3. Modular Rust Drivers", "Zero dependencies compile configuration loaded cleanly inside QEMU x86_64 target."),
            ],
            current: 0,
        }
    }

    pub fn draw(&self) {
        // High quality dark gray stage
        VGA.draw_rect(12, 28, 296, 144, 8);

        // Sidebar thumbnails panel
        VGA.draw_rect(12, 28, 60, 144, 7); // gray background
        VGA.draw_string(14, 34, "Slides", 1);
        
        for idx in 0..3 {
            let is_sel = idx == self.current;
            let bg_col = if is_sel { 1 } else { 7 };
            let fg_col = if is_sel { 15 } else { 8 };
            
            VGA.draw_rect(14, 46 + idx * 24, 56, 18, bg_col);
            let mut label = [b'P', b'a', b'g', b'e', b' ', b'1', b'\0'];
            label[5] = b'1' + idx as u8;
            VGA.draw_string(16, 50 + idx * 24, core::str::from_utf8(&label[..6]).unwrap(), fg_col);
        }

        // Active presentation slide card
        VGA.draw_rect(76, 34, 226, 132, 15); // Pure white slide paper sheet
        VGA.draw_rect(76, 34, 226, 12, 11); // Blue indicator header
        VGA.draw_string(80, 36, "Slide Deck Show", 0);

        let current_slide = self.slides[self.current];
        VGA.draw_string(82, 54, current_slide.0, 1);
        
        // Wrap presentation description text beautifully
        let desc = current_slide.1;
        if desc.len() > 24 {
            VGA.draw_string(82, 74, &desc[..24], 8);
            if desc.len() > 48 {
                VGA.draw_string(82, 88, &desc[24..48], 8);
                VGA.draw_string(82, 102, &desc[48..], 8);
            } else {
                VGA.draw_string(82, 88, &desc[24..], 8);
            }
        } else {
            VGA.draw_string(82, 74, desc, 8);
        }

        // Animated index progress indicator bar
        VGA.draw_rect(82, 128, 214, 4, 7); // progress track
        let fill_w = 71 * (self.current + 1);
        VGA.draw_rect(82, 128, fill_w, 4, 2); // green fill

        VGA.draw_string(82, 142, "Press [Space] Next Slide", 8);
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
                [100, 50, 0, 0, 0],
                [80, 20, 0, 0, 0],
                [30, 40, 0, 0, 0],
                [10, 10, 0, 0, 0],
                [5, 5, 0, 0, 0],
            ],
            selected_r: 0,
            selected_c: 0,
        }
    }

    pub fn draw(&mut self) {
        // High white workspace
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Dynamic formula recalculation: C = A + B, D = A - B
        for r in 0..5 {
            self.cells[r][2] = self.cells[r][0] + self.cells[r][1];
            self.cells[r][3] = self.cells[r][0] - self.cells[r][1];
        }

        // Columns headers
        VGA.draw_string(16, 32, "Row |  Col A |  Col B | Col C(SUM) ", 1);
        VGA.draw_rect(16, 42, 288, 1, 8); // Header divider line

        for r in 0..5 {
            let row_y = 48 + r * 15;
            
            // Draw row count label
            let mut row_label = [b' ', b' ', b'1' + r as u8, b' ', b'|', b'\0'];
            VGA.draw_string(16, row_y, core::str::from_utf8(&row_label[..5]).unwrap(), 8);

            for c in 0..3 {
                let cell_val = self.cells[r][c];
                let is_selected = self.selected_r == r && self.selected_c == c;
                
                let mut buf = [b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'\0'];
                
                // Write dynamic values to string buffer
                let mut temp = cell_val;
                let mut is_neg = false;
                if temp < 0 {
                    is_neg = true;
                    temp = -temp;
                }
                
                let mut char_idx = 5;
                if temp == 0 {
                    buf[char_idx] = b'0';
                } else {
                    while temp > 0 && char_idx > 0 {
                        buf[char_idx] = b'0' + (temp % 10) as u8;
                        temp /= 10;
                        char_idx -= 1;
                    }
                    if is_neg && char_idx > 0 {
                        buf[char_idx] = b'-';
                    }
                }

                let bg_color = if is_selected { 11 } else { 15 }; // Cyan highlight
                let fg_color = if is_selected { 0 } else { 8 };

                VGA.draw_rect(54 + c * 80, row_y - 2, 60, 11, bg_color);
                VGA.draw_string(58 + c * 80, row_y, core::str::from_utf8(&buf[..7]).unwrap(), fg_color);
            }
        }

        // Live calculation total Sum footer
        let mut total_sum = 0;
        for r in 0..5 {
            total_sum += self.cells[r][2];
        }

        VGA.draw_rect(16, 126, 288, 1, 8); // total divider
        VGA.draw_string(16, 132, "Total Formula Sum of Cells:", 12);
        
        let mut total_buf = [b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'\0'];
        let mut temp = total_sum;
        let mut char_idx = 5;
        if temp == 0 {
            total_buf[char_idx] = b'0';
        } else {
            while temp > 0 && char_idx > 0 {
                total_buf[char_idx] = b'0' + (temp % 10) as u8;
                temp /= 10;
                char_idx -= 1;
            }
        }
        VGA.draw_string(230, 132, core::str::from_utf8(&total_buf[..7]).unwrap(), 12);

        VGA.draw_string(16, 150, "WASD to Navigate | Press [+] / [-] Edit", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected_r > 0 { self.selected_r -= 1; } }
            's' | 'S' => { if self.selected_r < 4 { self.selected_r += 1; } }
            'a' | 'A' => { if self.selected_c > 0 { self.selected_c -= 1; } }
            'd' | 'D' => { if self.selected_c < 1 { self.selected_c += 1; } } // edit Col A & Col B
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

// ==========================================
// ULOWEATHER: Dynamic Weather dashboard
// ==========================================
pub struct UloWeather {
    pub selected: usize, // 0 = Mumbai, 1 = NY, 2 = London, 3 = Tokyo
}

impl UloWeather {
    pub const fn new() -> Self {
        UloWeather { selected: 0 }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        VGA.draw_rect(12, 28, 296, 14, 9);
        VGA.draw_string(16, 31, "UloWeather Live Forecast", 15);

        let cities = ["Mumbai, IN", "New York, US", "London, UK", "Tokyo, JP"];
        let temps = ["29 C", "22 C", "15 C", "18 C"];
        let conds = ["Rainy Monsoon", "Sunny Blue Sky", "Foggy Drizzle", "Blossom Breeze"];
        let humidities = ["85%", "42%", "92%", "50%"];
        let wind = ["24 km/h", "8 km/h", "18 km/h", "15 km/h"];

        VGA.draw_rect(12, 42, 90, 130, 7);
        for idx in 0..4 {
            let is_sel = idx == self.selected;
            let bg_col = if is_sel { 1 } else { 7 };
            let fg_col = if is_sel { 15 } else { 0 };
            VGA.draw_rect(14, 46 + idx * 24, 86, 16, bg_col);
            VGA.draw_string(16, 50 + idx * 24, cities[idx], fg_col);
        }

        VGA.draw_string(110, 50, "City: ", 8);
        VGA.draw_string(150, 50, cities[self.selected], 1);

        VGA.draw_string(110, 70, "Temperature: ", 8);
        VGA.draw_string(210, 70, temps[self.selected], 12);

        VGA.draw_string(110, 90, "Condition: ", 8);
        VGA.draw_string(200, 90, conds[self.selected], 2);

        VGA.draw_string(110, 110, "Humidity: ", 8);
        VGA.draw_string(180, 110, humidities[self.selected], 0);

        VGA.draw_string(110, 130, "Wind Speed: ", 8);
        VGA.draw_string(190, 130, wind[self.selected], 0);

        VGA.draw_string(110, 152, "[W/S] Change City", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => { if self.selected < 3 { self.selected += 1; } }
            _ => {}
        }
    }
}

// ==========================================
// ULOMUSIC: Audio Synthesizer
// ==========================================
pub struct UloMusic {
    pub last_frequency: u32,
    pub wave_type: usize,
}

impl UloMusic {
    pub const fn new() -> Self {
        UloMusic { last_frequency: 0, wave_type: 0 }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 0);

        VGA.draw_rect(12, 28, 296, 14, 5);
        VGA.draw_string(16, 31, "UloMusic Audio Synthesizer", 15);

        VGA.draw_rect(16, 55, 288, 1, 8);
        if self.last_frequency > 0 {
            for x in (16..300).step_by(12) {
                let peak = if x % 24 == 0 { 15 } else { 5 };
                VGA.draw_rect(x, 55 - peak, 2, peak * 2, 10);
            }
            VGA.draw_string(16, 80, "STATUS: Playing synthesized tone...", 10);
        } else {
            VGA.draw_string(16, 80, "STATUS: Silent (Press key to play)", 8);
        }

        VGA.draw_string(16, 105, "Press Keys to play frequency:", 15);
        VGA.draw_string(16, 120, "1:C4  2:D4  3:E4  4:F4  5:G4  6:A4", 11);
        VGA.draw_string(16, 134, "7:B4  8:C5  [Space]: Stop Tone", 11);
    }

    pub fn handle_input(&mut self, key: char) {
        let freq = match key {
            '1' => 261,
            '2' => 293,
            '3' => 329,
            '4' => 349,
            '5' => 392,
            '6' => 440,
            '7' => 493,
            '8' => 523,
            ' ' => 0,
            _ => return,
        };
        self.last_frequency = freq;
        unsafe {
            crate::sound::play_tone(freq);
        }
    }
}

// ==========================================
// ULOKEEP: Sticky notes manager
// ==========================================
pub struct UloKeep {
    pub notes: [&'static str; 3],
    pub selected: usize,
}

impl UloKeep {
    pub const fn new() -> Self {
        UloKeep {
            notes: [
                "Deploy UloOS to bare metal x86 hardware safely.",
                "Star the GitHub Aqua-code750/uloos-1.2 repo!",
                "Check out the beautiful Fluent web simulator design.",
            ],
            selected: 0,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        VGA.draw_rect(12, 28, 296, 14, 14);
        VGA.draw_string(16, 31, "UloKeep Sticky Notes Board", 0);

        for idx in 0..3 {
            let is_sel = idx == self.selected;
            let bg_col = if is_sel { 14 } else { 7 };
            let fg_col = if is_sel { 0 } else { 8 };
            
            VGA.draw_rect(18, 48 + idx * 36, 100, 30, bg_col);
            
            let mut label = [b'N', b'o', b't', b'e', b' ', b'0', b'\0'];
            label[5] = b'1' + idx as u8;
            VGA.draw_string(24, 58 + idx * 36, core::str::from_utf8(&label[..6]).unwrap(), fg_col);
        }

        VGA.draw_rect(130, 48, 168, 98, 14);
        VGA.draw_string(136, 52, "Selected Stickie:", 8);

        let note_text = self.notes[self.selected];
        if note_text.len() > 20 {
            VGA.draw_string(136, 72, &note_text[..20], 0);
            if note_text.len() > 40 {
                VGA.draw_string(136, 88, &note_text[20..40], 0);
                VGA.draw_string(136, 104, &note_text[40..], 0);
            } else {
                VGA.draw_string(136, 88, &note_text[20..], 0);
            }
        } else {
            VGA.draw_string(136, 72, note_text, 0);
        }

        VGA.draw_string(136, 130, "[W/S] Choose stickie note", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => { if self.selected < 2 { self.selected += 1; } }
            _ => {}
        }
    }
}
