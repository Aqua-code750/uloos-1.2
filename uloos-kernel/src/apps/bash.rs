use crate::vga_driver::VGA;

pub struct BashShell {
    pub input_buffer: [u8; 80],
    pub input_len: usize,
    pub history: [Option<[u8; 80]>; 5],
    pub history_count: usize,
}

impl BashShell {
    pub const fn new() -> Self {
        BashShell {
            input_buffer: [0; 80],
            input_len: 0,
            history: [None, None, None, None, None],
            history_count: 0,
        }
    }

    pub fn draw(&self) {
        // Draw inside our graphical window box (x:10, y:15, w:300, h:160)
        // Background terminal color (VGA 0: Black)
        VGA.draw_rect(12, 28, 296, 144, 0);

        VGA.draw_string(16, 32, "UloOS Bash Prompt [DOS Mode]", 11);
        VGA.draw_string(16, 44, "Type commands: help, neofetch, doom, exit", 7);

        // Draw history commands
        for i in 0..self.history_count {
            if let Some(ref hist_cmd) = self.history[i] {
                let cmd_str = core::str::from_utf8(hist_cmd).unwrap();
                VGA.draw_string(16, 60 + i * 14, "uloos:/$ ", 14); // Yellow prompt
                VGA.draw_string(90, 60 + i * 14, cmd_str, 15);     // White text
            }
        }

        // Draw prompt input line
        let prompt_y = 60 + self.history_count * 14;
        if prompt_y < 160 {
            VGA.draw_string(16, prompt_y, "uloos:/$ ", 14);
            if self.input_len > 0 {
                let input_str = core::str::from_utf8(&self.input_buffer[..self.input_len]).unwrap();
                VGA.draw_string(90, prompt_y, input_str, 15);
            }
            // Draw graphical prompt cursor blink bar
            let cursor_x = 90 + self.input_len * 8;
            VGA.draw_rect(cursor_x, prompt_y + 6, 8, 2, 11); // Cyan cursor
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.input_len < 25 { // Keep within compact graphic window limits
            self.input_buffer[self.input_len] = c as u8;
            self.input_len += 1;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.input_len > 0 {
            self.input_len -= 1;
        }
    }

    pub fn handle_enter(&mut self) -> Option<&str> {
        if self.input_len == 0 {
            return None;
        }

        if self.history_count < 5 {
            let mut hist = [0u8; 80];
            hist[..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history[self.history_count] = Some(hist);
            self.history_count += 1;
        } else {
            for i in 1..5 {
                self.history[i - 1] = self.history[i];
            }
            let mut hist = [0u8; 80];
            hist[..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history[4] = Some(hist);
        }

        let cmd = core::str::from_utf8(&self.input_buffer[..self.input_len]).unwrap();
        self.input_len = 0;
        Some(cmd)
    }
}
