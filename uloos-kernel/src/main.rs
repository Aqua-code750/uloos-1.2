#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;
mod keyboard;
mod apps;
mod mouse;
mod vga_driver;
mod vga_mode;
mod sound;

use vga_buffer::{Color, ColorCode, WRITER};
use keyboard::{get_key, DecodedKey};
use apps::bash::BashShell;
use apps::explorer::{FileExplorer, WebBrowser};
use apps::office::{UloText, UloSlides, UloNumbers, UloMail};
use apps::doom::TuiDoom;
use vga_driver::VGA;

use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveApp {
    Desktop,
    Bash,
    Explorer,
    Text,
    Slides,
    Numbers,
    Mail,
    Browser,
    Doom,
    Settings,
    Store,
}

pub static BASH: Mutex<BashShell> = Mutex::new(BashShell::new());
pub static EXPLORER: Mutex<FileExplorer> = Mutex::new(FileExplorer::new());
pub static TEXT_EDITOR: Mutex<UloText> = Mutex::new(UloText::new());
pub static SLIDES: Mutex<UloSlides> = Mutex::new(UloSlides::new());
pub static NUMBERS: Mutex<UloNumbers> = Mutex::new(UloNumbers::new());
pub static MAIL: Mutex<UloMail> = Mutex::new(UloMail::new());
pub static BROWSER: Mutex<WebBrowser> = Mutex::new(WebBrowser::new());
pub static DOOM_GAME: Mutex<TuiDoom> = Mutex::new(TuiDoom::new());
pub static SYSTEM_SETTINGS: Mutex<SystemSettings> = Mutex::new(SystemSettings::new());
pub static APP_STORE: Mutex<AppStore> = Mutex::new(AppStore::new());

