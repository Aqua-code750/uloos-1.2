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
        VGA.draw_string(16, 35, "< > R", 8);
        VGA.draw_rect(50, 32, 1, 14, 8);
        VGA.draw_string(55, 35, "URL: ", 8);
        
        let mut typed_url = "https://google.com/sandbox";
        if let Ok(u_str) = core::str::from_utf8(&self.url[..self.url_len]) {
            VGA.draw_string(85, 35, u_str, 0);
            typed_url = u_str;
            VGA.draw_rect(85 + self.url_len * 8, 41, 6, 2, 1);
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

        let is_github = typed_url.contains("git");
        let is_youtube = typed_url.contains("you");
        let is_wiki = typed_url.contains("wiki");

        if is_github {
            VGA.draw_rect(14, 48, 292, 12, 8);
            VGA.draw_string(18, 50, "GitHub - Aqua-code750/uloos-1.2", 15);

            VGA.draw_string(16, 68, "Repository: uloos-1.2 [Public]", 1);
            VGA.draw_string(16, 82, "Language: Rust 100% | Stars: 1,200", 2);

            VGA.draw_rect(16, 98, 288, 38, 7);
            VGA.draw_string(20, 102, "#![no_std]", 4);
            VGA.draw_string(20, 114, "fn _start() -> ! { loop {} }", 1);
            VGA.draw_string(20, 126, "// Bare-metal QEMU bootloader", 8);
            
            VGA.draw_string(16, 148, "Press [M] to cycle presets", 8);
        } else if is_youtube {
            VGA.draw_rect(14, 48, 292, 12, 12);
            VGA.draw_string(18, 50, "YouTube Studio Player", 15);

            VGA.draw_rect(20, 68, 120, 65, 0);
            VGA.draw_rect(75, 95, 10, 10, 12);
            VGA.draw_string(22, 122, "--> playing clip", 10);

            VGA.draw_string(148, 68, "Rust OS Tutorial", 1);
            VGA.draw_string(148, 82, "By: Aqua-code750", 8);
            VGA.draw_string(148, 96, "Views: 2.4M", 8);
            
            VGA.draw_rect(148, 115, 60, 4, 8);
            VGA.draw_rect(148, 115, 45, 4, 10);

            VGA.draw_string(16, 148, "Press [M] to cycle presets", 8);
        } else if is_wiki {
            VGA.draw_rect(14, 48, 292, 12, 7);
            VGA.draw_string(18, 50, "Wikipedia - The Free Encyclopedia", 0);

            VGA.draw_string(16, 68, "Hobby Operating System (OS)", 1);
            VGA.draw_string(16, 82, "---------------------------", 8);
            VGA.draw_string(16, 96, "An OS created from scratch", 0);
            VGA.draw_string(16, 110, "designed to study kernel design,", 0);
            VGA.draw_string(16, 124, "x86 paging, and custom BIOS.", 0);

            VGA.draw_string(16, 148, "Press [M] to cycle presets", 8);
        } else {
            VGA.draw_string(16, 56, "Google search engine mode", 1);
            VGA.draw_string(16, 72, "-----------------------------", 8);
            
            VGA.draw_string(16, 88, "Try typing one of these URLs:", 8);
            VGA.draw_string(16, 104, "- github.com  (Load Git repository)", 2);
            VGA.draw_string(16, 120, "- youtube.com (Load video player)", 12);
            VGA.draw_string(16, 136, "- wikipedia.org (Load Wikipedia page)", 1);

            VGA.draw_string(16, 150, "Press [M] Switch Mode | Type URL + [Enter]", 8);
        }
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
