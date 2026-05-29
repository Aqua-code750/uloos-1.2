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
    pub url: &'static str,
    pub page_content: [&'static str; 4],
}

impl WebBrowser {
    pub const fn new() -> Self {
        WebBrowser {
            url: "https://google.com/textmode",
            page_content: [
                "Google Text-Mode Search Engine",
                "----------------------------------------------",
                "  1. Nightly Rust docs (rust-lang.org)",
                "  2. QEMU Emulator manuals (qemu.org)",
            ],
        }
    }

    pub fn draw(&self) {
        // High white browser background
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Address bar
        VGA.draw_rect(14, 32, 292, 14, 7);
        VGA.draw_string(16, 35, "URL: ", 8);
        VGA.draw_string(50, 35, self.url, 0);

        // Page content
        for idx in 0..4 {
            VGA.draw_string(16, 60 + idx * 16, self.page_content[idx], 0);
        }
    }
}
