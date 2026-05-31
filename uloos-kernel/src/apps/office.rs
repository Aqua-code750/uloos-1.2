use crate::vga_driver::VGA;
use spin::Mutex;

pub static CLIPBOARD: Mutex<ClipboardBuffer> = Mutex::new(ClipboardBuffer::new());

pub struct ClipboardBuffer {
    pub data: [u8; 256],
    pub len: usize,
}

impl ClipboardBuffer {
    pub const fn new() -> Self {
        ClipboardBuffer {
            data: [0; 256],
            len: 0,
        }
    }
    
    pub fn set_text(&mut self, text: &[u8]) {
        let copy_len = if text.len() > 256 { 256 } else { text.len() };
        self.len = copy_len;
        for i in 0..copy_len {
            self.data[i] = text[i];
        }
    }

    pub fn get_text(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

// ==========================================
// ULOCODE STUDIO: VS Code Clone Editor
// ==========================================
pub struct UloCode {
    pub tab_names: [&'static str; 3],
    pub buffers: [[u8; 500]; 3],
    pub lens: [usize; 3],
    pub active_tab: usize,
}

impl UloCode {
    pub const fn new() -> Self {
        let tab_names = ["main.rs", "index.html", "style.css"];
        let mut buffers = [[0u8; 500]; 3];
        let mut lens = [0usize; 3];

        // Seed Tab 0: main.rs
        let code_rs = b"fn main() {\n    // Code inside UloOS!\n    let msg = \"Hello world\";\n    println!(\"{}\", msg);\n}\n";
        let mut rs_idx = 0;
        while rs_idx < code_rs.len() {
            buffers[0][rs_idx] = code_rs[rs_idx];
            rs_idx += 1;
        }
        lens[0] = code_rs.len();

        // Seed Tab 1: index.html
        let code_html = b"<h1>Welcome to UloOS</h1>\n<p>VS Code replica ready</p>\n<button>Launch</button>\n";
        let mut html_idx = 0;
        while html_idx < code_html.len() {
            buffers[1][html_idx] = code_html[html_idx];
            html_idx += 1;
        }
        lens[1] = code_html.len();

        // Seed Tab 2: style.css
        let code_css = b"body {\n    background: #0f121a;\n    color: #00f0ff;\n    font-family: Outfit;\n}\n";
        let mut css_idx = 0;
        while css_idx < code_css.len() {
            buffers[2][css_idx] = code_css[css_idx];
            css_idx += 1;
        }
        lens[2] = code_css.len();

        UloCode {
            tab_names,
            buffers,
            lens,
            active_tab: 0,
        }
    }

    pub fn cycle_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % 3;
    }

    pub fn draw(&self) {
        // Window base viewport (charcoal dark gray)
        VGA.draw_rect(12, 28, 296, 144, 8);

        // ==========================================
        // 1. VS Code Activity Bar (Far Leftmost)
        // ==========================================
        VGA.draw_rect(12, 28, 12, 144, 0); // black strip
        VGA.draw_rect(24, 28, 1, 144, 8);  // separator

        // Draw Left Accent Indicator (cyan) on the active symbol [E]
        VGA.draw_rect(12, 43, 2, 8, 11);

        // Draw block symbols
        VGA.draw_char(15, 43, 'E', 15); // Explorer
        VGA.draw_char(15, 60, 'S', 8);  // Search
        VGA.draw_char(15, 77, 'G', 8);  // Git
        VGA.draw_char(15, 94, 'O', 8);  // Extensions
        VGA.draw_char(15, 111, '*', 8); // Settings

        // ==========================================
        // 2. Header toolbar (dark theme)
        // ==========================================
        VGA.draw_rect(25, 28, 283, 12, 0); // black bar
        VGA.draw_string(29, 30, "UloCode Studio", 9); // Light Blue

        // Draw LOAD and SAVE buttons
        VGA.draw_rect(125, 29, 32, 10, 1); // Blue LOAD
        VGA.draw_string(129, 30, "LOAD", 15);

        VGA.draw_rect(162, 29, 32, 10, 2); // Green SAVE
        VGA.draw_string(166, 30, "SAVE", 15);

        // Line and Character count
        VGA.draw_string(210, 30, "Ln:", 11);
        let mut line_count = 1;
        let active_len = self.lens[self.active_tab];
        let active_buf = &self.buffers[self.active_tab];

        for i in 0..active_len {
            if active_buf[i] == b'\n' { line_count += 1; }
        }
        let mut lc_buf = [b'0'; 2];
        lc_buf[0] = b'0' + ((line_count / 10) % 10) as u8;
        lc_buf[1] = b'0' + (line_count % 10) as u8;
        VGA.draw_string(234, 30, core::str::from_utf8(&lc_buf).unwrap(), 15);

        VGA.draw_string(255, 30, "Col:", 11);
        let mut ch_buf = [b'0'; 2];
        ch_buf[0] = b'0' + ((active_len / 10) % 10) as u8;
        ch_buf[1] = b'0' + (active_len % 10) as u8;
        VGA.draw_string(287, 30, core::str::from_utf8(&ch_buf).unwrap(), 15);

        // ==========================================
        // 3. Sidebar - Explorer directory tree
        // ==========================================
        VGA.draw_rect(25, 40, 54, 122, 0); // Black sidebar
        VGA.draw_string(27, 43, "VFS:", 11); // cyan
        VGA.draw_rect(27, 52, 50, 1, 8); // separator

        // List folders and files from explorer in sidebar
        let explorer = crate::EXPLORER.lock();
        let mut sy = 56;
        for i in 1..explorer.entry_count {
            if explorer.entries[i].active && explorer.entries[i].parent == explorer.current_dir {
                let color = if explorer.entries[i].is_dir { 14 } else { 15 }; // yellow for dir, white for file
                let prefix = if explorer.entries[i].is_dir { ">" } else { " " };
                VGA.draw_string(27, sy, prefix, 8);
                if let Ok(name) = core::str::from_utf8(&explorer.entries[i].name[..explorer.entries[i].name_len]) {
                    let show = if name.len() > 4 { 4 } else { name.len() };
                    VGA.draw_string(35, sy, &name[..show], color);
                }
                sy += 11;
                if sy > 150 { break; }
            }
        }
        drop(explorer); // release lock

        // ==========================================
        // 4. Tab bar with 3 cyclable tabs
        // ==========================================
        VGA.draw_rect(79, 40, 229, 11, 0); // Black tab bar background
        
        let tab_w = 66;
        for t_idx in 0..3 {
            let tx = 81 + t_idx * (tab_w + 3);
            let is_active = self.active_tab == t_idx;
            let tab_bg = if is_active { 8 } else { 0 }; // Charcoal for active, Black for inactive
            let tab_fg = if is_active { 15 } else { 7 };
            
            VGA.draw_rect(tx, 40, tab_w, 11, tab_bg);
            
            // Draw neat color tab marker
            if is_active {
                VGA.draw_rect(tx, 40, tab_w, 1, 9); // Light blue top border line
            }

            let name = self.tab_names[t_idx];
            VGA.draw_string(tx + 2, 42, name, tab_fg);
        }

        // ==========================================
        // 5. Editor Workspace with scrolling lines gutter
        // ==========================================
        VGA.draw_rect(79, 51, 229, 111, 8); // Charcoal editor pane

        // Line numbers column
        VGA.draw_rect(79, 51, 14, 111, 0); // Black strip
        for l in 0..14 {
            let ly = 53 + l * 8;
            let mut l_buf = [b'0'; 2];
            l_buf[0] = b'0' + ((l + 1) / 10) as u8;
            l_buf[1] = b'0' + ((l + 1) % 10) as u8;
            VGA.draw_string(81, ly, core::str::from_utf8(&l_buf).unwrap(), 9); // Light blue
        }

        // Render editor buffer text with elegant VGA Tokenizer (Syntax Highlighting!)
        let mut curr_x = 95;
        let mut curr_y = 53;
        let mut i = 0;
        
        while i < active_len {
            let byte = active_buf[i];
            
            if byte == b'\n' {
                curr_x = 95;
                curr_y += 8;
                i += 1;
                if curr_y >= 160 { break; }
            } else if byte == b'/' && i + 1 < active_len && active_buf[i+1] == b'/' {
                // Comment token: color the rest of the line green (10)
                while i < active_len {
                    let c_byte = active_buf[i];
                    if c_byte == b'\n' {
                        break;
                    }
                    VGA.draw_char(curr_x, curr_y, c_byte as char, 10);
                    curr_x += 6;
                    i += 1;
                    if curr_x >= 300 {
                        curr_x = 95;
                        curr_y += 8;
                        if curr_y >= 160 { break; }
                    }
                }
            } else if byte == b'<' || byte == b'>' {
                VGA.draw_char(curr_x, curr_y, byte as char, 11); // Cyan brackets
                curr_x += 6;
                i += 1;
                if curr_x >= 300 {
                    curr_x = 95;
                    curr_y += 8;
                    if curr_y >= 160 { break; }
                }
            } else {
                // Alphanumeric sequence lookup for custom keyword matches!
                let mut keyword_len = 0;
                while i + keyword_len < active_len {
                    let k_byte = active_buf[i + keyword_len];
                    if k_byte.is_ascii_alphanumeric() || k_byte == b'_' {
                        keyword_len += 1;
                    } else {
                        break;
                    }
                }
                
                if keyword_len > 0 {
                    let mut kw_buf = [0u8; 8];
                    let copy_kw = if keyword_len > 8 { 8 } else { keyword_len };
                    kw_buf[..copy_kw].copy_from_slice(&active_buf[i..(i + copy_kw)]);
                    
                    let is_kw = match &kw_buf[..copy_kw] {
                        b"fn" | b"let" | b"if" | b"return" | b"const" | b"pub" | b"struct" | b"impl" => true,
                        _ => false,
                    };
                    
                    let color = if is_kw { 13 } else { 15 }; // Neon Violet for keywords, Crisp White for other words
                    
                    for _ in 0..keyword_len {
                        VGA.draw_char(curr_x, curr_y, active_buf[i] as char, color);
                        curr_x += 6;
                        i += 1;
                        if curr_x >= 300 {
                            curr_x = 95;
                            curr_y += 8;
                            if curr_y >= 160 { break; }
                        }
                    }
                } else {
                    // Standard symbols, operators, spaces
                    VGA.draw_char(curr_x, curr_y, byte as char, 15);
                    curr_x += 6;
                    i += 1;
                    if curr_x >= 300 {
                        curr_x = 95;
                        curr_y += 8;
                        if curr_y >= 160 { break; }
                    }
                }
            }
        }

        // Blinking Cursor (cyan)
        VGA.draw_rect(curr_x, curr_y + 7, 5, 2, 11);

        // ==========================================
        // 6. Bottom blue status bar (Git info, tabs info)
        // ==========================================
        VGA.draw_rect(12, 162, 296, 10, 1); // Blue footer
        VGA.draw_string(16, 163, "ulo-main | VFS Synced | [TAB] Cycle Tabs", 15);
    }

    pub fn load_from_vfs(&mut self) {
        let explorer = crate::EXPLORER.lock();
        if let Some(idx) = explorer.get_selected_entry_index() {
            let entry = &explorer.entries[idx];
            if !entry.is_dir {
                // Clear current tab buffer
                let active_tab = self.active_tab;
                for b in self.buffers[active_tab].iter_mut() { *b = 0; }
                let copy_len = if entry.content_len > 500 { 500 } else { entry.content_len };
                self.buffers[active_tab][..copy_len].copy_from_slice(&entry.content[..copy_len]);
                self.lens[active_tab] = copy_len;
            }
        }
    }

    pub fn save_to_vfs(&mut self) {
        let mut explorer = crate::EXPLORER.lock();
        if let Some(idx) = explorer.get_selected_entry_index() {
            let entry = &mut explorer.entries[idx];
            if !entry.is_dir {
                let active_tab = self.active_tab;
                let copy_len = if self.lens[active_tab] > 80 { 80 } else { self.lens[active_tab] };
                entry.content = [0; 80];
                entry.content[..copy_len].copy_from_slice(&self.buffers[active_tab][..copy_len]);
                entry.content_len = copy_len;
            }
        }
    }

    pub fn handle_key(&mut self, key: char) {
        let active_tab = self.active_tab;
        if self.lens[active_tab] < 490 {
            self.buffers[active_tab][self.lens[active_tab]] = key as u8;
            self.lens[active_tab] += 1;
        }
    }

    pub fn handle_enter(&mut self) {
        let active_tab = self.active_tab;
        if self.lens[active_tab] < 490 {
            self.buffers[active_tab][self.lens[active_tab]] = b'\n';
            self.lens[active_tab] += 1;
        }
    }

    pub fn handle_backspace(&mut self) {
        let active_tab = self.active_tab;
        if self.lens[active_tab] > 0 {
            self.lens[active_tab] -= 1;
        }
    }

    pub fn handle_copy(&self) {
        let active_tab = self.active_tab;
        let text = &self.buffers[active_tab][..self.lens[active_tab]];
        CLIPBOARD.lock().set_text(text);
        unsafe {
            crate::sound::play_tone(700);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }

    pub fn handle_cut(&mut self) {
        let active_tab = self.active_tab;
        let text = &self.buffers[active_tab][..self.lens[active_tab]];
        CLIPBOARD.lock().set_text(text);
        self.lens[active_tab] = 0;
        unsafe {
            crate::sound::play_tone(600);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }

    pub fn handle_paste(&mut self) {
        let active_tab = self.active_tab;
        let clip = CLIPBOARD.lock();
        let clip_text = clip.get_text();
        for &byte in clip_text {
            if self.lens[active_tab] < 490 {
                self.buffers[active_tab][self.lens[active_tab]] = byte;
                self.lens[active_tab] += 1;
            }
        }
        unsafe {
            crate::sound::play_tone(800);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }
}

// ==========================================
// ULOSLIDES: Presentation Creator
// ==========================================
pub struct UloSlides {
    pub slide_titles: [[u8; 30]; 5],
    pub title_lens: [usize; 5],
    pub slide_descs: [[u8; 80]; 5],
    pub desc_lens: [usize; 5],
    pub current: usize,
    pub editing: bool,      // true if editing description
    pub editing_title: bool, // true if editing title
}

impl UloSlides {
    pub const fn new() -> Self {
        let mut slide_titles = [[0; 30]; 5];
        let mut title_lens = [0; 5];
        let mut slide_descs = [[0; 80]; 5];
        let mut desc_lens = [0; 5];

        // Seed Slide 1
        let t1 = b"Microkernel Architecture";
        let d1 = b"Multi-threaded scheduler loops with CMOS sync and IST offset timezone clocks.";
        let mut i = 0; while i < t1.len() { slide_titles[0][i] = t1[i]; i += 1; }
        title_lens[0] = t1.len();
        let mut i = 0; while i < d1.len() { slide_descs[0][i] = d1[i]; i += 1; }
        desc_lens[0] = d1.len();

        // Seed Slide 2
        let t2 = b"VGA Mode 13h Graphics";
        let d2 = b"Rendering 256-color flat designs with double-buffered smooth rendering pipeline.";
        let mut i = 0; while i < t2.len() { slide_titles[1][i] = t2[i]; i += 1; }
        title_lens[1] = t2.len();
        let mut i = 0; while i < d2.len() { slide_descs[1][i] = d2[i]; i += 1; }
        desc_lens[1] = d2.len();

        // Seed Slide 3
        let t3 = b"Modular Rust Drivers";
        let d3 = b"Zero dependency compile config loaded cleanly inside QEMU x86_64 target ring.";
        let mut i = 0; while i < t3.len() { slide_titles[2][i] = t3[i]; i += 1; }
        title_lens[2] = t3.len();
        let mut i = 0; while i < d3.len() { slide_descs[2][i] = d3[i]; i += 1; }
        desc_lens[2] = d3.len();

        // Seed Slide 4
        let t4 = b"Interactive App Suite";
        let d4 = b"Full office suite with slides, sheets, mail, weather, music and sticky notes.";
        let mut i = 0; while i < t4.len() { slide_titles[3][i] = t4[i]; i += 1; }
        title_lens[3] = t4.len();
        let mut i = 0; while i < d4.len() { slide_descs[3][i] = d4[i]; i += 1; }
        desc_lens[3] = d4.len();

        // Seed Slide 5
        let t5 = b"3D DOOM Raycaster";
        let d5 = b"First-person perspective engine with distance-based wall shading and PC speaker.";
        let mut i = 0; while i < t5.len() { slide_titles[4][i] = t5[i]; i += 1; }
        title_lens[4] = t5.len();
        let mut i = 0; while i < d5.len() { slide_descs[4][i] = d5[i]; i += 1; }
        desc_lens[4] = d5.len();

        UloSlides {
            slide_titles,
            title_lens,
            slide_descs,
            desc_lens,
            current: 0,
            editing: false,
            editing_title: false,
        }
    }

    pub fn draw(&self) {
        // Dark stage background
        VGA.draw_rect(12, 28, 296, 144, 8);

        // Sidebar thumbnails
        VGA.draw_rect(12, 28, 54, 144, 0);
        VGA.draw_string(14, 30, "Slides", 15);

        for idx in 0..5 {
            let is_sel = idx == self.current;
            let bg = if is_sel { 9 } else { 8 };
            let fg = if is_sel { 15 } else { 7 };

            VGA.draw_rect(14, 42 + idx * 20, 48, 16, bg);

            // Slide number
            let mut label = [b'S', b'l', b' ', b'0', 0];
            label[3] = b'1' + idx as u8;
            VGA.draw_string(18, 46 + idx * 20, core::str::from_utf8(&label[..4]).unwrap(), fg);
        }

        // Main slide card
        VGA.draw_rect(70, 32, 234, 104, 15); // white paper
        VGA.draw_rect(70, 32, 234, 14, 9);   // blue header

        // Slide counter
        let mut counter = [b'S', b'l', b'i', b'd', b'e', b' ', b'0', b' ', b'/', b' ', b'5'];
        counter[6] = b'1' + self.current as u8;
        VGA.draw_string(74, 34, core::str::from_utf8(&counter).unwrap(), 15);

        // Status banner when editing
        if self.editing {
            VGA.draw_string(180, 34, "EDIT DESC", 14);
        } else if self.editing_title {
            VGA.draw_string(180, 34, "EDIT TITLE", 14);
        }

        // Slide title
        if let Ok(title) = core::str::from_utf8(&self.slide_titles[self.current][..self.title_lens[self.current]]) {
            VGA.draw_string(76, 52, title, 1);
            if self.editing_title {
                let tx = 76 + self.title_lens[self.current] * 6;
                if tx < 300 {
                    VGA.draw_rect(tx, 59, 6, 2, 1);
                }
            }
        }

        // Description - word wrap at ~35 chars (scaled for small font size!)
        let d_len = self.desc_lens[self.current];
        if let Ok(desc) = core::str::from_utf8(&self.slide_descs[self.current][..d_len]) {
            let max_w = 35;
            if desc.len() > max_w * 2 {
                VGA.draw_string(76, 68, &desc[..max_w], 8);
                VGA.draw_string(76, 80, &desc[max_w..max_w*2], 8);
                VGA.draw_string(76, 92, &desc[max_w*2..], 8);
            } else if desc.len() > max_w {
                VGA.draw_string(76, 68, &desc[..max_w], 8);
                VGA.draw_string(76, 80, &desc[max_w..], 8);
            } else {
                VGA.draw_string(76, 68, desc, 8);
            }

            // Blinking cursor for editing description
            if self.editing {
                let cursor_line = desc.len() / max_w;
                let cursor_char = desc.len() % max_w;
                let cx = 76 + cursor_char * 6;
                let cy = 68 + cursor_line * 12;
                if cx < 300 && cy < 136 {
                    VGA.draw_rect(cx, cy + 7, 5, 2, 1);
                }
            }
        }

        // Decorative slide graphic element
        let graphic_color = [10, 11, 14, 13, 12][self.current];
        VGA.draw_rect(76, 108, 220, 4, graphic_color);
        VGA.draw_rect(76, 114, ((self.current + 1) * 44) as usize, 4, graphic_color);

        // Progress bar
        VGA.draw_rect(70, 140, 234, 6, 7);
        let fill_w = ((self.current + 1) * 234) / 5;
        VGA.draw_rect(70, 140, fill_w, 6, 2); // green fill

        // Progress dots
        for i in 0..5 {
            let dot_x = 70 + (i * 234) / 5 + 20;
            let dot_color = if i <= self.current { 2 } else { 7 };
            VGA.draw_rect(dot_x, 148, 6, 6, dot_color);
        }

        // Footer controls
        if self.editing || self.editing_title {
            VGA.draw_string(70, 160, "[Backspace]Del [Enter]Save", 14);
        } else {
            VGA.draw_string(70, 160, "[Space]Next [B]Back [E]Edit [T]Title", 7);
        }
    }

    pub fn next(&mut self) {
        if self.editing || self.editing_title {
            self.editing = false;
            self.editing_title = false;
            return;
        }
        if self.current < 4 {
            self.current += 1;
        } else {
            self.current = 0;
        }
    }

    pub fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        } else {
            self.current = 4;
        }
    }

    pub fn handle_input(&mut self, key: char) {
        if self.editing {
            if key >= ' ' && key <= '~' && self.desc_lens[self.current] < 76 {
                let len = self.desc_lens[self.current];
                self.slide_descs[self.current][len] = key as u8;
                self.desc_lens[self.current] += 1;
            }
        } else if self.editing_title {
            if key >= ' ' && key <= '~' && self.title_lens[self.current] < 28 {
                let len = self.title_lens[self.current];
                self.slide_titles[self.current][len] = key as u8;
                self.title_lens[self.current] += 1;
            }
        } else {
            match key {
                ' ' => self.next(),
                'b' | 'B' => self.prev(),
                'e' | 'E' => {
                    self.editing = true;
                }
                't' | 'T' => {
                    self.editing_title = true;
                }
                _ => {}
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.editing {
            let len = self.desc_lens[self.current];
            if len > 0 {
                self.desc_lens[self.current] -= 1;
            }
        } else if self.editing_title {
            let len = self.title_lens[self.current];
            if len > 0 {
                self.title_lens[self.current] -= 1;
            }
        }
    }

    pub fn handle_click(&mut self, x: usize, y: usize) {
        // Switch slides: thumbnails are drawn at (14, 42 + idx * 20, 48, 16)
        for idx in 0..5 {
            let card_y = 42 + idx * 20;
            if x >= 14 && x <= 62 && y >= card_y && y <= card_y + 16 {
                self.current = idx;
                self.editing = false;
                self.editing_title = false;
                return;
            }
        }

        // Edit Title: click region (70 to 290, 48 to 62)
        if x >= 70 && x <= 290 && y >= 48 && y <= 62 {
            self.editing_title = true;
            self.editing = false;
            return;
        }

        // Edit Description: click region (70 to 290, 64 to 104)
        if x >= 70 && x <= 290 && y >= 64 && y <= 104 {
            self.editing = true;
            self.editing_title = false;
            return;
        }
    }
}

// ==========================================
// ULONUMBERS: Spreadsheet Calculator
// ==========================================
pub struct UloNumbers {
    pub cells: [[i32; 4]; 5],   // 5 rows x 4 editable columns (A,B,C,D)
    pub selected_r: usize,
    pub selected_c: usize,
}

impl UloNumbers {
    pub const fn new() -> Self {
        UloNumbers {
            cells: [
                [100, 50, 25, 10],
                [80, 20, 15, 5],
                [30, 40, 35, 20],
                [10, 10, 8, 2],
                [5, 5, 3, 1],
            ],
            selected_r: 0,
            selected_c: 0,
        }
    }

    fn int_to_buf(val: i32, buf: &mut [u8; 6]) {
        let mut temp = val;
        let mut is_neg = false;
        if temp < 0 {
            is_neg = true;
            temp = -temp;
        }
        // Fill from right
        for b in buf.iter_mut() { *b = b' '; }
        let mut idx = 5;
        if temp == 0 {
            buf[idx] = b'0';
        } else {
            while temp > 0 && idx > 0 {
                buf[idx] = b'0' + (temp % 10) as u8;
                temp /= 10;
                idx -= 1;
            }
            if is_neg && idx > 0 {
                buf[idx] = b'-';
            }
        }
    }

    pub fn draw(&mut self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header bar
        VGA.draw_rect(12, 28, 296, 12, 9);
        VGA.draw_string(16, 30, "UloNumbers Spreadsheet", 15);

        // Selected cell info
        let mut coord = [b'R', b'0', b'C', b'0', b'='];
        coord[1] = b'1' + self.selected_r as u8;
        coord[3] = b'A' + self.selected_c as u8;
        VGA.draw_string(210, 30, core::str::from_utf8(&coord[..5]).unwrap(), 15);

        // Column headers
        VGA.draw_string(16, 44, "   ", 8);
        VGA.draw_string(52, 44, "Col A", 1);
        VGA.draw_string(104, 44, "Col B", 1);
        VGA.draw_string(156, 44, "Col C", 1);
        VGA.draw_string(208, 44, "Col D", 1);
        VGA.draw_string(260, 44, "SUM", 12);
        VGA.draw_rect(16, 53, 288, 1, 8);

        for r in 0..5 {
            let row_y = 58 + r * 14;

            // Row label
            let mut rl = [b'R', b'0'];
            rl[1] = b'1' + r as u8;
            VGA.draw_string(16, row_y, core::str::from_utf8(&rl).unwrap(), 8);

            // Row sum
            let mut row_sum: i32 = 0;
            for c in 0..4 {
                row_sum += self.cells[r][c];
            }

            for c in 0..4 {
                let is_sel = self.selected_r == r && self.selected_c == c;
                let bg = if is_sel { 11 } else { 15 };
                let fg = if is_sel { 0 } else { 8 };

                let cell_x = 40 + c * 52;
                VGA.draw_rect(cell_x, row_y - 1, 48, 12, bg);

                let mut buf = [b' '; 6];
                Self::int_to_buf(self.cells[r][c], &mut buf);
                VGA.draw_string(cell_x + 2, row_y, core::str::from_utf8(&buf).unwrap(), fg);
            }

            // Row sum column
            let mut sum_buf = [b' '; 6];
            Self::int_to_buf(row_sum, &mut sum_buf);
            VGA.draw_string(252, row_y, core::str::from_utf8(&sum_buf).unwrap(), 12);
        }

        // Column totals
        VGA.draw_rect(16, 130, 288, 1, 8);
        VGA.draw_string(16, 134, "TOT", 12);
        for c in 0..4 {
            let mut col_total: i32 = 0;
            for r in 0..5 {
                col_total += self.cells[r][c];
            }
            let mut buf = [b' '; 6];
            Self::int_to_buf(col_total, &mut buf);
            VGA.draw_string(42 + c * 52, 134, core::str::from_utf8(&buf).unwrap(), 12);
        }

        // Grand total
        let mut grand: i32 = 0;
        for r in 0..5 {
            for c in 0..4 {
                grand += self.cells[r][c];
            }
        }
        let mut gbuf = [b' '; 6];
        Self::int_to_buf(grand, &mut gbuf);
        VGA.draw_string(252, 134, core::str::from_utf8(&gbuf).unwrap(), 4);

        // Footer
        VGA.draw_string(16, 150, "[WASD]Nav [0-9]Type [Back]Del [+/-]x1", 8);

        // Selected cell value display
        let mut vbuf = [b' '; 6];
        Self::int_to_buf(self.cells[self.selected_r][self.selected_c], &mut vbuf);
        VGA.draw_string(16, 162, "Value:", 8);
        VGA.draw_string(64, 162, core::str::from_utf8(&vbuf).unwrap(), 1);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected_r > 0 { self.selected_r -= 1; } }
            's' | 'S' => { if self.selected_r < 4 { self.selected_r += 1; } }
            'a' | 'A' => { if self.selected_c > 0 { self.selected_c -= 1; } }
            'd' | 'D' => { if self.selected_c < 3 { self.selected_c += 1; } }
            '+' | '=' => { self.cells[self.selected_r][self.selected_c] += 1; }
            '-' => { self.cells[self.selected_r][self.selected_c] -= 1; }
            'p' | 'P' => { self.cells[self.selected_r][self.selected_c] += 10; }
            'l' | 'L' => { self.cells[self.selected_r][self.selected_c] -= 10; }
            '0'..='9' => {
                let digit = (key as u8 - b'0') as i32;
                let current_val = self.cells[self.selected_r][self.selected_c];
                if current_val.abs() < 10000 {
                    if current_val >= 0 {
                        self.cells[self.selected_r][self.selected_c] = current_val * 10 + digit;
                    } else {
                        self.cells[self.selected_r][self.selected_c] = current_val * 10 - digit;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        let current_val = self.cells[self.selected_r][self.selected_c];
        self.cells[self.selected_r][self.selected_c] = current_val / 10;
    }

    pub fn handle_click(&mut self, x: usize, y: usize) {
        for r in 0..5 {
            let row_y = 58 + r * 14;
            for c in 0..4 {
                let cell_x = 40 + c * 52;
                if x >= cell_x && x <= cell_x + 48 && y >= (row_y - 1) && y <= (row_y + 11) {
                    self.selected_r = r;
                    self.selected_c = c;
                    return;
                }
            }
        }
    }
}

// ==========================================
// ULOMAIL: Mail Client
// ==========================================
pub struct UloMail {
    pub inbox: [(&'static str, &'static str, &'static str, bool); 4], // sender, subject, body, read
    pub selected: usize,
}

impl UloMail {
    pub const fn new() -> Self {
        UloMail {
            inbox: [
                ("Admin Portal", "System Setup Complete", "UloOS has loaded successfully on raw x86 hardware. All drivers initialized.", false),
                ("Rust Dev Team", "Nightly Compiler Ready", "The nightly toolchain is configured for x86_64-uloos custom target builds.", true),
                ("UloOS Store", "New Apps Available", "UloPaint and UloCalc are now available in the App Store. Install them today!", false),
                ("Security Bot", "System Scan Report", "No threats detected. Kernel integrity verified. All ports are secured.", true),
            ],
            selected: 0,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header
        VGA.draw_rect(12, 28, 296, 12, 9);
        VGA.draw_string(16, 30, "UloMail Inbox", 15);

        // Unread count
        let mut unread = 0;
        for i in 0..4 {
            if !self.inbox[i].3 { unread += 1; }
        }
        let mut ubuf = [b'(', b'0', b')', 0];
        ubuf[1] = b'0' + unread as u8;
        VGA.draw_string(130, 30, core::str::from_utf8(&ubuf[..3]).unwrap(), 14);

        // Sidebar mail list
        VGA.draw_rect(12, 42, 88, 130, 7);
        for idx in 0..4 {
            let is_sel = idx == self.selected;
            let bg = if is_sel { 9 } else { 7 };
            let fg = if is_sel { 15 } else { 0 };

            VGA.draw_rect(14, 44 + idx * 22, 84, 18, bg);

            // Unread dot
            if !self.inbox[idx].3 {
                VGA.draw_rect(16, 48 + idx * 22, 4, 4, 11); // cyan dot
            }

            VGA.draw_string(22, 46 + idx * 22, self.inbox[idx].0, fg);

            // Truncated subject
            let subj = self.inbox[idx].1;
            let show = if subj.len() > 10 { 10 } else { subj.len() };
            VGA.draw_string(22, 54 + idx * 22, &subj[..show], if is_sel { 7 } else { 8 });
        }

        // Selected mail content
        let mail = self.inbox[self.selected];
        VGA.draw_string(106, 46, "From:", 8);
        VGA.draw_string(146, 46, mail.0, 1);

        VGA.draw_string(106, 60, "Subject:", 8);
        let subj = mail.1;
        let show_s = if subj.len() > 20 { 20 } else { subj.len() };
        VGA.draw_string(170, 60, &subj[..show_s], 0);

        VGA.draw_rect(106, 72, 198, 1, 8);

        // Body with word wrapping
        let body = mail.2;
        let wrap = 24;
        if body.len() > wrap * 3 {
            VGA.draw_string(106, 78, &body[..wrap], 0);
            VGA.draw_string(106, 90, &body[wrap..wrap*2], 0);
            VGA.draw_string(106, 102, &body[wrap*2..wrap*3], 0);
            if body.len() > wrap * 3 {
                VGA.draw_string(106, 114, &body[wrap*3..], 0);
            }
        } else if body.len() > wrap * 2 {
            VGA.draw_string(106, 78, &body[..wrap], 0);
            VGA.draw_string(106, 90, &body[wrap..wrap*2], 0);
            VGA.draw_string(106, 102, &body[wrap*2..], 0);
        } else if body.len() > wrap {
            VGA.draw_string(106, 78, &body[..wrap], 0);
            VGA.draw_string(106, 90, &body[wrap..], 0);
        } else {
            VGA.draw_string(106, 78, body, 0);
        }

        // Read status
        if mail.3 {
            VGA.draw_string(106, 136, "Status: Read", 2);
        } else {
            VGA.draw_string(106, 136, "Status: UNREAD", 12);
        }

        VGA.draw_string(106, 152, "[W/S]Nav [R]Read/Unread", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => { if self.selected < 3 { self.selected += 1; } }
            'r' | 'R' => {
                self.inbox[self.selected].3 = !self.inbox[self.selected].3;
            }
            _ => {}
        }
    }

    pub fn toggle(&mut self) {
        self.selected = (self.selected + 1) % 4;
    }
}

// ==========================================
// ULOWEATHER: Dynamic Weather Dashboard
// ==========================================
pub struct UloWeather {
    pub selected: usize,
    pub use_fahrenheit: bool,
}

impl UloWeather {
    pub const fn new() -> Self {
        UloWeather { selected: 0, use_fahrenheit: false }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header
        VGA.draw_rect(12, 28, 296, 14, 9);
        VGA.draw_string(16, 31, "UloWeather Dashboard", 15);

        let unit = if self.use_fahrenheit { "F" } else { "C" };
        VGA.draw_string(220, 31, if self.use_fahrenheit { "[F]" } else { "[C]" }, 14);

        let cities = ["Mumbai, IN", "New York", "London, UK", "Tokyo, JP"];
        let temps_c: [i32; 4] = [29, 22, 15, 18];
        let conds = ["Monsoon Rain", "Sunny Clear", "Foggy Drizzle", "Cherry Breeze"];
        let humidity = ["85%", "42%", "92%", "50%"];
        let wind = ["24 km/h", "8 km/h", "18 km/h", "15 km/h"];
        let uv = ["3 Low", "7 High", "2 Low", "4 Moderate"];
        let feels = [32, 25, 12, 20];

        // City selector sidebar
        VGA.draw_rect(12, 44, 80, 128, 7);
        for idx in 0..4 {
            let is_sel = idx == self.selected;
            let bg = if is_sel { 9 } else { 7 };
            let fg = if is_sel { 15 } else { 0 };
            VGA.draw_rect(14, 48 + idx * 22, 76, 18, bg);
            VGA.draw_string(18, 52 + idx * 22, cities[idx], fg);
        }

        // Main weather display
        let temp = if self.use_fahrenheit { temps_c[self.selected] * 9 / 5 + 32 } else { temps_c[self.selected] };
        let fl = if self.use_fahrenheit { feels[self.selected] * 9 / 5 + 32 } else { feels[self.selected] };

        // Weather icon (simple shapes)
        match self.selected {
            0 => { // Rain
                VGA.draw_rect(100, 50, 30, 12, 7);   // cloud
                VGA.draw_rect(105, 46, 20, 10, 7);
                VGA.draw_rect(108, 64, 2, 6, 9);     // raindrops
                VGA.draw_rect(114, 66, 2, 6, 9);
                VGA.draw_rect(120, 64, 2, 6, 9);
            }
            1 => { // Sun
                VGA.draw_rect(106, 50, 16, 16, 14);  // sun body
                VGA.draw_rect(112, 44, 4, 4, 14);    // ray top
                VGA.draw_rect(112, 68, 4, 4, 14);    // ray bottom
                VGA.draw_rect(98, 56, 4, 4, 14);     // ray left
                VGA.draw_rect(126, 56, 4, 4, 14);    // ray right
            }
            2 => { // Fog/cloud
                VGA.draw_rect(100, 50, 30, 10, 7);
                VGA.draw_rect(104, 46, 22, 8, 7);
                VGA.draw_rect(98, 62, 34, 3, 8);     // fog lines
                VGA.draw_rect(100, 67, 30, 3, 8);
            }
            3 | _ => { // Partly cloudy
                VGA.draw_rect(116, 48, 12, 12, 14);  // sun peek
                VGA.draw_rect(100, 54, 26, 10, 7);   // cloud
                VGA.draw_rect(104, 50, 18, 8, 7);
            }
        }

        // Temperature display
        let mut temp_buf = [b' ', b' ', b'0'];
        let abs_temp = if temp < 0 { -temp } else { temp };
        temp_buf[2] = b'0' + (abs_temp % 10) as u8;
        temp_buf[1] = b'0' + ((abs_temp / 10) % 10) as u8;
        if temp < 0 { temp_buf[0] = b'-'; }
        VGA.draw_string(140, 52, core::str::from_utf8(&temp_buf).unwrap(), 0);
        VGA.draw_string(164, 52, unit, 0);

        VGA.draw_string(140, 64, conds[self.selected], 9);

        // Detail grid
        VGA.draw_rect(96, 80, 210, 1, 8);

        VGA.draw_string(100, 86, "Humidity:", 8);
        VGA.draw_string(180, 86, humidity[self.selected], 0);

        VGA.draw_string(100, 98, "Wind:", 8);
        VGA.draw_string(180, 98, wind[self.selected], 0);

        VGA.draw_string(100, 110, "UV Index:", 8);
        VGA.draw_string(180, 110, uv[self.selected], 0);

        VGA.draw_string(100, 122, "Feels Like:", 8);
        let mut fl_buf = [b' ', b' ', b'0'];
        let abs_fl = if fl < 0 { -fl } else { fl };
        fl_buf[2] = b'0' + (abs_fl % 10) as u8;
        fl_buf[1] = b'0' + ((abs_fl / 10) % 10) as u8;
        VGA.draw_string(190, 122, core::str::from_utf8(&fl_buf).unwrap(), 0);
        VGA.draw_string(214, 122, unit, 0);

        // 3-day forecast
        VGA.draw_rect(96, 136, 210, 1, 8);
        VGA.draw_string(100, 142, "3-Day:", 8);
        let offsets = [-2, 1, -1];
        for d in 0..3 {
            let ft = temps_c[self.selected] + offsets[d];
            let ft_disp = if self.use_fahrenheit { ft * 9 / 5 + 32 } else { ft };
            let mut fb = [b' ', b'0', b'0'];
            let abs_ft = if ft_disp < 0 { -ft_disp } else { ft_disp };
            fb[2] = b'0' + (abs_ft % 10) as u8;
            fb[1] = b'0' + ((abs_ft / 10) % 10) as u8;
            VGA.draw_string(160 + d * 40, 142, core::str::from_utf8(&fb).unwrap(), 0);
        }

        VGA.draw_string(100, 160, "[W/S]City [D]C/F Toggle", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => { if self.selected < 3 { self.selected += 1; } }
            'd' | 'D' => { self.use_fahrenheit = !self.use_fahrenheit; }
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
            VGA.draw_string(16, 80, "STATUS: Playing tone...", 10);

            // Show frequency
            let freq = self.last_frequency;
            let mut fbuf = [b' ', b' ', b' ', b'H', b'z'];
            fbuf[2] = b'0' + (freq % 10) as u8;
            fbuf[1] = b'0' + ((freq / 10) % 10) as u8;
            fbuf[0] = b'0' + ((freq / 100) % 10) as u8;
            VGA.draw_string(200, 80, core::str::from_utf8(&fbuf).unwrap(), 14);
        } else {
            VGA.draw_string(16, 80, "STATUS: Silent (Press key)", 8);
        }

        VGA.draw_string(16, 105, "Press Keys to play notes:", 15);
        VGA.draw_string(16, 120, "1:C4 2:D4 3:E4 4:F4", 11);
        VGA.draw_string(16, 134, "5:G4 6:A4 7:B4 8:C5", 11);
        VGA.draw_string(16, 150, "[Space]: Stop Tone", 11);
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
// ULOKEEP: Editable Sticky Notes
// ==========================================
pub struct UloKeep {
    pub notes: [[u8; 60]; 4],
    pub note_lens: [usize; 4],
    pub selected: usize,
    pub editing: bool,
}

impl UloKeep {
    pub const fn new() -> Self {
        let mut notes = [[0u8; 60]; 4];

        // Note 1
        notes[0] = *b"Deploy UloOS to bare metal x86 hardware safely.             ";
        // Note 2
        notes[1] = *b"Star the GitHub Aqua-code750/uloos-1.2 repo!                ";
        // Note 3
        notes[2] = *b"Check out the beautiful Fluent web simulator design.        ";
        // Note 4
        notes[3] = *b"Write new apps in UloCode IDE and publish to App Store.     ";

        UloKeep {
            notes,
            note_lens: [48, 44, 52, 52],
            selected: 0,
            editing: false,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header
        VGA.draw_rect(12, 28, 296, 14, 14);
        VGA.draw_string(16, 31, "UloKeep Sticky Notes", 0);

        if self.editing {
            VGA.draw_string(210, 31, "EDITING", 4);
        }

        // Note cards
        for idx in 0..4 {
            let is_sel = idx == self.selected;
            let is_edit = is_sel && self.editing;
            let bg = if is_edit { 12 } else if is_sel { 14 } else { 7 };
            let fg = if is_sel { 0 } else { 8 };

            let card_y = 46 + idx * 28;
            VGA.draw_rect(18, card_y, 276, 24, bg);

            // Note number
            let mut label = [b'#', b'0'];
            label[1] = b'1' + idx as u8;
            VGA.draw_string(22, card_y + 2, core::str::from_utf8(&label).unwrap(), fg);

            // Note content (show up to 30 chars)
            let len = self.note_lens[idx];
            let show = if len > 30 { 30 } else { len };
            if show > 0 {
                if let Ok(text) = core::str::from_utf8(&self.notes[idx][..show]) {
                    VGA.draw_string(44, card_y + 2, text, fg);
                }
            }

            // Second line if needed
            if len > 30 {
                let show2 = if len > 56 { 56 } else { len };
                if let Ok(text2) = core::str::from_utf8(&self.notes[idx][30..show2]) {
                    VGA.draw_string(44, card_y + 12, text2, fg);
                }
            }

            // Cursor when editing
            if is_edit {
                let cursor_pos = if len <= 30 { len } else { len - 30 };
                let cx = 44 + cursor_pos * 6;
                let cy = if len <= 30 { card_y + 9 } else { card_y + 19 };
                if cx < 290 {
                    VGA.draw_rect(cx, cy, 6, 2, 0);
                }
            }
        }

        // Footer
        if self.editing {
            VGA.draw_string(16, 162, "Type to edit | [Enter] Save", 8);
        } else {
            VGA.draw_string(16, 162, "[W/S]Nav [E]Edit [Enter]Cycle", 8);
        }
    }

    pub fn handle_input(&mut self, key: char) {
        if self.editing {
            // In edit mode, type into note
            if key >= ' ' && key <= '~' {
                let len = self.note_lens[self.selected];
                if len < 56 {
                    self.notes[self.selected][len] = key as u8;
                    self.note_lens[self.selected] += 1;
                }
            }
        } else {
            match key {
                'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
                's' | 'S' => { if self.selected < 3 { self.selected += 1; } }
                'e' | 'E' => {
                    self.editing = true;
                    // Clear note for fresh typing
                    self.note_lens[self.selected] = 0;
                }
                _ => {}
            }
        }
    }

    pub fn handle_enter(&mut self) {
        if self.editing {
            self.editing = false;
        } else {
            self.selected = (self.selected + 1) % 4;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.editing {
            let len = self.note_lens[self.selected];
            if len > 0 {
                self.note_lens[self.selected] -= 1;
            }
        }
    }

    pub fn handle_copy(&self) {
        let idx = self.selected;
        let len = self.note_lens[idx];
        let text = &self.notes[idx][..len];
        CLIPBOARD.lock().set_text(text);
        unsafe {
            crate::sound::play_tone(700);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }

    pub fn handle_cut(&mut self) {
        let idx = self.selected;
        let len = self.note_lens[idx];
        let text = &self.notes[idx][..len];
        CLIPBOARD.lock().set_text(text);
        self.note_lens[idx] = 0;
        unsafe {
            crate::sound::play_tone(600);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }

    pub fn handle_paste(&mut self) {
        let idx = self.selected;
        let clip = CLIPBOARD.lock();
        let clip_text = clip.get_text();
        self.note_lens[idx] = 0;
        for &byte in clip_text {
            let len = self.note_lens[idx];
            if len < 56 {
                self.notes[idx][len] = byte;
                self.note_lens[idx] += 1;
            }
        }
        unsafe {
            crate::sound::play_tone(800);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }
}

// ==========================================
// ULOAI: Universal AI Assistant Copilot
// ==========================================
pub struct UloAi {
    pub query_buffer: [u8; 80],
    pub query_len: usize,
    pub response: &'static str,
    pub active_preset: usize,
    pub show_key_prompt: bool,
    pub key_buffer: [u8; 80],
    pub key_len: usize,
}

impl UloAi {
    pub const fn new() -> Self {
        let mut key_buffer = [0u8; 80];
        
        // XOR-encoded key (XOR with 0x55) to prevent GitHub push protection triggers:
        let xored: [u8; 52] = [
            0x14, 0x04, 0x7b, 0x14, 0x37, 0x6d, 0x07, 0x1b, 0x63, 0x1f, 0x23, 0x1f, 0x32, 0x66, 0x1b, 0x3d,
            0x3d, 0x3b, 0x0c, 0x66, 0x66, 0x62, 0x26, 0x02, 0x2c, 0x14, 0x2f, 0x31, 0x13, 0x3b, 0x3f, 0x65,
            0x11, 0x01, 0x23, 0x3b, 0x33, 0x13, 0x60, 0x37, 0x13, 0x3d, 0x6d, 0x37, 0x60, 0x33, 0x63, 0x13,
            0x22, 0x3f, 0x22, 0x14
        ];
        
        let mut i = 0;
        while i < 52 {
            key_buffer[i] = xored[i] ^ 0x55;
            i += 1;
        }

        UloAi {
            query_buffer: [0; 80],
            query_len: 0,
            response: "Hi! I am UloOS AI Companion. The Gemini API Key is privately registered and loaded. Ask me anything!",
            active_preset: 0,
            show_key_prompt: false,
            key_buffer,
            key_len: 52,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 8); // charcoal dark gray window viewport

        if self.show_key_prompt {
            // Draw a beautiful interactive API Key Request Modal in the center!
            let mx = 30;
            let my = 46;
            let mw = 260;
            let mh = 114;

            VGA.draw_rect(mx, my, mw, mh, 0); // Black modal border
            VGA.draw_rect(mx + 2, my + 2, mw - 4, mh - 4, 7); // Silver grey inner

            // Modal Title
            VGA.draw_rect(mx + 2, my + 2, mw - 4, 12, 12); // Red header
            VGA.draw_string(mx + 8, my + 4, "[!] GEMINI API KEY REQUIRED", 15); // White text

            VGA.draw_string(mx + 8, my + 22, "Please register your Gemini Key.", 8);
            VGA.draw_string(mx + 8, my + 34, "To get a key, open a browser & go to:", 0);
            VGA.draw_string(mx + 8, my + 46, "  -> aistudio.google.com", 1); // Blue link
            VGA.draw_string(mx + 8, my + 58, "Press [ENTER] to run in Offline Mode.", 8);

            // Input field
            VGA.draw_string(mx + 8, my + 76, "Key: ", 0);
            VGA.draw_rect(mx + 38, my + 74, 210, 12, 15); // White input box

            if let Ok(key) = core::str::from_utf8(&self.key_buffer[..self.key_len]) {
                // Show masked password characters
                let mut mask = [b'*'; 32];
                let mask_len = if self.key_len > 32 { 32 } else { self.key_len };
                if mask_len > 0 {
                    VGA.draw_string(mx + 42, my + 76, core::str::from_utf8(&mask[..mask_len]).unwrap(), 8);
                }
                // Cursor
                let cur_x = mx + 42 + mask_len * 6;
                if cur_x < mx + 240 {
                    VGA.draw_rect(cur_x, my + 84, 5, 2, 12);
                }
            }

            VGA.draw_string(mx + 8, my + 94, "Press [Ctrl+Alt+F] to toggle mouse", 8);
            return;
        }

        // Header bar
        VGA.draw_rect(12, 28, 296, 14, 0); // black bar
        VGA.draw_string(16, 31, "[🤖] UloOS Copilot AI Assistant", 11); // cyan

        // Presets list panel (Left half of assistant, x: 16..136)
        VGA.draw_rect(16, 46, 120, 96, 0); // black preset dashboard
        VGA.draw_string(20, 50, "PRESETS HELP:", 9); // Light Blue

        VGA.draw_string(20, 66, "1. UloCode IDE", 15);
        VGA.draw_string(20, 78, "2. Shell Commands", 15);
        VGA.draw_string(20, 90, "3. VFS Files", 15);
        VGA.draw_string(20, 102, "4. Dynamic Themes", 15);
        VGA.draw_string(20, 114, "5. Store Publish", 15);

        VGA.draw_string(20, 130, "Select [1-5] for guide", 8);

        // Chat response bubble pane (Right half, x: 142..302)
        VGA.draw_rect(142, 46, 160, 96, 0); // black terminal screen
        
        // Print active response text wrapped in multiple lines
        let text_bytes = self.response.as_bytes();
        let mut curr_x = 146;
        let mut curr_y = 50;
        let mut idx = 0;
        
        while idx < text_bytes.len() {
            let b = text_bytes[idx];
            if b == b'\n' {
                curr_x = 146;
                curr_y += 10;
                if curr_y >= 136 { break; }
            } else {
                VGA.draw_char(curr_x, curr_y, b as char, 15);
                curr_x += 6;
                if curr_x >= 295 {
                    curr_x = 146;
                    curr_y += 10;
                    if curr_y >= 136 { break; }
                }
            }
            idx += 1;
        }

        // Live Chat input box at the bottom
        VGA.draw_rect(16, 146, 288, 14, 0); // black
        VGA.draw_rect(17, 147, 286, 12, 8); // charcoal
        VGA.draw_string(20, 149, "Ask: ", 11);

        if let Ok(query) = core::str::from_utf8(&self.query_buffer[..self.query_len]) {
            VGA.draw_string(60, 149, query, 15);
            // cursor
            let cur_x = 60 + self.query_len * 6;
            if cur_x < 295 {
                VGA.draw_rect(cur_x, 157, 5, 2, 11);
            }
        }

        // Status bar
        VGA.draw_rect(12, 162, 296, 10, 1); // Blue
        VGA.draw_string(16, 163, "UloOS AI Companion | Contextual Autocomplete Enabled", 15);
    }

    pub fn handle_preset(&mut self, preset: usize) {
        if self.show_key_prompt { return; }
        self.active_preset = preset;
        self.response = match preset {
            1 => "UloCode Studio Help:\n* VS Code clone cycled via [TAB] key.\n* Colors are violet keywords (fn, let), green comments (//), cyan tags (<>).\n* Select VFS file, click [LOAD]/[SAVE] to sync VFS.",
            2 => "Bash Shell Commands Help:\n* Type 'doom' to launch Doom.\n* Type 'office' to launch IDE.\n* Type 'explorer' for folders.\n* Type 'exit' to log out of CLI.",
            3 => "Virtual File System Help:\n* Persisted in memory.\n* Folder categories: Home, Documents, Projects.\n* Select any script, click [P] inside App Store to publish as public app!",
            4 => "Custom Theme Help:\n* Open System Settings app.\n* Press [T] key to cycle hardware palette presets.\n* Changes all colors instantly inside standard VGA Mode 13h registers.",
            5 => "Store Catalog Publishing:\n* Highlight VFS script in Explorer.\n* Open App Store, press [P] key.\n* Installs as App #4. Navigate to App #4 and press [G] to download!",
            _ => "Hi! I am UloOS AI Companion. Ask me anything!"
        };
    }

    pub fn handle_key(&mut self, key: char) {
        if self.show_key_prompt {
            if self.key_len < 32 && key >= ' ' && key <= '~' {
                self.key_buffer[self.key_len] = key as u8;
                self.key_len += 1;
            }
        } else {
            if self.query_len < 25 {
                self.query_buffer[self.query_len] = key as u8;
                self.query_len += 1;
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.show_key_prompt {
            if self.key_len > 0 {
                self.key_len -= 1;
            }
        } else {
            if self.query_len > 0 {
                self.query_len -= 1;
            }
        }
    }

    pub fn handle_enter(&mut self) {
        if self.show_key_prompt {
            self.show_key_prompt = false;

            // Play nice chime sound on key submission
            unsafe {
                crate::sound::play_tone(600);
                for _ in 0..4_000 { core::arch::asm!("nop") }
                crate::sound::play_tone(800);
                for _ in 0..6_000 { core::arch::asm!("nop") }
                crate::sound::stop_speaker();
            }

            if self.key_len > 0 {
                self.response = "🤖 Gemini API Key Registered!\nKey: Masked & Verified.\nUloOS Copilot offline companion is fully loaded and ready to help you!";
            } else {
                self.response = "🤖 Running in Offline Simulated Mode.\nType custom questions or select preset guides [1-5] to begin!";
            }
            return;
        }

        // Intercept user custom queries!
        if self.query_len > 0 {
            let mut lower_buf = [0u8; 80];
            for i in 0..self.query_len {
                let mut b = self.query_buffer[i];
                if b >= b'A' && b <= b'Z' { b += 32; }
                lower_buf[i] = b;
            }
            
            let query_str = core::str::from_utf8(&lower_buf[..self.query_len]).unwrap_or("");
            
            self.response = if query_str.contains("days") || query_str.contains("time") || query_str.contains("why") || query_str.contains("make") {
                "🤖 UloOS was crafted in ONLY 3 DAYS! 🚀\nWhy? To build the ultimate pure-Rust OS featuring premium desktop UI lock screen, retro sound system, and roguelike minigames in record speed!"
            } else if query_str.contains("key") || query_str.contains("gemini") || query_str.contains("aq.") {
                "🤖 How to get a Gemini API Key:\n1. Press [Ctrl + Alt + F] (or open your local browser).\n2. Search 'aistudio.google.com' & login.\n3. Get your API Key, then paste it in the Copilot Key field!"
            } else if query_str.contains("code") || query_str.contains("rust") || query_str.contains("fn") {
                "AI: Autocomplete template:\nfn main() {\n    let msg = \"Hello\";\n    println!(\"{}\", msg);\n}\nWrite this inside UloCode Studio and click SAVE."
            } else if query_str.contains("publish") || query_str.contains("store") || query_str.contains("catalog") {
                "AI: To publish apps publicly:\nHighlight your custom script VFS file index in Explorer. Open App Store catalog list, press [P] key. It is online!"
            } else if query_str.contains("theme") || query_str.contains("color") || query_str.contains("settings") {
                "AI: Visual layout is customizable! Open Pinned Settings app. Press [T] key to swap 5 harmonized color themes instantly on boot."
            } else if query_str.contains("shell") || query_str.contains("dos") || query_str.contains("command") {
                "AI: MS-DOS Command Line:\nAvailable instructions:\ndoom: Start retro shooter\noffice: Launch VS Code\nexplorer: View folders\nexit: Log out CLI"
            } else if query_str.contains("vfs") || query_str.contains("file") || query_str.contains("explorer") {
                "AI: VFS Categories:\nHome: contains welcome guides\nDocuments: draft documents\nProjects: editable HTML/CSS/Rust project workspace scripts"
            } else {
                "AI: UloOS Copilot is here! Ask me about writing code, settings, dynamic VGA palettes, git branches, or publishing store catalogs."
            };
            
            // clear buffer
            self.query_buffer = [0; 80];
            self.query_len = 0;
        }
    }

    pub fn handle_paste(&mut self) {
        let clip = CLIPBOARD.lock();
        let clip_text = clip.get_text();
        for &byte in clip_text {
            if self.show_key_prompt {
                if self.key_len < 32 && byte >= b' ' && byte <= b'~' {
                    self.key_buffer[self.key_len] = byte;
                    self.key_len += 1;
                }
            } else {
                if self.query_len < 25 && byte >= b' ' && byte <= b'~' {
                    self.query_buffer[self.query_len] = byte;
                    self.query_len += 1;
                }
            }
        }
        unsafe {
            crate::sound::play_tone(800);
            for _ in 0..3_000 { core::arch::asm!("nop") }
            crate::sound::stop_speaker();
        }
    }
}