// Global mouse cursor coordinate state
pub static CURSOR_X: Mutex<usize> = Mutex::new(160);
pub static CURSOR_Y: Mutex<usize> = Mutex::new(100);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Brief motherboard/interrupt stabilization delay to let virtual ports settle
    for _ in 0..100_000 {
        unsafe { core::arch::asm!("nop") }
    }

    // Seed UloText default notes
    {
        let mut editor = TEXT_EDITOR.lock();
        let initial = "Welcome to UloText! Write school notes or developer documentation here...\n";
        editor.buffer[..initial.len()].copy_from_slice(initial.as_bytes());
        editor.len = initial.len();
    }

    // 2. Switch CPU graphic mode to high-resolution mode 0x13 instantly
    vga_mode::set_vga_mode_320x200();

    let mut current_app = ActiveApp::Desktop;
    let mut start_menu_open = false;

    // 3. Render the Windows 98 desktop layout immediately so the user sees it instantly
    draw_win95_graphics_desktop(current_app, start_menu_open);

    // 4. Initialize PS/2 mouse hardware driver AFTER screen setup is stable!
    mouse::mouse_init();

    // 5. Play nostalgic startup chime sound while desktop is visible
    sound::play_startup_sound();

    loop {
        // Poll PS/2 Mouse movements until buffer is empty to eliminate cursor lag/drift
        let sens = SYSTEM_SETTINGS.lock().mouse_sensitivity;
        while mouse::poll_mouse(sens) {}
        
        unsafe {
            let mx = mouse::MOUSE_X as usize;
            let my = mouse::MOUSE_Y as usize;
            
            *CURSOR_X.lock() = if mx >= 315 { 315 } else { mx };
            *CURSOR_Y.lock() = if my >= 195 { 195 } else { my };
        }

        // Draw Windows 95 Graphical Desktop
        draw_win95_graphics_desktop(current_app, start_menu_open);

        // Draw Active application viewport
        match current_app {
            ActiveApp::Desktop => {}
            ActiveApp::Bash => {
                draw_gui_window("MS-DOS Prompt", 10, 15, 300, 160);
                BASH.lock().draw();
            }
            ActiveApp::Explorer => {
                draw_gui_window("My Computer", 10, 15, 300, 160);
                EXPLORER.lock().draw();
            }
            ActiveApp::Text => {
                draw_gui_window("UloText Editor", 10, 15, 300, 160);
                TEXT_EDITOR.lock().draw();
            }
            ActiveApp::Slides => {
                draw_gui_window("UloSlides Creator", 10, 15, 300, 160);
                SLIDES.lock().draw();
            }
            ActiveApp::Numbers => {
                draw_gui_window("UloNumbers Sheets", 10, 15, 300, 160);
                NUMBERS.lock().draw();
            }
            ActiveApp::Mail => {
                draw_gui_window("UloMail Exchange", 10, 15, 300, 160);
                MAIL.lock().draw();
            }
            ActiveApp::Browser => {
                draw_gui_window("UloBrowser Text Mode", 10, 15, 300, 160);
                BROWSER.lock().draw();
            }
            ActiveApp::Doom => {
                draw_gui_window("TUI DOOM", 10, 15, 300, 160);
                DOOM_GAME.lock().draw();
            }
            ActiveApp::Settings => {
                draw_gui_window("System Settings", 10, 15, 300, 160);
                SYSTEM_SETTINGS.lock().draw();
            }
            ActiveApp::Store => {
                draw_gui_window("UloOS App Store", 10, 15, 300, 160);
                APP_STORE.lock().draw();
            }
        }

        // Draw high-resolution cursor pixel mapping pointer (Standard Arrow cursor)
        let cx = *CURSOR_X.lock();
        let cy = *CURSOR_Y.lock();
        VGA.draw_cursor(cx, cy);

        // Swap the backbuffer into standard physical VGA memory address
        VGA.swap_buffers();

        // Mouse click handler
        unsafe {
            if mouse::LEFT_CLICK {
                mouse::LEFT_CLICK = false;
                let click_x = cx;
                let click_y = cy;

                // Close active window [X] click region
                if current_app != ActiveApp::Desktop && click_y >= 16 && click_y <= 24 && click_x >= 295 && click_x <= 308 {
                    current_app = ActiveApp::Desktop;
                }
                // Start button click (Centered at x = 65 to 95)
                else if click_y >= 185 && click_y <= 198 && click_x >= 63 && click_x <= 97 {
                    start_menu_open = !start_menu_open;
                }
                // Start Menu popups
                else if start_menu_open {
                    if click_x >= 50 && click_x <= 270 {
                        if click_y >= 60 && click_y <= 70 {
                            if click_x < 150 { current_app = ActiveApp::Bash; } 
                            else { current_app = ActiveApp::Explorer; }
                            start_menu_open = false;
                        }
                        else if click_y >= 72 && click_y <= 82 {
                            if click_x < 150 { current_app = ActiveApp::Text; } 
                            else { current_app = ActiveApp::Slides; }
                            start_menu_open = false;
                        }
                        else if click_y >= 84 && click_y <= 94 {
                            if click_x < 150 { current_app = ActiveApp::Numbers; } 
                            else { current_app = ActiveApp::Mail; }
                            start_menu_open = false;
                        }
                        else if click_y >= 96 && click_y <= 106 {
                            if click_x < 150 { current_app = ActiveApp::Browser; } 
                            else { current_app = ActiveApp::Doom; }
                            start_menu_open = false;
                        }
                        else if click_y >= 108 && click_y <= 118 {
                            if click_x < 150 { current_app = ActiveApp::Store; } 
                            else { current_app = ActiveApp::Settings; }
                            start_menu_open = false;
                        }
                    }
                }
                // Desktop Shortcut clicks (Refined for small size icons in high-density layout)
                else if current_app == ActiveApp::Desktop {
                    if click_x >= 10 && click_x <= 120 {
                        if click_y >= 15 && click_y <= 27 { current_app = ActiveApp::Explorer; }
                        else if click_y >= 35 && click_y <= 47 { current_app = ActiveApp::Bash; }
                        else if click_y >= 55 && click_y <= 67 { current_app = ActiveApp::Text; }
                        else if click_y >= 75 && click_y <= 87 { current_app = ActiveApp::Doom; }
                        else if click_y >= 95 && click_y <= 107 { current_app = ActiveApp::Store; }
                    }
                }
            }
        }

        // Read Keyboard events
        if let Some(key) = get_key() {
            match key {
                DecodedKey::Ascii(c) => {
                    match current_app {
                        ActiveApp::Bash => BASH.lock().add_char(c),
                        ActiveApp::Text => TEXT_EDITOR.lock().handle_key(c),
                        ActiveApp::Numbers => NUMBERS.lock().handle_input(c),
                        ActiveApp::Doom => DOOM_GAME.lock().handle_input(c),
                        ActiveApp::Settings => SYSTEM_SETTINGS.lock().handle_input(c),
                        ActiveApp::Store => APP_STORE.lock().handle_input(c),
                        _ => {}
                    }
                }
                DecodedKey::Backspace => {
                    match current_app {
                        ActiveApp::Bash => BASH.lock().handle_backspace(),
                        ActiveApp::Text => TEXT_EDITOR.lock().handle_backspace(),
                        _ => {}
                    }
                }
                DecodedKey::Enter => {
                    match current_app {
                        ActiveApp::Bash => {
                            let mut b = BASH.lock();
                            if let Some(cmd) = b.handle_enter() {
                                match cmd {
                                    "doom" => current_app = ActiveApp::Doom,
                                    "office" => current_app = ActiveApp::Text,
                                    "explorer" => current_app = ActiveApp::Explorer,
                                    "exit" => current_app = ActiveApp::Desktop,
                                    _ => {}
                                }
                            }
                        }
                        ActiveApp::Explorer => EXPLORER.lock().move_down(),
                        ActiveApp::Slides => SLIDES.lock().next(),
                        ActiveApp::Mail => MAIL.lock().toggle(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Delay to regulate speed inside QEMU CPU environment - reduced to 50 for ultra-fast response
        for _ in 0..50 {
            unsafe { core::arch::asm!("nop") }
        }
    }
}


unsafe fn rtc_read(reg: u8) -> u8 {
    // Write register address to port 0x70
    core::arch::asm!("out 0x70, al", in("al") reg, options(nomem, nostack, preserves_flags));
    // Read register byte from port 0x71
    let val: u8;
    core::arch::asm!("in al, 0x71", out("al") val, options(nomem, nostack, preserves_flags));
    val
}

unsafe fn rtc_is_updating() -> bool {
    (rtc_read(0x0A) & 0x80) != 0
}

fn get_rtc_time() -> (u8, u8, u8) {
    unsafe {
        let mut timeout = 1000;
        while rtc_is_updating() && timeout > 0 {
            timeout -= 1;
        }
        
        let mut seconds = rtc_read(0x00);
        let mut minutes = rtc_read(0x02);
        let mut hours = rtc_read(0x04);
        let reg_b = rtc_read(0x0B);

        // Convert from BCD (Binary Coded Decimal) to binary if bit 2 of Register B is clear
        if (reg_b & 0x04) == 0 {
            seconds = ((seconds / 16) * 10) + (seconds % 16);
            minutes = ((minutes / 16) * 10) + (minutes % 16);
            hours = ((hours / 16) * 10) + (hours % 16);
        }

        (hours, minutes, seconds)
    }
}

fn get_local_time() -> (u8, u8, u8) {
    let (h_raw, m_raw, s) = get_rtc_time();
    
    // Get timezone offset in minutes: UTC(0), EST(-300), PST(-480), CET(60), IST(330), CST(480), JST(540)
    let offsets = [0, -300, -480, 60, 330, 480, 540];
    let offset_minutes = offsets[SYSTEM_SETTINGS.lock().timezone_index];
    
    let mut total_minutes = (h_raw as i32) * 60 + (m_raw as i32) + offset_minutes;
    
    // Handle modular underflow/overflow dynamically
    if total_minutes < 0 {
        total_minutes += 1440;
    }
    total_minutes = total_minutes % 1440;
    
    let h_local = (total_minutes / 60) as u8;
    let m_local = (total_minutes % 60) as u8;
    
    (h_local, m_local, s)
}

// Draws the modern centered Windows 11 style Fluent desktop in 320x200 VGA resolution
fn draw_win95_graphics_desktop(active: ActiveApp, start_open: bool) {
    // 1. Modern deep dark slate desktop background (completely flat dark mode)
    VGA.draw_rect(0, 0, 320, 185, 0); 

    // Draw premium carbon-grid dots for high-end aesthetic
    for gy in (4..180).step_by(10) {
        for gx in (4..316).step_by(10) {
            VGA.draw_rect(gx, gy, 1, 1, 8); // subtle dark gray dots
        }
    }

    // 2. Modern flat vertical desktop shortcuts (glowing white text, aligned to click boundaries)
    draw_gui_icon("FILES", 10, 15, 10);
    draw_gui_icon("DOS", 10, 35, 14);
    draw_gui_icon("WRITE", 10, 55, 11);
    draw_gui_icon("DOOM", 10, 75, 12);
    draw_gui_icon("STORE", 10, 95, 13);

    // Modern Floating Clock Widget at Top-Right
    VGA.draw_rect(248, 12, 62, 14, 8);  // Outline border
    VGA.draw_rect(249, 13, 60, 12, 0);  // Dark interior
    
    let (h, m, _) = get_local_time();
    let ampm = if h >= 12 { "PM" } else { "AM" };
    let mut h12 = h % 12;
    if h12 == 0 { h12 = 12; }

    let mut time_str = [b'0'; 8];
    time_str[0] = b'0' + (h12 / 10);
    time_str[1] = b'0' + (h12 % 10);
    time_str[2] = b':';
    time_str[3] = b'0' + (m / 10);
    time_str[4] = b'0' + (m % 10);
    time_str[5] = b' ';
    time_str[6] = ampm.as_bytes()[0];
    time_str[7] = ampm.as_bytes()[1];

    if let Ok(s) = core::str::from_utf8(&time_str) {
        VGA.draw_string(252, 15, s, 15);
    }

    // 3. Floating centered dock in modern Fluent design (charcoal pill with glowing blue border!)
    VGA.draw_rect(78, 186, 164, 12, 11); // Neon Light Cyan dock boundary glow!
    VGA.draw_rect(79, 187, 162, 10, 8);  // Charcoal dock body
    VGA.draw_rect(80, 188, 160, 8, 8);   // Flat interior

    // Centered modern Start Icon (ToT)
    VGA.draw_rect(84, 188, 20, 8, 9); // Sleek blue icon
    VGA.draw_string(86, 188, "ToT", 15);

    // Centered active app representation
    let icon_offset_x = 115;
    match active {
        ActiveApp::Desktop => VGA.draw_string(icon_offset_x, 188, "[Desktop]", 11),
        ActiveApp::Bash => VGA.draw_string(icon_offset_x, 188, "[MS-DOS]", 11),
        ActiveApp::Explorer => VGA.draw_string(icon_offset_x, 188, "[Explorer]", 11),
        ActiveApp::Text => VGA.draw_string(icon_offset_x, 188, "[UloText]", 11),
        ActiveApp::Slides => VGA.draw_string(icon_offset_x, 188, "[Slides]", 11),
        ActiveApp::Numbers => VGA.draw_string(icon_offset_x, 188, "[Numbers]", 11),
        ActiveApp::Mail => VGA.draw_string(icon_offset_x, 188, "[UloMail]", 11),
        ActiveApp::Browser => VGA.draw_string(icon_offset_x, 188, "[Browser]", 11),
        ActiveApp::Doom => VGA.draw_string(icon_offset_x, 188, "[DOOM]", 11),
        ActiveApp::Settings => VGA.draw_string(icon_offset_x, 188, "[Settings]", 11),
        ActiveApp::Store => VGA.draw_string(icon_offset_x, 188, "[Store]", 11),
    }

    // UloOS 1.2 Modern Centered Start Menu overlay (placed directly above the centered Start button)
    if start_open {
        VGA.draw_rect(50, 42, 220, 142, 11); // glowing neon cyan border outline
        VGA.draw_rect(51, 43, 218, 140, 8);  // Charcoal solid body (no retro bevels!)
        VGA.draw_rect(52, 44, 216, 138, 8);

        VGA.draw_rect(55, 47, 210, 10, 9); // Sleek modern blue header
        VGA.draw_string(58, 48, "UloOS 1.2 Pinned Apps", 15);

        VGA.draw_string(58, 62, "1. Bash Shell  2. Explorer", 15);
        VGA.draw_string(58, 74, "3. UloText     4. UloSlides", 15);
        VGA.draw_string(58, 86, "5. UloNumbers  6. UloMail", 15);
        VGA.draw_string(58, 98, "7. UloBrowser  8. TUI DOOM", 15);
        VGA.draw_string(58, 110, "9. App Store   0. Settings", 15);
        VGA.draw_rect(55, 126, 210, 1, 0); // separator
        VGA.draw_string(58, 134, "Restart UloOS Machine", 12);
    }
}

// Draws a premium Windows 11 Rounded-corner window with modern borders
fn draw_gui_window(title: &str, x: usize, y: usize, w: usize, h: usize) {
    VGA.draw_rect(x, y, w, h, 8); // Thin dark border outline
    VGA.draw_rect(x + 1, y + 1, w - 2, h - 2, 0); // Crisp dark background inside window (modern dark mode!)

    // Skip extreme corner pixels to simulate smooth rounded corners!
    VGA.draw_rect(x, y, 1, 1, 0); 
    VGA.draw_rect(x + w - 1, y, 1, 1, 0); 
    VGA.draw_rect(x, y + h - 1, 1, 1, 0); 
    VGA.draw_rect(x + w - 1, y + h - 1, 1, 1, 0); 

    // Sleek minimalist header bar (completely flat, dark gray index 8)
    VGA.draw_rect(x + 2, y + 2, w - 4, 12, 8); 
    VGA.draw_string(x + 6, y + 4, title, 15); // Crisp white title text

    // Minimalist Close button [X]
    VGA.draw_rect(x + w - 14, y + 4, 10, 8, 4); // Solid Red close box
    VGA.draw_string(x + w - 12, y + 4, "X", 15);
}

// Desktop Icon draw helper (Minimalist Windows 11 flat modern style)
fn draw_gui_icon(label: &str, x: usize, y: usize, _color: u8) {
    let theme_bg = if SYSTEM_SETTINGS.lock().active_theme == 0 { 0 } else { 8 };
    VGA.draw_rect(x - 2, y - 2, 54, 20, theme_bg); // clear area

    // Draw minimalist flat modern icons
    match label {
        "FILES" => {
            VGA.draw_rect(x, y, 10, 8, 9); // Blue folder base
            VGA.draw_rect(x + 2, y - 2, 4, 2, 9); // Folder tab
        }
        "DOS" => {
            VGA.draw_rect(x, y, 10, 8, 8); // Black screen base
            VGA.draw_rect(x + 2, y + 2, 6, 4, 0); // Inner CLI screen
        }
        "WRITE" => {
            VGA.draw_rect(x, y - 2, 8, 12, 15); // Document sheet
            VGA.draw_rect(x + 2, y + 1, 4, 1, 8); // text lines
            VGA.draw_rect(x + 2, y + 4, 4, 1, 8);
        }
        "DOOM" => {
            VGA.draw_rect(x, y, 10, 10, 12); // Red floppy disk
            VGA.draw_rect(x + 2, y + 4, 6, 6, 15); // Floppy label
        }
        "STORE" => {
            VGA.draw_rect(x, y, 10, 9, 14); // Yellow shopping bag
            VGA.draw_rect(x + 2, y - 2, 6, 2, 0); // Bag handle
        }
        _ => {}
    }

    // Draw minimalist text aligned side-by-side
    VGA.draw_string(x + 14, y, label, 15);
}



#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Red screen system panic display
    VGA.draw_rect(0, 0, 320, 200, 4);
    VGA.draw_string(10, 10, "[SYSTEM PANIC] UloOS has crashed!", 15);
    VGA.draw_string(10, 30, "Please restart the QEMU machine.", 15);
    VGA.swap_buffers();
    loop {}
}

pub struct SystemSettings {
    pub simulated_resolution: usize, // 0 = 320x200 (Default), 1 = 2560x1600 (4 Million Pixels!)
    pub mouse_sensitivity: usize,    // 0 = Normal, 1 = High, 2 = Ultra Smooth
    pub active_theme: usize,         // 0 = Windows 95 Teal, 1 = SAS OS Gray
    pub timezone_index: usize,       // 0 = UTC, 1 = EST, 2 = PST, 3 = CET, 4 = IST, 5 = CST, 6 = JST
}

impl SystemSettings {
    pub const fn new() -> Self {
        SystemSettings {
            simulated_resolution: 0,
            mouse_sensitivity: 0,
            active_theme: 0,
            timezone_index: 4,
        }
    }

    pub fn draw(&self) {
        // Light gray panel workspace
        VGA.draw_rect(12, 28, 296, 144, 7);

        VGA.draw_string(20, 34, "System Settings Dashboard", 1);
        VGA.draw_string(20, 44, "--------------------------", 8);

        // 1. Resolution Setting Option
        VGA.draw_string(20, 58, "1. Screen Size: ", 0);
        if self.simulated_resolution == 0 {
            VGA.draw_string(140, 58, "[ 320x200 Standard ]", 1);
        } else {
            VGA.draw_string(140, 58, "[ 2560x1600 (4M Px) ]", 2);
        }

        // 2. Mouse Precision / Smoothness Option
        VGA.draw_string(20, 78, "2. Mouse Speed: ", 0);
        if self.mouse_sensitivity == 0 {
            VGA.draw_string(140, 78, "[ Normal 1.0x ]", 1);
        } else if self.mouse_sensitivity == 1 {
            VGA.draw_string(140, 78, "[ Smooth 2.0x ]", 1);
        } else {
            VGA.draw_string(140, 78, "[ Butter 4.0x ]", 2);
        }

        // 3. UI Theme Option
        VGA.draw_string(20, 98, "3. UI Theme:   ", 0);
        if self.active_theme == 0 {
            VGA.draw_string(140, 98, "[ Windows 11 Dark ]", 1);
        } else {
            VGA.draw_string(140, 98, "[ Modern Slate ]", 1);
        }

        // 4. Time Zone Option
        VGA.draw_string(20, 118, "4. Time Zone:  ", 0);
        let tz_names = ["UTC +0:00", "EST -5:00", "PST -8:00", "CET +1:00", "IST +5:30", "CST +8:00", "JST +9:00"];
        VGA.draw_string(140, 118, tz_names[self.timezone_index], 1);

        // Footer instructions
        VGA.draw_string(20, 136, "Press [R] Resolution [M] Mouse", 8);
        VGA.draw_string(20, 148, "Press [T] UI Theme   [Z] TimeZone", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'r' | 'R' => {
                self.simulated_resolution = if self.simulated_resolution == 0 { 1 } else { 0 };
            }
            'm' | 'M' => {
                self.mouse_sensitivity = (self.mouse_sensitivity + 1) % 3;
            }
            't' | 'T' => {
                self.active_theme = if self.active_theme == 0 { 1 } else { 0 };
            }
            'z' | 'Z' => {
                self.timezone_index = (self.timezone_index + 1) % 7;
            }
            _ => {}
        }
    }
}

pub struct AppStore {
    pub installed: [bool; 3], // 0 = UloPaint, 1 = UloMusic, 2 = UloCalc
    pub selected: usize,      // 0, 1, 2
}

impl AppStore {
    pub const fn new() -> Self {
        AppStore {
            installed: [false, false, false],
            selected: 0,
        }
    }

    pub fn draw(&self) {
        // White store card workspace
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header
        VGA.draw_rect(12, 28, 296, 15, 1); // Blue XP Luna header
        VGA.draw_string(20, 31, "UloOS App Store - Market", 15);

        // App List
        let apps = [
            ("1. UloPaint", "Vector graphics editor tool", 10),
            ("2. UloMusic", "Chime sequencer & sound test", 11),
            ("3. UloCalc", "Simple accounting calculator", 12),
        ];

        for i in 0..3 {
            let row_y = 52 + i * 30;
            let is_selected = self.selected == i;
            let bg_col = if is_selected { 11 } else { 7 }; // light cyan selection
            let fg_col = if is_selected { 0 } else { 8 };

            VGA.draw_rect(16, row_y - 2, 288, 24, bg_col);
            VGA.draw_string(20, row_y + 2, apps[i].0, fg_col);
            VGA.draw_string(110, row_y + 2, apps[i].1, 8);

            // Install status badge
            if self.installed[i] {
                VGA.draw_string(230, row_y + 2, "[ INSTALLED ]", 2); // Green installed
            } else {
                VGA.draw_string(230, row_y + 2, "[ GET / FREE ]", 1); // Blue get
            }
        }

        // Instructions Footer
        VGA.draw_string(16, 148, "Use [W/S] to navigate. Press [G] to GET app.", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => { if self.selected < 2 { self.selected += 1; } }
            'g' | 'G' => {
                self.installed[self.selected] = !self.installed[self.selected]; // toggle installation
            }
            _ => {}
        }
    }
}
