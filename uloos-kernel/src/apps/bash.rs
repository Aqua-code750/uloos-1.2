use crate::vga_driver::VGA;

pub struct BashShell {
    pub input_buffer: [u8; 80],
    pub input_len: usize,
    pub lines: [[u8; 60]; 12],
    pub line_count: usize,
}

impl BashShell {
    pub const fn new() -> Self {
        let lines = [[0u8; 60]; 12];
        BashShell {
            input_buffer: [0; 80],
            input_len: 0,
            lines,
            line_count: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        for i in 1..12 {
            self.lines[i - 1] = self.lines[i];
        }
        self.lines[11] = [0u8; 60];
        if self.line_count > 0 {
            self.line_count -= 1;
        }
    }

    pub fn add_line(&mut self, text: &str) {
        if self.line_count >= 12 {
            self.scroll_up();
            self.line_count = 11;
        }
        let len = if text.len() > 58 { 58 } else { text.len() };
        self.lines[self.line_count] = [0u8; 60];
        self.lines[self.line_count][..len].copy_from_slice(&text.as_bytes()[..len]);
        self.line_count += 1;
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 0); // Black terminal background

        VGA.draw_string(16, 30, "UloOS Bash Shell v1.2.0 (x86_64)", 11); // cyan
        VGA.draw_rect(12, 40, 296, 1, 8); // divider line

        let mut y = 44;
        for i in 0..self.line_count {
            if let Ok(line_str) = core::str::from_utf8(&self.lines[i]) {
                let trimmed = line_str.trim_end_matches('\0');
                if !trimmed.is_empty() {
                    if trimmed.starts_with("$ ") {
                        VGA.draw_string(16, y, "$ ", 14); // yellow prompt symbol
                        VGA.draw_string(28, y, &trimmed[2..], 15); // white command
                    } else if trimmed.starts_with("Err:") || trimmed.starts_with("Error:") {
                        VGA.draw_string(16, y, trimmed, 12); // Red for error
                    } else if trimmed.starts_with("Welcome") || trimmed.starts_with("Commands:") {
                        VGA.draw_string(16, y, trimmed, 14); // Yellow for highlights
                    } else {
                        VGA.draw_string(16, y, trimmed, 10); // Green output text
                    }
                    y += 10;
                }
            }
        }

        // Draw active typing prompt line
        if y < 162 {
            VGA.draw_string(16, y, "$ ", 14);
            if self.input_len > 0 {
                if let Ok(input_str) = core::str::from_utf8(&self.input_buffer[..self.input_len]) {
                    VGA.draw_string(28, y, input_str, 15);
                }
            }
            // Cursor
            let cursor_x = 28 + self.input_len * 6;
            if cursor_x < 300 {
                VGA.draw_rect(cursor_x, y + 6, 5, 2, 11);
            }
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.input_len < 40 {
            self.input_buffer[self.input_len] = c as u8;
            self.input_len += 1;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.input_len > 0 {
            self.input_len -= 1;
        }
    }

    pub fn handle_enter(&mut self) -> Option<&'static str> {
        if self.input_len == 0 {
            return None;
        }

        // Copy the input buffer to a local stack array
        let mut cmd_copy = [0u8; 80];
        let copy_len = if self.input_len > 78 { 78 } else { self.input_len };
        cmd_copy[..copy_len].copy_from_slice(&self.input_buffer[..copy_len]);

        // Get command string from the copy
        let cmd_str = match core::str::from_utf8(&cmd_copy[..copy_len]) {
            Ok(s) => s,
            Err(_) => {
                self.input_len = 0;
                return None;
            }
        };

        // Add to prompt history
        let mut prompt_line = [0u8; 60];
        prompt_line[0] = b'$';
        prompt_line[1] = b' ';
        let prompt_copy_len = if cmd_str.len() > 56 { 56 } else { cmd_str.len() };
        prompt_line[2..(2 + prompt_copy_len)].copy_from_slice(&cmd_str.as_bytes()[..prompt_copy_len]);
        
        // Print typed command to screen lines
        let prompt_str = core::str::from_utf8(&prompt_line[..(2 + prompt_copy_len)]).unwrap_or("");
        
        // Clear input_len now
        self.input_len = 0;

        self.add_line(prompt_str);

        // Parse args
        let mut parts = cmd_str.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("");

        let mut app_switch = None;

        match cmd {
            "help" => {
                self.add_line("Commands: help, ls, cd, cat, mkdir, touch,");
                self.add_line("          rm, neofetch, clear, whoami, uname,");
                self.add_line("          doom, office, explorer, exit");
            }
            "neofetch" => {
                self.add_line("UloOS 1.2 x86_64 QEMU");
                self.add_line("Kernel: Rust no_std microkernel");
                self.add_line("Display: VGA Mode 13h 320x200");
                self.add_line("Shell: UloOS POSIX Bash v1.2");
            }
            "whoami" => {
                let name_bytes = crate::USERNAME.lock();
                let name_len = *crate::USERNAME_LEN.lock();
                if let Ok(name) = core::str::from_utf8(&name_bytes[..name_len]) {
                    let mut out = [0u8; 60];
                    let mut len = 0;
                    len += copy_slice(&mut out[len..], name.as_bytes());
                    len += copy_slice(&mut out[len..], b" (superuser)");
                    self.add_line(core::str::from_utf8(&out[..len]).unwrap_or(""));
                } else {
                    self.add_line("uloos-root");
                }
            }
            "uname" => {
                self.add_line("UloOS 1.2.0-nightly x86_64 SMP");
            }
            "clear" => {
                self.line_count = 0;
                self.lines = [[0u8; 60]; 12];
            }
            "ls" => {
                let explorer = crate::EXPLORER.lock();
                let mut line_buf = [0u8; 60];
                let mut len = 0;
                for i in 1..explorer.entry_count {
                    let entry = &explorer.entries[i];
                    if entry.active && entry.parent == explorer.current_dir {
                        if let Ok(name) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                            if len + name.len() + 3 > 58 {
                                self.add_line(core::str::from_utf8(&line_buf[..len]).unwrap_or(""));
                                line_buf = [0u8; 60];
                                len = 0;
                            }
                            len += copy_slice(&mut line_buf[len..], name.as_bytes());
                            if entry.is_dir {
                                len += copy_slice(&mut line_buf[len..], b"/  ");
                            } else {
                                len += copy_slice(&mut line_buf[len..], b"   ");
                            }
                        }
                    }
                }
                if len > 0 {
                    self.add_line(core::str::from_utf8(&line_buf[..len]).unwrap_or(""));
                }
            }
            "cd" => {
                if arg.is_empty() {
                    let mut explorer = crate::EXPLORER.lock();
                    explorer.current_dir = 0;
                } else if arg == ".." {
                    let mut explorer = crate::EXPLORER.lock();
                    if explorer.current_dir != 0 {
                        let parent = explorer.entries[explorer.current_dir].parent;
                        explorer.current_dir = parent;
                    }
                } else {
                    let mut explorer = crate::EXPLORER.lock();
                    let mut found = false;
                    for i in 1..explorer.entry_count {
                        let entry = &explorer.entries[i];
                        if entry.active && entry.is_dir && entry.parent == explorer.current_dir {
                            if let Ok(name) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                                if name == arg {
                                    explorer.current_dir = i;
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found {
                        self.add_line("Error: directory not found");
                    }
                }
            }
            "cat" => {
                if arg.is_empty() {
                    self.add_line("Usage: cat <filename>");
                } else {
                    let explorer = crate::EXPLORER.lock();
                    let mut found = false;
                    for i in 1..explorer.entry_count {
                        let entry = &explorer.entries[i];
                        if entry.active && !entry.is_dir && entry.parent == explorer.current_dir {
                            if let Ok(name) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                                if name == arg {
                                    if let Ok(content) = core::str::from_utf8(&entry.content[..entry.content_len]) {
                                        // print in chunks of 58 chars
                                        let mut start = 0;
                                        while start < content.len() {
                                            let end = if start + 58 > content.len() { content.len() } else { start + 58 };
                                            self.add_line(&content[start..end]);
                                            start = end;
                                        }
                                    }
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found {
                        self.add_line("Error: file not found");
                    }
                }
            }
            "mkdir" => {
                if arg.is_empty() {
                    self.add_line("Usage: mkdir <dirname>");
                } else {
                    let mut explorer = crate::EXPLORER.lock();
                    if explorer.entry_count < 24 {
                        let idx = explorer.entry_count;
                        let copy_len = if arg.len() > 18 { 18 } else { arg.len() };
                        explorer.entries[idx].name = [0; 20];
                        explorer.entries[idx].name[..copy_len].copy_from_slice(&arg.as_bytes()[..copy_len]);
                        explorer.entries[idx].name_len = copy_len;
                        explorer.entries[idx].is_dir = true;
                        explorer.entries[idx].parent = explorer.current_dir;
                        explorer.entries[idx].active = true;
                        explorer.entry_count += 1;
                        self.add_line("Created directory successfully.");
                    } else {
                        self.add_line("Error: Virtual Filesystem full.");
                    }
                }
            }
            "touch" => {
                if arg.is_empty() {
                    self.add_line("Usage: touch <filename>");
                } else {
                    let mut explorer = crate::EXPLORER.lock();
                    if explorer.entry_count < 24 {
                        let idx = explorer.entry_count;
                        let copy_len = if arg.len() > 18 { 18 } else { arg.len() };
                        explorer.entries[idx].name = [0; 20];
                        explorer.entries[idx].name[..copy_len].copy_from_slice(&arg.as_bytes()[..copy_len]);
                        explorer.entries[idx].name_len = copy_len;
                        explorer.entries[idx].is_dir = false;
                        explorer.entries[idx].parent = explorer.current_dir;
                        explorer.entries[idx].active = true;
                        explorer.entries[idx].content = *b"(empty file)                                                                    ";
                        explorer.entries[idx].content_len = 12;
                        explorer.entry_count += 1;
                        self.add_line("Created file successfully.");
                    } else {
                        self.add_line("Error: Virtual Filesystem full.");
                    }
                }
            }
            "rm" => {
                if arg.is_empty() {
                    self.add_line("Usage: rm <filename>");
                } else {
                    let mut explorer = crate::EXPLORER.lock();
                    let mut found = false;
                    for i in 1..explorer.entry_count {
                        let entry = &explorer.entries[i];
                        if entry.active && entry.parent == explorer.current_dir {
                            if let Ok(name) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                                if name == arg {
                                    explorer.entries[i].active = false;
                                    self.add_line("Removed entry successfully.");
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found {
                        self.add_line("Error: entry not found.");
                    }
                }
            }
            "matrix" => {
                self.add_line("Running Matrix code cascade... Press ESC to exit.");
                unsafe {
                    let mut streams = [0u8; 40];
                    let mut lengths = [0u8; 40];
                    let mut speeds = [0u8; 40];
                    
                    // Seed columns
                    for col in 0..40 {
                        streams[col] = (col * 7) as u8; // starting Y
                        lengths[col] = (5 + (col % 8)) as u8;
                        speeds[col] = (1 + (col % 3)) as u8;
                    }

                    VGA.draw_rect(0, 0, 320, 200, 0); // clear screen
                    
                    let mut frame = 0;
                    loop {
                        // Poll escape or key press to exit
                        if let Some(key) = crate::keyboard::get_key() {
                            match key {
                                crate::keyboard::DecodedKey::Escape => break,
                                _ => {}
                            }
                        }

                        // Draw columns
                        for col in 0..40 {
                            let cx = col * 8;
                            let cy = streams[col] as usize;
                            
                            // Erase old top trailing character
                            let erase_y = if cy >= (lengths[col] as usize * 8) {
                                cy - (lengths[col] as usize * 8)
                            } else {
                                0
                            };
                            VGA.draw_rect(cx, erase_y, 8, 8, 0);

                            // Draw trail Y
                            for i in 0..(lengths[col] as usize) {
                                let y_val = cy.saturating_sub(i * 8);
                                if y_val < 190 {
                                    let char_idx = (frame + col + i) % 15;
                                    let char_val = match char_idx {
                                        0 => '0', 1 => '1', 2 => 'A', 3 => 'K', 4 => 'X', 
                                        5 => 'Z', 6 => '7', 7 => '3', 8 => '#', 9 => '%',
                                        _ => '*',
                                    };
                                    let color = if i == 0 { 15 } else if i < 3 { 10 } else { 2 }; // White head, Bright Green body, Dark Green tail
                                    VGA.draw_char(cx, y_val, char_val, color);
                                }
                            }

                            // Advance Y
                            streams[col] = ((streams[col] as usize + speeds[col] as usize) % 200) as u8;
                        }

                        VGA.swap_buffers();
                        
                        // regulate speed
                        for _ in 0..40_000 {
                            core::arch::asm!("nop");
                        }
                        frame += 1;
                    }

                    // Restore normal screen state
                    VGA.draw_rect(0, 0, 320, 200, 0);
                }
            }
            "maximise" | "maximize" => {
                unsafe {
                    crate::IS_MAXIMISED = true;
                }
                self.add_line("Window maximized successfully.");
            }
            "minimise" | "minimize" => {
                app_switch = Some("exit");
            }
            "cowsay" => {
                if arg.is_empty() {
                    self.add_line("Usage: cowsay <message>");
                } else {
                    let mut bubble = [0u8; 60];
                    bubble[0] = b'<';
                    bubble[1] = b' ';
                    let full_msg = if cmd_str.len() > 7 { &cmd_str[7..] } else { arg };
                    let show_len = if full_msg.len() > 36 { 36 } else { full_msg.len() };
                    
                    self.add_line("  _____________________________________");
                    bubble[2..(2 + show_len)].copy_from_slice(&full_msg.as_bytes()[..show_len]);
                    bubble[2 + show_len] = b' ';
                    bubble[3 + show_len] = b'>';
                    self.add_line(core::str::from_utf8(&bubble[..(4 + show_len)]).unwrap_or(""));
                    self.add_line("  -------------------------------------");
                    self.add_line("        \\   ^__^");
                    self.add_line("         \\  (oo)\\_______");
                    self.add_line("            (__)\\       )\\/\\");
                    self.add_line("                ||----w |");
                    self.add_line("                ||     ||");
                }
            }
            "sing" => {
                self.add_line("Playing arpeggio melody on PC speaker... 🎶");
                unsafe {
                    let notes = [261, 329, 392, 523, 392, 523];
                    for &note in notes.iter() {
                        crate::sound::play_tone(note);
                        for _ in 0..12_000 { core::arch::asm!("nop"); }
                    }
                    crate::sound::stop_speaker();
                }
                self.add_line("Melody completed.");
            }
            "fortune" => {
                let fortunes = [
                    "You will build the ultimate Rust microkernel in 3 days.",
                    "A green character stream is in your near future.",
                    "You are superuser (uloos-root). Use this power wisely.",
                    "The PC speaker is humming a happy tune for you today.",
                    "Avoid SSE float instructions on bare-metal x86 today.",
                    "VGA Mode 13h is 256 colors of absolute retro beauty.",
                ];
                let (_, _, s) = crate::get_rtc_time();
                let idx = (s as usize) % fortunes.len();
                self.add_line(fortunes[idx]);
            }
            "doom" => app_switch = Some("doom"),
            "office" => app_switch = Some("office"),
            "explorer" => app_switch = Some("explorer"),
            "exit" => app_switch = Some("exit"),
            _ => {
                self.add_line("Error: command not found. Type 'help'");
            }
        }

        self.input_len = 0;
        app_switch
    }
}

fn copy_slice(dest: &mut [u8], src: &[u8]) -> usize {
    let len = if src.len() > dest.len() { dest.len() } else { src.len() };
    dest[..len].copy_from_slice(&src[..len]);
    len
}
