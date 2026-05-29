use crate::vga_driver::VGA;

// ==========================================
// FILE EXPLORER
// ==========================================
pub struct FileExplorer {
    pub files: [(&'static str, &'static str); 4],
    pub selected: usize,
}

impl FileExplorer {
    pub const fn new() -> Self {
        FileExplorer {
            files: [
                ("Welcome_UloOS.txt", "Welcome to UloOS Minimal Graphics OS! Complete core loaded successfully."),
                ("system_config.cfg", "QEMU_MEMORY=8192\nBOOT_MODE=VGA_GRAPHICS\nINTERRUPT=POLLING"),
                ("office_notes.txt", "Verify spreadsheet totals inside UloNumbers cells."),
                ("kernel_manifest.sys", "nightly-rustc-2026-x86_64-uloos-binary"),
            ],
            selected: 0,
        }
    }

    pub fn draw(&self) {
        // Light gray workspace background
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Sidebar divider line
        VGA.draw_rect(100, 28, 2, 144, 7);

        // Sidebar headers
        VGA.draw_string(16, 34, "Folders", 1);
        VGA.draw_string(16, 48, "[C:] Drive", 0);
        VGA.draw_string(24, 62, "- Home", 8);
        VGA.draw_string(24, 76, "- System", 8);

        // File listings in the main pane
        VGA.draw_string(106, 34, "Files List:", 1);
        for idx in 0..4 {
            let file = self.files[idx];
            let is_sel = idx == self.selected;
            let bg_col = if is_sel { 1 } else { 15 }; // Blue background if selected
            let fg_col = if is_sel { 15 } else { 0 }; // White text if selected, else black
            
            VGA.draw_rect(106, 50 + idx * 14, 190, 12, bg_col);
            VGA.draw_string(108, 52 + idx * 14, file.0, fg_col);
        }

        // Preview Pane at the bottom of the window
        VGA.draw_rect(106, 115, 196, 2, 7);
        VGA.draw_string(106, 120, "File Preview:", 1);
        let active_content = self.files[self.selected].1;
        // Print content in compact preview slices
        VGA.draw_string(106, 134, "Content:", 8);
        if active_content.len() > 24 {
            VGA.draw_string(106, 146, &active_content[..24], 0);
        } else {
            VGA.draw_string(106, 146, active_content, 0);
        }
    }

    pub fn move_down(&mut self) {
        if self.selected < 3 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = 3;
        }
    }
}

// ==========================================
// WEB BROWSER
// ==========================================
pub struct WebBrowser {
    pub url: [u8; 120],
    pub url_len: usize,
    pub current_mode: usize, // 0 = Sandbox, 1 = Firefox Proxy, 2 = Chrome
}

impl WebBrowser {
    pub const fn new() -> Self {
        let default_url = "https://google.com/sandbox";
        let mut url = [0; 120];
        let mut idx = 0;
        while idx < default_url.len() && idx < 120 {
            url[idx] = default_url.as_bytes()[idx];
            idx += 1;
        }
        WebBrowser {
            url,
            url_len: idx,
            current_mode: 0,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        VGA.draw_rect(14, 32, 292, 14, 7);
        VGA.draw_string(16, 35, "URL: ", 8);
        
        if let Ok(u_str) = core::str::from_utf8(&self.url[..self.url_len]) {
            VGA.draw_string(50, 35, u_str, 0);
            VGA.draw_rect(50 + self.url_len * 8, 41, 6, 2, 1);
        }

        if self.current_mode == 0 {
            VGA.draw_rect(240, 33, 62, 12, 9);
            VGA.draw_string(244, 35, "[Sandbox]", 15);
        } else if self.current_mode == 1 {
            VGA.draw_rect(240, 33, 62, 12, 12);
            VGA.draw_string(244, 35, "[Firefox]", 15);
        } else {
            VGA.draw_rect(240, 33, 62, 12, 2);
            VGA.draw_string(244, 35, "[ Chrome]", 15);
        }

        if self.current_mode == 0 {
            VGA.draw_string(16, 56, "Holograph Sandbox Browser", 1);
            VGA.draw_string(16, 72, "-----------------------------", 8);
            VGA.draw_string(16, 88, "* Star Aqua-code750/uloos-1.2", 0);
            VGA.draw_string(16, 104, "* Read Hobby OS Wikipedia pages", 0);
            VGA.draw_string(16, 120, "* Test keyboard beeps in Music app", 0);
        } else if self.current_mode == 1 {
            VGA.draw_string(16, 56, "Firefox Unrestricted Proxy Dev", 12);
            VGA.draw_string(16, 72, "-----------------------------", 8);
            VGA.draw_string(16, 88, "* Bypassing CORS controls...", 10);
            VGA.draw_string(16, 104, "  CroxyProxy live websocket: OK", 0);
            VGA.draw_string(16, 120, "  Unrestricted real-world web!", 2);
        } else {
            VGA.draw_string(16, 56, "Google Chrome Real Search Mode", 2);
            VGA.draw_string(16, 72, "-----------------------------", 8);
            VGA.draw_string(16, 88, "Search query parser: Active", 0);
            VGA.draw_string(16, 104, "Targeting google.com/search?igu=1", 0);
            VGA.draw_string(16, 120, "Searching code repositories...", 8);
        }

        VGA.draw_string(16, 150, "Press [M] Switch Mode | Type URL + [Enter]", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        if key == 'm' || key == 'M' {
            self.current_mode = (self.current_mode + 1) % 3;
            let default_url = if self.current_mode == 0 {
                "https://google.com/sandbox"
            } else if self.current_mode == 1 {
                "https://www.croxyproxy.com/"
            } else {
                "https://google.com/search?igu=1"
            };
            self.url_len = 0;
            for idx in 0..default_url.len() {
                if idx < 120 {
                    self.url[idx] = default_url.as_bytes()[idx];
                    self.url_len += 1;
                }
            }
        } else if self.url_len < 22 {
            self.url[self.url_len] = key as u8;
            self.url_len += 1;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.url_len > 0 {
            self.url_len -= 1;
        }
    }
}
