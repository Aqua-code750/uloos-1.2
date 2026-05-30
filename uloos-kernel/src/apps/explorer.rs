use crate::vga_driver::VGA;

// ==========================================
// VIRTUAL FILESYSTEM
// ==========================================
#[derive(Clone, Copy)]
pub struct FSEntry {
    pub name: [u8; 20],
    pub name_len: usize,
    pub content: [u8; 80],
    pub content_len: usize,
    pub is_dir: bool,
    pub parent: usize,       // index of parent (0 = root)
    pub active: bool,        // is this entry in use?
}

impl FSEntry {
    pub const fn empty() -> Self {
        FSEntry {
            name: [0; 20],
            name_len: 0,
            content: [0; 80],
            content_len: 0,
            is_dir: false,
            parent: 0,
            active: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ExplorerMode {
    Normal,
    CreatingFile,
    CreatingFolder,
}

// ==========================================
// FILE EXPLORER with Virtual FS
// ==========================================
pub struct FileExplorer {
    pub entries: [FSEntry; 24],
    pub entry_count: usize,
    pub current_dir: usize,      // index of current directory
    pub selected: usize,         // index within the visible list
    pub mode: ExplorerMode,
    pub input_buf: [u8; 20],
    pub input_len: usize,
}

impl FileExplorer {
    pub const fn new() -> Self {
        let mut entries = [FSEntry::empty(); 24];

        // Entry 0: Root directory (always exists)
        entries[0].name[0] = b'C';
        entries[0].name[1] = b':';
        entries[0].name[2] = b'\\';
        entries[0].name_len = 3;
        entries[0].is_dir = true;
        entries[0].parent = 0;
        entries[0].active = true;

        // Entry 1: Welcome.txt
        entries[1].name = *b"Welcome.txt\0\0\0\0\0\0\0\0\0";
        entries[1].name_len = 11;
        entries[1].content = *b"Welcome to UloOS! A bare-metal Rust OS running directly on x86_64 hardware.   \0\0";
        entries[1].content_len = 76;
        entries[1].is_dir = false;
        entries[1].parent = 0;
        entries[1].active = true;

        // Entry 2: system.cfg
        entries[2].name = *b"system.cfg\0\0\0\0\0\0\0\0\0\0";
        entries[2].name_len = 10;
        entries[2].content = *b"BOOT=VGA_GRAPHICS MEMORY=8192 INTERRUPT=POLLING DISPLAY=320x200             \0\0\0\0";
        entries[2].content_len = 72;
        entries[2].is_dir = false;
        entries[2].parent = 0;
        entries[2].active = true;

        // Entry 3: Documents folder
        entries[3].name = *b"Documents\0\0\0\0\0\0\0\0\0\0\0";
        entries[3].name_len = 9;
        entries[3].is_dir = true;
        entries[3].parent = 0;
        entries[3].active = true;

        // Entry 4: Projects folder
        entries[4].name = *b"Projects\0\0\0\0\0\0\0\0\0\0\0\0";
        entries[4].name_len = 8;
        entries[4].is_dir = true;
        entries[4].parent = 0;
        entries[4].active = true;

        // Entry 5: notes.txt inside Documents
        entries[5].name = *b"notes.txt\0\0\0\0\0\0\0\0\0\0\0";
        entries[5].name_len = 9;
        entries[5].content = *b"Check UloNumbers spreadsheet totals and verify all cell formulas correctly.\0\0\0\0\0";
        entries[5].content_len = 76;
        entries[5].is_dir = false;
        entries[5].parent = 3;
        entries[5].active = true;

        // Entry 6: readme.md inside Projects
        entries[6].name = *b"readme.md\0\0\0\0\0\0\0\0\0\0\0";
        entries[6].name_len = 9;
        entries[6].content = *b"UloOS 1.2 - A minimal bare-metal Rust kernel for QEMU x86_64 environments.  \0\0\0\0";
        entries[6].content_len = 74;
        entries[6].is_dir = false;
        entries[6].parent = 4;
        entries[6].active = true;

        FileExplorer {
            entries,
            entry_count: 7,
            current_dir: 0,
            selected: 0,
            mode: ExplorerMode::Normal,
            input_buf: [0; 20],
            input_len: 0,
        }
    }

    // Get children of current directory
    fn get_children_count(&self) -> usize {
        let mut count = 0;
        for i in 1..self.entry_count {
            if self.entries[i].active && self.entries[i].parent == self.current_dir {
                count += 1;
            }
        }
        count
    }

    // Get the nth child index of current directory
    fn get_child_index(&self, nth: usize) -> Option<usize> {
        let mut count = 0;
        for i in 1..self.entry_count {
            if self.entries[i].active && self.entries[i].parent == self.current_dir {
                if count == nth {
                    return Some(i);
                }
                count += 1;
            }
        }
        None
    }

    pub fn get_selected_entry_index(&self) -> Option<usize> {
        let mut vis_idx = 0;
        for i in 1..self.entry_count {
            if self.entries[i].active && self.entries[i].parent == self.current_dir {
                if vis_idx == self.selected {
                    return Some(i);
                }
                vis_idx += 1;
            }
        }
        None
    }

    pub fn draw(&self) {
        // Background
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Sidebar
        VGA.draw_rect(12, 28, 80, 144, 7);

        // Current path header
        VGA.draw_rect(12, 28, 296, 12, 9);
        VGA.draw_string(16, 30, "File Explorer", 15);

        // Sidebar - show folder tree
        VGA.draw_string(14, 44, "Folders", 1);
        VGA.draw_string(16, 56, "[C:]", 0);

        // Show root-level folders in sidebar
        let mut sidebar_y = 68;
        for i in 1..self.entry_count {
            if self.entries[i].active && self.entries[i].is_dir && self.entries[i].parent == 0 {
                let is_current = i == self.current_dir;
                let fg = if is_current { 9 } else { 8 };
                if let Ok(name) = core::str::from_utf8(&self.entries[i].name[..self.entries[i].name_len]) {
                    VGA.draw_string(22, sidebar_y, name, fg);
                }
                sidebar_y += 12;
                if sidebar_y > 160 { break; }
            }
        }

        // Main pane - file listing
        let children_count = self.get_children_count();

        // Current directory path
        VGA.draw_string(96, 44, "Path: ", 8);
        if self.current_dir == 0 {
            VGA.draw_string(140, 44, "C:\\", 0);
        } else if let Ok(name) = core::str::from_utf8(&self.entries[self.current_dir].name[..self.entries[self.current_dir].name_len]) {
            VGA.draw_string(140, 44, name, 0);
        }

        // File/folder list
        let mut list_y = 58;
        let mut vis_idx = 0;
        for i in 1..self.entry_count {
            if self.entries[i].active && self.entries[i].parent == self.current_dir {
                let is_sel = vis_idx == self.selected;
                let bg = if is_sel { 11 } else { 15 };
                let fg = if is_sel { 0 } else { 8 };

                VGA.draw_rect(94, list_y - 1, 210, 11, bg);

                // Detailed folder & file icons
                if self.entries[i].is_dir {
                    VGA.draw_rect(96, list_y, 10, 7, 14); // Bright Yellow folder
                    VGA.draw_rect(96, list_y - 2, 5, 2, 14); // folder tab
                    VGA.draw_rect(97, list_y, 8, 6, 6); // brown inside shadow
                    VGA.draw_rect(98, list_y + 1, 6, 5, 14); // yellow leaf (3D depth)
                } else {
                    VGA.draw_rect(96, list_y - 2, 8, 10, 15); // White document body
                    VGA.draw_rect(102, list_y - 2, 2, 2, 8); // Folded corner shadow
                    VGA.draw_rect(98, list_y + 1, 4, 1, 1); // Blue text line
                    VGA.draw_rect(98, list_y + 4, 4, 1, 1); // Blue text line
                    VGA.draw_rect(98, list_y + 6, 3, 1, 1); // Blue text line
                }

                if let Ok(name) = core::str::from_utf8(&self.entries[i].name[..self.entries[i].name_len]) {
                    VGA.draw_string(108, list_y, name, fg);
                }

                // Show type label
                if self.entries[i].is_dir {
                    VGA.draw_string(260, list_y, "DIR", 9);
                }

                list_y += 13;
                vis_idx += 1;
                if list_y > 140 { break; }
            }
        }

        if children_count == 0 {
            VGA.draw_string(96, 70, "(empty folder)", 8);
        }

        // Preview pane for selected file
        if let Some(sel_idx) = self.get_child_index(self.selected) {
            if !self.entries[sel_idx].is_dir && self.entries[sel_idx].content_len > 0 {
                VGA.draw_rect(94, 148, 210, 1, 7);
                if let Ok(content) = core::str::from_utf8(&self.entries[sel_idx].content[..self.entries[sel_idx].content_len]) {
                    let show_len = if content.len() > 25 { 25 } else { content.len() };
                    VGA.draw_string(96, 152, &content[..show_len], 0);
                }
            }
        }

        // Mode-specific UI
        match self.mode {
            ExplorerMode::Normal => {
                VGA.draw_string(96, 164, "[W/S]Nav [N]File [F]Dir", 8);
            }
            ExplorerMode::CreatingFile => {
                VGA.draw_rect(94, 148, 210, 22, 14);
                VGA.draw_string(96, 150, "New File Name:", 0);
                if let Ok(name) = core::str::from_utf8(&self.input_buf[..self.input_len]) {
                    VGA.draw_string(96, 160, name, 1);
                }
                VGA.draw_rect(96 + self.input_len * 6, 166, 5, 2, 1);
            }
            ExplorerMode::CreatingFolder => {
                VGA.draw_rect(94, 148, 210, 22, 11);
                VGA.draw_string(96, 150, "New Folder Name:", 0);
                if let Ok(name) = core::str::from_utf8(&self.input_buf[..self.input_len]) {
                    VGA.draw_string(96, 160, name, 1);
                }
                VGA.draw_rect(96 + self.input_len * 6, 166, 5, 2, 1);
            }
        }
    }

    pub fn handle_input(&mut self, key: char) {
        match self.mode {
            ExplorerMode::Normal => {
                match key {
                    'w' | 'W' => {
                        if self.selected > 0 {
                            self.selected -= 1;
                        }
                    }
                    's' | 'S' => {
                        let count = self.get_children_count();
                        if count > 0 && self.selected < count - 1 {
                            self.selected += 1;
                        }
                    }
                    'n' | 'N' => {
                        // Start creating a file
                        self.mode = ExplorerMode::CreatingFile;
                        self.input_len = 0;
                    }
                    'f' | 'F' => {
                        // Start creating a folder
                        self.mode = ExplorerMode::CreatingFolder;
                        self.input_len = 0;
                    }
                    'd' | 'D' => {
                        // Delete selected entry
                        if let Some(sel_idx) = self.get_child_index(self.selected) {
                            self.entries[sel_idx].active = false;
                            let count = self.get_children_count();
                            if self.selected >= count && count > 0 {
                                self.selected = count - 1;
                            } else if count == 0 {
                                self.selected = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
            ExplorerMode::CreatingFile | ExplorerMode::CreatingFolder => {
                if key >= ' ' && key <= '~' && self.input_len < 18 {
                    self.input_buf[self.input_len] = key as u8;
                    self.input_len += 1;
                }
            }
        }
    }

    pub fn handle_enter(&mut self) {
        match self.mode {
            ExplorerMode::Normal => {
                // Navigate into folder or preview file
                if let Some(sel_idx) = self.get_child_index(self.selected) {
                    if self.entries[sel_idx].is_dir {
                        self.current_dir = sel_idx;
                        self.selected = 0;
                    }
                }
            }
            ExplorerMode::CreatingFile => {
                if self.input_len > 0 && self.entry_count < 24 {
                    let idx = self.entry_count;
                    self.entries[idx].name[..self.input_len].copy_from_slice(&self.input_buf[..self.input_len]);
                    self.entries[idx].name_len = self.input_len;
                    self.entries[idx].is_dir = false;
                    self.entries[idx].parent = self.current_dir;
                    self.entries[idx].active = true;
                    self.entries[idx].content = *b"(new file - edit in UloText)                                                    ";
                    self.entries[idx].content_len = 27;
                    self.entry_count += 1;
                }
                self.mode = ExplorerMode::Normal;
                self.input_len = 0;
            }
            ExplorerMode::CreatingFolder => {
                if self.input_len > 0 && self.entry_count < 24 {
                    let idx = self.entry_count;
                    self.entries[idx].name[..self.input_len].copy_from_slice(&self.input_buf[..self.input_len]);
                    self.entries[idx].name_len = self.input_len;
                    self.entries[idx].is_dir = true;
                    self.entries[idx].parent = self.current_dir;
                    self.entries[idx].active = true;
                    self.entry_count += 1;
                }
                self.mode = ExplorerMode::Normal;
                self.input_len = 0;
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            ExplorerMode::Normal => {
                // Go up one level
                if self.current_dir != 0 {
                    let parent = self.entries[self.current_dir].parent;
                    self.current_dir = parent;
                    self.selected = 0;
                }
            }
            ExplorerMode::CreatingFile | ExplorerMode::CreatingFolder => {
                if self.input_len > 0 {
                    self.input_len -= 1;
                } else {
                    self.mode = ExplorerMode::Normal;
                }
            }
        }
    }

    pub fn handle_escape(&mut self) {
        if self.mode != ExplorerMode::Normal {
            self.mode = ExplorerMode::Normal;
            self.input_len = 0;
        }
    }
}

// ==========================================
// WEB BROWSER with Search Engine
// ==========================================

#[derive(Clone, Copy, PartialEq)]
pub enum BrowserPage {
    Home,
    SearchResults,
    GitHub,
    YouTube,
    Wikipedia,
    CustomPage,
}

#[derive(Clone, Copy)]
pub struct SearchResult {
    pub title: [u8; 40],
    pub title_len: usize,
    pub url: [u8; 50],
    pub url_len: usize,
    pub desc: [u8; 80],
    pub desc_len: usize,
}

fn generate_dynamic_results(query: &[u8]) -> [SearchResult; 3] {
    let mut results = [
        SearchResult { title: [0; 40], title_len: 0, url: [0; 50], url_len: 0, desc: [0; 80], desc_len: 0 },
        SearchResult { title: [0; 40], title_len: 0, url: [0; 50], url_len: 0, desc: [0; 80], desc_len: 0 },
        SearchResult { title: [0; 40], title_len: 0, url: [0; 50], url_len: 0, desc: [0; 80], desc_len: 0 },
    ];

    let q_len = if query.len() > 15 { 15 } else { query.len() };
    let q = &query[..q_len];

    let copy_bytes = |dest: &mut [u8], src: &[u8]| -> usize {
        let len = if src.len() > dest.len() { dest.len() } else { src.len() };
        dest[..len].copy_from_slice(&src[..len]);
        len
    };

    // Result 0: "[Query] Official Site" -> "www.[query].com" -> "Get official news, releases, and information for [query]."
    {
        let mut title_buf = [0u8; 40];
        let mut t_len = 0;
        t_len += copy_bytes(&mut title_buf[t_len..], q);
        t_len += copy_bytes(&mut title_buf[t_len..], b" Official Site");
        results[0].title = title_buf;
        results[0].title_len = t_len;

        let mut url_buf = [0u8; 50];
        let mut u_len = 0;
        u_len += copy_bytes(&mut url_buf[u_len..], b"www.");
        u_len += copy_bytes(&mut url_buf[u_len..], q);
        u_len += copy_bytes(&mut url_buf[u_len..], b".com");
        results[0].url = url_buf;
        results[0].url_len = u_len;

        let mut desc_buf = [0u8; 80];
        let mut d_len = 0;
        d_len += copy_bytes(&mut desc_buf[d_len..], b"Official updates, documentation, download, and features for ");
        d_len += copy_bytes(&mut desc_buf[d_len..], q);
        d_len += copy_bytes(&mut desc_buf[d_len..], b".");
        results[0].desc = desc_buf;
        results[0].desc_len = d_len;
    }

    // Result 1: "[Query] - Wikipedia" -> "en.wikipedia.org/wiki/[query]" -> "Learn about [query] history, origins, and cultural impact on Wiki."
    {
        let mut title_buf = [0u8; 40];
        let mut t_len = 0;
        t_len += copy_bytes(&mut title_buf[t_len..], q);
        t_len += copy_bytes(&mut title_buf[t_len..], b" - Wikipedia");
        results[1].title = title_buf;
        results[1].title_len = t_len;

        let mut url_buf = [0u8; 50];
        let mut u_len = 0;
        u_len += copy_bytes(&mut url_buf[u_len..], b"en.wikipedia.org/wiki/");
        u_len += copy_bytes(&mut url_buf[u_len..], q);
        results[1].url = url_buf;
        results[1].url_len = u_len;

        let mut desc_buf = [0u8; 80];
        let mut d_len = 0;
        d_len += copy_bytes(&mut desc_buf[d_len..], b"Read the comprehensive overview, community records, and timeline of ");
        d_len += copy_bytes(&mut desc_buf[d_len..], q);
        results[1].desc = desc_buf;
        results[1].desc_len = d_len;
    }

    // Result 2: "GitHub - Topics: [query]" -> "github.com/topics/[query]" -> "Explore open source projects and code repositories related to [query]."
    {
        let mut title_buf = [0u8; 40];
        let mut t_len = 0;
        t_len += copy_bytes(&mut title_buf[t_len..], b"GitHub - Topics: ");
        t_len += copy_bytes(&mut title_buf[t_len..], q);
        results[2].title = title_buf;
        results[2].title_len = t_len;

        let mut url_buf = [0u8; 50];
        let mut u_len = 0;
        u_len += copy_bytes(&mut url_buf[u_len..], b"github.com/topics/");
        u_len += copy_bytes(&mut url_buf[u_len..], q);
        results[2].url = url_buf;
        results[2].url_len = u_len;

        let mut desc_buf = [0u8; 80];
        let mut d_len = 0;
        d_len += copy_bytes(&mut desc_buf[d_len..], b"Browse repositories, open-source files, and developer libraries for ");
        d_len += copy_bytes(&mut desc_buf[d_len..], q);
        results[2].desc = desc_buf;
        results[2].desc_len = d_len;
    }

    results
}

pub struct WebBrowser {
    pub url: [u8; 120],
    pub url_len: usize,
    pub current_mode: usize,     // 0 = Sandbox, 1 = Firefox, 2 = Chrome
    pub page: BrowserPage,
    pub result_sel: usize,       // selected search result
    pub custom_title: [u8; 40],
    pub custom_title_len: usize,
    pub custom_desc: [u8; 80],
    pub custom_desc_len: usize,
}

impl WebBrowser {
    pub const fn new() -> Self {
        let mut url = [0; 120];
        url[0] = b'S'; url[1] = b'e'; url[2] = b'a'; url[3] = b'r';
        url[4] = b'c'; url[5] = b'h'; url[6] = b'.'; url[7] = b'.';
        url[8] = b'.';
        WebBrowser {
            url,
            url_len: 9,
            current_mode: 0,
            page: BrowserPage::Home,
            result_sel: 0,
            custom_title: [0; 40],
            custom_title_len: 0,
            custom_desc: [0; 80],
            custom_desc_len: 0,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // 1. Chrome Tab Bar
        VGA.draw_rect(12, 28, 296, 12, 8); // dark gray bar
        
        // Active Tab: Google
        VGA.draw_rect(14, 28, 65, 12, 15); // white background
        VGA.draw_string(18, 30, "Google", 0); // black text
        VGA.draw_string(68, 30, "x", 8); // tab close mark
        
        // Inactive Tab: New Tab
        VGA.draw_rect(84, 28, 60, 12, 7); // light gray
        VGA.draw_string(88, 30, "New Tab", 8); // gray text
        
        // Plus button
        VGA.draw_rect(148, 30, 8, 8, 7);
        VGA.draw_string(149, 30, "+", 15);

        // 2. Chrome URL omnibox area
        VGA.draw_rect(12, 40, 296, 16, 7); // light gray navigation bar
        
        // Navigation arrows (< > R H)
        VGA.draw_string(16, 44, "< >", 15);
        VGA.draw_string(45, 44, "R", 15);
        
        // Rounded Omnibox
        VGA.draw_rect(65, 42, 180, 12, 15); // White input box
        if let Ok(u_str) = core::str::from_utf8(&self.url[..self.url_len]) {
            let show_u = if u_str.len() > 20 { 20 } else { u_str.len() };
            VGA.draw_string(69, 44, &u_str[..show_u], 0);
        }
        
        // Cursor blink inside Omnibox
        let cursor_x = 69 + self.url_len * 6;
        if cursor_x < 240 {
            VGA.draw_rect(cursor_x, 48, 5, 2, 11); // cyan cursor
        }

        // Mini Profile Circle
        VGA.draw_rect(252, 42, 12, 12, 9); // Blue profile avatar frame
        VGA.draw_rect(256, 44, 4, 4, 15); // silhouette head
        VGA.draw_rect(254, 49, 8, 4, 15); // silhouette body

        // Menu stacked dots
        VGA.draw_rect(272, 43, 2, 2, 8);
        VGA.draw_rect(272, 47, 2, 2, 8);
        VGA.draw_rect(272, 51, 2, 2, 8);

        // Chrome Mode Badge
        VGA.draw_rect(280, 42, 24, 12, 2); // green CHRM
        VGA.draw_string(282, 44, "CHRM", 15);

        // Page content
        match self.page {
            BrowserPage::Home => {
                self.draw_home_page();
            }
            BrowserPage::SearchResults => {
                self.draw_search_results();
            }
            BrowserPage::GitHub => {
                self.draw_github_page();
            }
            BrowserPage::YouTube => {
                self.draw_youtube_page();
            }
            BrowserPage::Wikipedia => {
                self.draw_wiki_page();
            }
            BrowserPage::CustomPage => {
                self.draw_custom_page();
            }
        }
    }

    fn draw_home_page(&self) {
        VGA.draw_rect(14, 56, 292, 116, 15);

        // Google multi-colored logo
        let lx = 120;
        let ly = 68;
        VGA.draw_string(lx, ly, "G", 1);     // Blue
        VGA.draw_string(lx + 8, ly, "o", 4); // Red
        VGA.draw_string(lx + 16, ly, "o", 14); // Yellow
        VGA.draw_string(lx + 24, ly, "g", 1); // Blue
        VGA.draw_string(lx + 32, ly, "l", 2); // Green
        VGA.draw_string(lx + 40, ly, "e", 4); // Red

        // Omnibox Google search bar shadow card
        VGA.draw_rect(50, 90, 220, 16, 7); // Light gray input box
        VGA.draw_rect(51, 91, 218, 14, 15); // White inner
        VGA.draw_string(56, 94, "Search Google or type URL...", 8);
        
        // Search buttons
        VGA.draw_rect(65, 116, 90, 14, 7);
        VGA.draw_string(69, 119, "Google Search", 8);

        VGA.draw_rect(165, 116, 90, 14, 7);
        VGA.draw_string(169, 119, "I'm Feeling Lucky", 8);

        VGA.draw_string(54, 144, "Google offered in: English, Rust, QEMU", 8);
        VGA.draw_string(14, 160, "[Type] Search  [Enter] Go  [Esc] Desktop", 8);
    }

    fn draw_search_results(&self) {
        VGA.draw_rect(14, 46, 292, 2, 9);

        // Results header
        VGA.draw_string(16, 52, "UloSearch Results for:", 8);
        if let Ok(query) = core::str::from_utf8(&self.url[..self.url_len]) {
            let show = if query.len() > 20 { 20 } else { query.len() };
            VGA.draw_string(190, 52, &query[..show], 1);
        }

        // Determine category from URL content
        let query_bytes = &self.url[..self.url_len];
        let has = |needle: &[u8]| -> bool {
            for i in 0..self.url_len {
                if i + needle.len() <= self.url_len {
                    let mut found = true;
                    for j in 0..needle.len() {
                        let a = query_bytes[i + j];
                        let b = needle[j];
                        // case insensitive compare
                        let la = if a >= b'A' && a <= b'Z' { a + 32 } else { a };
                        let lb = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                        if la != lb { found = false; break; }
                    }
                    if found { return true; }
                }
            }
            false
        };

        // Generate 3 results based on keywords
        let results: [(&str, &str, &str); 3];
        let has_any = has(b"rust") || has(b"kernel") || has(b"os") || has(b"code") ||
                      has(b"game") || has(b"doom") || has(b"play") ||
                      has(b"weather") || has(b"temp") || has(b"rain") ||
                      has(b"music") || has(b"song") || has(b"audio") ||
                      has(b"git") || has(b"hub") ||
                      has(b"you") || has(b"video") || has(b"tube") ||
                      has(b"wiki");

        if has(b"rust") || has(b"kernel") || has(b"os") || has(b"code") {
            results = [
                ("The Rust Language", "rust-lang.org", "Fast and safe systems programming"),
                ("Rust by Example", "doc.rust-lang.org/stable", "Hands-on Rust tutorials and code"),
                ("Awesome Rust Repos", "github.com/rust-unofficial", "Curated Rust frameworks and libs"),
            ];
        } else if has(b"game") || has(b"doom") || has(b"play") {
            results = [
                ("DOOM Classic Online", "dos.zone/doom", "Play original DOOM in the browser"),
                ("Retro Game Archive", "archive.org/details/games", "Free classic DOS games collection"),
                ("Game Dev with Rust", "arewegameyet.rs", "Rust game development resources"),
            ];
        } else if has(b"weather") || has(b"temp") || has(b"rain") {
            results = [
                ("Weather Forecast", "weather.gov", "Official live weather forecasts"),
                ("World Weather Map", "openweathermap.org", "Real-time global weather data API"),
                ("Storm Tracker Live", "windy.com", "Interactive wind and rain maps"),
            ];
        } else if has(b"music") || has(b"song") || has(b"audio") {
            results = [
                ("Free Music Library", "freemusicarchive.org", "Royalty-free music downloads"),
                ("Online Synthesizer", "webaudioapi.com/synth", "Web-based audio synthesizer tool"),
                ("Music Theory Guide", "musictheory.net", "Learn scales, chords, and more"),
            ];
        } else if has(b"git") || has(b"hub") {
            results = [
                ("UloOS Repository", "github.com/Aqua-code750", "Bare-metal Rust OS for QEMU x86"),
                ("GitHub Trending", "github.com/trending", "Popular open source projects"),
                ("Git Documentation", "git-scm.com/docs", "Official Git reference manual"),
            ];
        } else if has(b"you") || has(b"video") || has(b"tube") {
            results = [
                ("YouTube Video Hub", "youtube.com", "World largest video platform"),
                ("Rust OS Tutorial", "youtube.com/watch?v=os1", "Build a kernel in Rust from zero"),
                ("QEMU Setup Guide", "youtube.com/watch?v=qemu", "How to set up QEMU for bare metal"),
            ];
        } else if has(b"wiki") {
            results = [
                ("Wikipedia", "en.wikipedia.org", "The free online encyclopedia"),
                ("Hobby OS Article", "wiki.osdev.org", "OSDev community wiki resources"),
                ("Rust Wikipedia", "en.wikipedia.org/Rust", "Rust programming language article"),
            ];
        } else {
            results = [("", "", ""); 3];
        }

        let mut ry = 66;
        for i in 0..3 {
            let is_sel = i == self.result_sel;
            if is_sel {
                VGA.draw_rect(14, ry - 2, 292, 26, 7);
            }

            if has_any {
                VGA.draw_string(20, ry, results[i].0, 9);        // title in blue
                VGA.draw_string(20, ry + 10, results[i].1, 2);   // URL in green
                VGA.draw_string(20, ry + 18, results[i].2, 8);   // desc in gray
            } else {
                let dyn_res = generate_dynamic_results(query_bytes);
                if let Ok(title) = core::str::from_utf8(&dyn_res[i].title[..dyn_res[i].title_len]) {
                    VGA.draw_string(20, ry, title, 9);
                }
                if let Ok(url_str) = core::str::from_utf8(&dyn_res[i].url[..dyn_res[i].url_len]) {
                    VGA.draw_string(20, ry + 10, url_str, 2);
                }
                if let Ok(desc) = core::str::from_utf8(&dyn_res[i].desc[..dyn_res[i].desc_len]) {
                    VGA.draw_string(20, ry + 18, desc, 8);
                }
            }

            ry += 28;
        }

        VGA.draw_string(14, 160, "[Enter]Open [Tab]Select [Esc]Home [Click]UI", 8);
    }

    fn draw_custom_page(&self) {
        let url_bytes = &self.url[..self.url_len];
        let contains_url = |needle: &[u8]| -> bool {
            for i in 0..self.url_len {
                if i + needle.len() <= self.url_len {
                    let mut found = true;
                    for j in 0..needle.len() {
                        let a = url_bytes[i + j];
                        let b = needle[j];
                        let la = if a >= b'A' && a <= b'Z' { a + 32 } else { a };
                        let lb = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                        if la != lb { found = false; break; }
                    }
                    if found { return true; }
                }
            }
            false
        };

        if contains_url(b"rust") {
            // Orange Header (Color 12 / 14)
            VGA.draw_rect(14, 48, 292, 12, 12);
            VGA.draw_string(18, 50, "Rust Programming Language", 15);

            VGA.draw_string(16, 68, "Empowering developers to build", 0);
            VGA.draw_string(16, 78, "reliable and efficient software.", 0);

            VGA.draw_rect(16, 92, 288, 1, 8); // Separator

            // Highlights
            VGA.draw_string(16, 98, "Why Rust?", 12);
            VGA.draw_string(16, 110, "- Performance: Blazing fast & thin", 8);
            VGA.draw_string(16, 120, "- Reliability: Memory-safe compiler", 8);
            VGA.draw_string(16, 130, "- Productivity: Package manager (cargo)", 8);

            // Button / Link
            VGA.draw_rect(16, 144, 90, 12, 12);
            VGA.draw_string(20, 146, "Get Started", 15);
        } else if contains_url(b"doom") || contains_url(b"game") || contains_url(b"play") {
            // Red Header (Color 4)
            VGA.draw_rect(14, 48, 292, 12, 4);
            VGA.draw_string(18, 50, "DOOM Classic Online Player", 15);

            VGA.draw_string(16, 68, "Play Ultimate DOOM / DOOM II", 4);
            VGA.draw_string(16, 78, "in pure bare-metal DOSBox!", 0);

            VGA.draw_rect(16, 92, 288, 1, 8);

            VGA.draw_string(16, 98, "Virtual Sandbox Controller", 2);
            VGA.draw_string(16, 110, "Use arrows to run, space to door.", 8);
            VGA.draw_string(16, 122, "Press F8 to open Desktop DOOM!", 14);

            VGA.draw_rect(16, 140, 120, 12, 2);
            VGA.draw_string(20, 142, "Launch Game Client", 0);
        } else if contains_url(b"weather") || contains_url(b"temp") || contains_url(b"rain") {
            // Light Blue Header (Color 9)
            VGA.draw_rect(14, 48, 292, 12, 9);
            VGA.draw_string(18, 50, "National Weather Service", 15);

            VGA.draw_string(16, 68, "Live Weather Forecast & Alerts", 1);
            VGA.draw_string(16, 80, "No active severe storm warnings.", 2);

            VGA.draw_rect(16, 92, 288, 1, 8);

            VGA.draw_string(16, 98, "Current Status (Worldwide)", 8);
            VGA.draw_string(16, 110, "Humidity: 45%  Wind: 12 km/h", 0);
            VGA.draw_string(16, 122, "Temperature: 22 C / Sunny Clear", 0);
        } else if contains_url(b"music") || contains_url(b"song") || contains_url(b"audio") {
            // Magenta Header (Color 13)
            VGA.draw_rect(14, 48, 292, 12, 13);
            VGA.draw_string(18, 50, "Free Music Archive", 15);

            VGA.draw_string(16, 68, "Discover free audio tracks", 13);
            VGA.draw_string(16, 78, "and synthesizers online.", 0);

            VGA.draw_rect(16, 92, 288, 1, 8);

            VGA.draw_string(16, 98, "Interactive Audio Test", 11);
            VGA.draw_string(16, 110, "Chimes and speaker registers OK.", 8);
            VGA.draw_string(16, 122, "Use UloMusic app to sequence tones.", 8);
        } else {
            // Elegant Slate Header (Color 8)
            VGA.draw_rect(14, 48, 292, 12, 8);
            if let Ok(title) = core::str::from_utf8(&self.custom_title[..self.custom_title_len]) {
                let show_t = if title.len() > 30 { 30 } else { title.len() };
                VGA.draw_string(18, 50, &title[..show_t], 15);
            }

            VGA.draw_string(16, 68, "Official Homepage Portal", 1);
            VGA.draw_string(16, 78, "Official web content and documentation.", 8);

            VGA.draw_rect(16, 92, 288, 1, 7);

            // Render custom description nicely
            VGA.draw_string(16, 98, "Site Summary:", 8);
            if let Ok(desc) = core::str::from_utf8(&self.custom_desc[..self.custom_desc_len]) {
                let wrap = 30;
                if desc.len() > wrap * 2 {
                    VGA.draw_string(16, 110, &desc[..wrap], 0);
                    VGA.draw_string(16, 120, &desc[wrap..wrap*2], 0);
                    VGA.draw_string(16, 130, &desc[wrap*2..], 0);
                } else if desc.len() > wrap {
                    VGA.draw_string(16, 110, &desc[..wrap], 0);
                    VGA.draw_string(16, 120, &desc[wrap..], 0);
                } else {
                    VGA.draw_string(16, 110, desc, 0);
                }
            }
            
            VGA.draw_string(16, 144, "Links: [Home]  [About]  [Contact]", 9);
        }

        VGA.draw_string(16, 160, "[Esc]Go Back Home", 8);
    }

    fn draw_github_page(&self) {
        VGA.draw_rect(14, 48, 292, 12, 8);
        VGA.draw_string(18, 50, "GitHub - Aqua-code750/uloos-1.2", 15);

        VGA.draw_string(16, 68, "Repository: uloos-1.2 [Public]", 1);
        VGA.draw_string(16, 82, "Language: Rust 100%", 2);
        VGA.draw_string(180, 82, "Stars: 1200", 14);

        VGA.draw_rect(16, 98, 288, 38, 7);
        VGA.draw_string(20, 102, "#![no_std]", 4);
        VGA.draw_string(20, 114, "fn _start() -> ! { loop {} }", 1);
        VGA.draw_string(20, 126, "// Bare-metal QEMU bootloader", 8);

        VGA.draw_string(16, 148, "[Enter]Search [H]Home [M]Mode", 8);
    }

    fn draw_youtube_page(&self) {
        VGA.draw_rect(14, 48, 292, 12, 12);
        VGA.draw_string(18, 50, "YouTube Studio Player", 15);

        // Video player area
        VGA.draw_rect(20, 68, 120, 65, 0);
        VGA.draw_rect(75, 95, 10, 10, 12); // play button
        VGA.draw_string(22, 122, "--> playing clip", 10);

        VGA.draw_string(148, 68, "Rust OS Tutorial", 1);
        VGA.draw_string(148, 82, "By: Aqua-code750", 8);
        VGA.draw_string(148, 96, "Views: 2.4M", 8);

        VGA.draw_rect(148, 115, 60, 4, 8);
        VGA.draw_rect(148, 115, 45, 4, 10);

        VGA.draw_string(16, 148, "[Enter]Search [H]Home [M]Mode", 8);
    }

    fn draw_wiki_page(&self) {
        VGA.draw_rect(14, 48, 292, 12, 7);
        VGA.draw_string(18, 50, "Wikipedia - Free Encyclopedia", 0);

        VGA.draw_string(16, 68, "Hobby Operating System", 1);
        VGA.draw_string(16, 82, "--------------------------", 8);
        VGA.draw_string(16, 96, "An OS created from scratch", 0);
        VGA.draw_string(16, 110, "to study kernel design,", 0);
        VGA.draw_string(16, 124, "x86 paging, and custom BIOS.", 0);

        VGA.draw_string(16, 148, "[Enter]Search [H]Home [M]Mode", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            '\t' => {
                // Tab cycles search results
                if self.page == BrowserPage::SearchResults {
                    self.result_sel = (self.result_sel + 1) % 3;
                }
            }
            _ => {
                // Type into URL bar
                if self.url_len < 28 {
                    // Clear default text on first real keystroke
                    if self.page == BrowserPage::Home && self.url_len == 9 {
                        let starts_with_search = self.url[0] == b'S' && self.url[1] == b'e';
                        if starts_with_search {
                            self.url_len = 0;
                        }
                    }
                    self.url[self.url_len] = key as u8;
                    self.url_len += 1;
                }
            }
        }
    }

    pub fn go_home(&mut self) {
        self.page = BrowserPage::Home;
        self.url_len = 0;
        let home = b"Search...";
        self.url[..home.len()].copy_from_slice(home);
        self.url_len = home.len();
        self.result_sel = 0;
    }

    pub fn handle_click(&mut self, x: usize, y: usize) {
        // Mode badge click
        if x >= 260 && x <= 302 && y >= 30 && y <= 44 {
            self.current_mode = (self.current_mode + 1) % 3;
            return;
        }

        // Home button click (click "<" or "H" in URL bar)
        if x >= 16 && x <= 39 && y >= 30 && y <= 44 {
            self.go_home();
            return;
        }

        // Search results click
        if self.page == BrowserPage::SearchResults {
            let mut ry = 66;
            for i in 0..3 {
                if x >= 14 && x <= 306 && y >= (ry - 2) && y <= (ry + 24) {
                    self.result_sel = i;
                    self.handle_enter();
                    break;
                }
                ry += 28;
            }
        }
    }

    pub fn handle_enter(&mut self) {
        if self.page == BrowserPage::SearchResults {
            // Open the selected result!
            let query_bytes = &self.url[..self.url_len];
            let has = |needle: &[u8]| -> bool {
                for i in 0..self.url_len {
                    if i + needle.len() <= self.url_len {
                        let mut found = true;
                        for j in 0..needle.len() {
                            let a = query_bytes[i + j];
                            let b = needle[j];
                            let la = if a >= b'A' && a <= b'Z' { a + 32 } else { a };
                            let lb = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                            if la != lb { found = false; break; }
                        }
                        if found { return true; }
                    }
                }
                false
            };

            let (title, url_str, desc) = if has(b"rust") || has(b"kernel") || has(b"os") || has(b"code") {
                let res = [
                    ("The Rust Language", "rust-lang.org", "Fast and safe systems programming"),
                    ("Rust by Example", "doc.rust-lang.org/stable", "Hands-on Rust tutorials and code"),
                    ("Awesome Rust Repos", "github.com/rust-unofficial", "Curated Rust frameworks and libs"),
                ];
                res[self.result_sel]
            } else if has(b"game") || has(b"doom") || has(b"play") {
                let res = [
                    ("DOOM Classic Online", "dos.zone/doom", "Play original DOOM in the browser"),
                    ("Retro Game Archive", "archive.org/details/games", "Free classic DOS games collection"),
                    ("Game Dev with Rust", "arewegameyet.rs", "Rust game development resources"),
                ];
                res[self.result_sel]
            } else if has(b"weather") || has(b"temp") || has(b"rain") {
                let res = [
                    ("Weather Forecast", "weather.gov", "Official live weather forecasts"),
                    ("World Weather Map", "openweathermap.org", "Real-time global weather data API"),
                    ("Storm Tracker Live", "windy.com", "Interactive wind and rain maps"),
                ];
                res[self.result_sel]
            } else if has(b"music") || has(b"song") || has(b"audio") {
                let res = [
                    ("Free Music Library", "freemusicarchive.org", "Royalty-free music downloads"),
                    ("Online Synthesizer", "webaudioapi.com/synth", "Web-based audio synthesizer tool"),
                    ("Music Theory Guide", "musictheory.net", "Learn scales, chords, and more"),
                ];
                res[self.result_sel]
            } else if has(b"git") || has(b"hub") {
                let res = [
                    ("UloOS Repository", "github.com/Aqua-code750", "Bare-metal Rust OS for QEMU x86"),
                    ("GitHub Trending", "github.com/trending", "Popular open source projects"),
                    ("Git Documentation", "git-scm.com/docs", "Official Git reference manual"),
                ];
                res[self.result_sel]
            } else if has(b"you") || has(b"video") || has(b"tube") {
                let res = [
                    ("YouTube Video Hub", "youtube.com", "World largest video platform"),
                    ("Rust OS Tutorial", "youtube.com/watch?v=os1", "Build a kernel in Rust from zero"),
                    ("QEMU Setup Guide", "youtube.com/watch?v=qemu", "How to set up QEMU for bare metal"),
                ];
                res[self.result_sel]
            } else if has(b"wiki") {
                let res = [
                    ("Wikipedia", "en.wikipedia.org", "The free online encyclopedia"),
                    ("Hobby OS Article", "wiki.osdev.org", "OSDev community wiki resources"),
                    ("Rust Wikipedia", "en.wikipedia.org/Rust", "Rust programming language article"),
                ];
                res[self.result_sel]
            } else {
                let dyn_res = generate_dynamic_results(query_bytes);
                let sel_res = &dyn_res[self.result_sel];
                
                self.custom_title_len = sel_res.title_len;
                self.custom_title[..sel_res.title_len].copy_from_slice(&sel_res.title[..sel_res.title_len]);
                
                self.custom_desc_len = sel_res.desc_len;
                self.custom_desc[..sel_res.desc_len].copy_from_slice(&sel_res.desc[..sel_res.desc_len]);
                
                self.url_len = sel_res.url_len;
                self.url[..sel_res.url_len].copy_from_slice(&sel_res.url[..sel_res.url_len]);
                
                self.page = BrowserPage::CustomPage;
                return;
            };

            self.custom_title_len = title.len();
            self.custom_title[..title.len()].copy_from_slice(title.as_bytes());
            self.custom_desc_len = desc.len();
            self.custom_desc[..desc.len()].copy_from_slice(desc.as_bytes());
            
            self.url_len = url_str.len();
            self.url[..url_str.len()].copy_from_slice(url_str.as_bytes());

            if url_str.contains("github.com") {
                self.page = BrowserPage::GitHub;
            } else if url_str.contains("youtube.com") {
                self.page = BrowserPage::YouTube;
            } else if url_str.contains("wikipedia.org") || url_str.contains("wiki") {
                self.page = BrowserPage::Wikipedia;
            } else {
                self.page = BrowserPage::CustomPage;
            }
            return;
        }

        if self.url_len == 0 {
            return;
        }

        let query_bytes = &self.url[..self.url_len];
        let has = |needle: &[u8]| -> bool {
            for i in 0..self.url_len {
                if i + needle.len() <= self.url_len {
                    let mut found = true;
                    for j in 0..needle.len() {
                        let a = query_bytes[i + j];
                        let b = needle[j];
                        let la = if a >= b'A' && a <= b'Z' { a + 32 } else { a };
                        let lb = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                        if la != lb { found = false; break; }
                    }
                    if found { return true; }
                }
            }
            false
        };

        if has(b"github.com") {
            self.page = BrowserPage::GitHub;
        } else if has(b"youtube.com") {
            self.page = BrowserPage::YouTube;
        } else if has(b"wikipedia") {
            self.page = BrowserPage::Wikipedia;
        } else {
            self.page = BrowserPage::SearchResults;
            self.result_sel = 0;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.url_len > 0 {
            self.url_len -= 1;
        }
    }
}
