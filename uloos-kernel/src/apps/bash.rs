use crate::vga_driver::VGA;

pub struct BashShell {
    pub input_buffer: [u8; 80],
    pub input_len: usize,
    pub history: [Option<[u8; 80]>; 6],
    pub history_count: usize,
    pub output_lines: [[u8; 40]; 4],
    pub output_count: usize,
}

impl BashShell {
    pub const fn new() -> Self {
        BashShell {
            input_buffer: [0; 80],
            input_len: 0,
            history: [None, None, None, None, None, None],
            history_count: 0,
            output_lines: [[0u8; 40]; 4],
            output_count: 0,
        }
    }

    fn set_output(&mut self, lines: &[&str]) {
        self.output_count = 0;
        for line in lines.iter() {
            if self.output_count >= 4 { break; }
            let len = if line.len() > 38 { 38 } else { line.len() };
            self.output_lines[self.output_count] = [0; 40];
            self.output_lines[self.output_count][..len].copy_from_slice(&line.as_bytes()[..len]);
            self.output_count += 1;
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 0);

        VGA.draw_string(16, 32, "UloOS Terminal [DOS Mode]", 11);
        VGA.draw_string(16, 42, "Type 'help' for commands", 7);

        // Draw command output (if any)
        let mut y = 56;
        for i in 0..self.output_count {
            if let Ok(line) = core::str::from_utf8(&self.output_lines[i]) {
                let trimmed = line.trim_end_matches('\0');
                if !trimmed.is_empty() {
                    VGA.draw_string(16, y, trimmed, 10);
                    y += 10;
                }
            }
        }

        // Draw history commands
        for i in 0..self.history_count {
            if let Some(ref hist_cmd) = self.history[i] {
                if let Ok(cmd_str) = core::str::from_utf8(hist_cmd) {
                    let trimmed = cmd_str.trim_end_matches('\0');
                    if !trimmed.is_empty() && y < 148 {
                        VGA.draw_string(16, y, "$ ", 14);
                        VGA.draw_string(32, y, trimmed, 15);
                        y += 10;
                    }
                }
            }
        }

        // Draw prompt input line
        if y < 160 {
            VGA.draw_string(16, y, "$ ", 14);
            if self.input_len > 0 {
                if let Ok(input_str) = core::str::from_utf8(&self.input_buffer[..self.input_len]) {
                    VGA.draw_string(32, y, input_str, 15);
                }
            }
            // Cursor
            let cursor_x = 32 + self.input_len * 6;
            if cursor_x < 300 {
                VGA.draw_rect(cursor_x, y + 6, 5, 2, 11);
            }
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.input_len < 25 {
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

        // Copy command to local buffer to avoid borrow conflict
        let mut cmd_buf = [0u8; 30];
        let cmd_len = if self.input_len > 28 { 28 } else { self.input_len };
        cmd_buf[..cmd_len].copy_from_slice(&self.input_buffer[..cmd_len]);

        // Detect command by comparing bytes
        let is_cmd = |expected: &[u8]| -> bool {
            if cmd_len != expected.len() { return false; }
            for i in 0..cmd_len {
                if cmd_buf[i] != expected[i] { return false; }
            }
            true
        };

        // Handle built-in commands
        if is_cmd(b"help") {
            self.set_output(&[
                "Commands: help neofetch ls",
                "  whoami clear doom exit",
                "  explorer office uname",
            ]);
        } else if is_cmd(b"neofetch") {
            self.set_output(&[
                "UloOS 1.2 x86_64 QEMU",
                "Kernel: Rust no_std bare-metal",
                "Display: VGA 320x200 256-color",
                "Shell: UloTerminal v1.0",
            ]);
        } else if is_cmd(b"ls") {
            self.set_output(&[
                "Welcome.txt  system.cfg",
                "Documents/   Projects/",
                "notes.txt    readme.md",
            ]);
        } else if is_cmd(b"whoami") {
            self.set_output(&[
                "uloos-root (superuser)",
            ]);
        } else if is_cmd(b"uname") {
            self.set_output(&[
                "UloOS 1.2.0 x86_64 QEMU-KVM",
                "rustc nightly-2026 bare-metal",
            ]);
        } else if is_cmd(b"clear") {
            self.history_count = 0;
            self.output_count = 0;
            self.input_len = 0;
            return None;
        } else {
            self.output_count = 0;
        }

        // Store in history
        if self.history_count < 6 {
            let mut hist = [0u8; 80];
            hist[..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history[self.history_count] = Some(hist);
            self.history_count += 1;
        } else {
            for i in 1..6 {
                self.history[i - 1] = self.history[i];
            }
            let mut hist = [0u8; 80];
            hist[..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history[5] = Some(hist);
        }

        // Return command for main.rs to check app-switching commands
        let cmd = core::str::from_utf8(&self.input_buffer[..self.input_len]).unwrap();
        self.input_len = 0;
        Some(cmd)
    }
}
