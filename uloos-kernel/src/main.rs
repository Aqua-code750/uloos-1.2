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

use keyboard::{get_key, DecodedKey};
use apps::bash::BashShell;
use apps::explorer::{FileExplorer, WebBrowser, BrowserPage};
use apps::office::{UloCode, UloSlides, UloNumbers, UloMail, UloWeather, UloMusic, UloKeep, UloAi};
use apps::doom::TuiDoom;
use vga_driver::VGA;

use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveApp {
    Login,
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
    Weather,
    Music,
    Keep,
    Ai,
}

pub static IS_SETUP: Mutex<bool> = Mutex::new(false);
pub static USERNAME: Mutex<[u8; 15]> = Mutex::new(*b"               ");
pub static USERNAME_LEN: Mutex<usize> = Mutex::new(0);


pub static BASH: Mutex<BashShell> = Mutex::new(BashShell::new());
pub static EXPLORER: Mutex<FileExplorer> = Mutex::new(FileExplorer::new());
pub static TEXT_EDITOR: Mutex<UloCode> = Mutex::new(UloCode::new());
pub static SLIDES: Mutex<UloSlides> = Mutex::new(UloSlides::new());
pub static NUMBERS: Mutex<UloNumbers> = Mutex::new(UloNumbers::new());
pub static MAIL: Mutex<UloMail> = Mutex::new(UloMail::new());
pub static BROWSER: Mutex<WebBrowser> = Mutex::new(WebBrowser::new());
pub static DOOM_GAME: Mutex<TuiDoom> = Mutex::new(TuiDoom::new());
pub static SYSTEM_SETTINGS: Mutex<SystemSettings> = Mutex::new(SystemSettings::new());
pub static APP_STORE: Mutex<AppStore> = Mutex::new(AppStore::new());
pub static WEATHER: Mutex<UloWeather> = Mutex::new(UloWeather::new());
pub static MUSIC_SYNTH: Mutex<UloMusic> = Mutex::new(UloMusic::new());
pub static STICKY_KEEP: Mutex<UloKeep> = Mutex::new(UloKeep::new());
pub static CO_PILOT: Mutex<UloAi> = Mutex::new(UloAi::new());

// Global mouse cursor coordinate state
pub static CURSOR_X: Mutex<usize> = Mutex::new(160);
pub static CURSOR_Y: Mutex<usize> = Mutex::new(100);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Brief motherboard/interrupt stabilization delay to let virtual ports settle
    for _ in 0..100_000 {
        unsafe { core::arch::asm!("nop") }
    }

    // Seed UloCode default notes
    {
        let mut editor = TEXT_EDITOR.lock();
        let initial = "fn main() {\n    // Code inside UloOS!\n    let msg = \"Hello world\";\n    println!(\"{}\", msg);\n}\n";
        let mut rs_idx = 0;
        while rs_idx < initial.len() && rs_idx < 500 {
            editor.buffers[0][rs_idx] = initial.as_bytes()[rs_idx];
            rs_idx += 1;
        }
        editor.lens[0] = initial.len();
    }

    // 2. Switch CPU graphic mode to high-resolution mode 0x13 instantly
    vga_mode::set_vga_mode_320x200();

    let mut current_app = if *IS_SETUP.lock() { ActiveApp::Desktop } else { ActiveApp::Login };
    let mut start_menu_open = false;

    // 3. Render the desktop layout immediately
    draw_win95_graphics_desktop(current_app, start_menu_open);

    // 4. Initialize PS/2 mouse hardware driver AFTER screen setup is stable
    mouse::mouse_init();

    // 5. Play nostalgic startup chime sound
    sound::play_startup_sound();

    loop {
        // Poll PS/2 Mouse movements
        let sens = SYSTEM_SETTINGS.lock().mouse_sensitivity;
        while mouse::poll_mouse(sens) {}
        
        unsafe {
            let mx = mouse::MOUSE_X as usize;
            let my = mouse::MOUSE_Y as usize;
            
            *CURSOR_X.lock() = if mx >= 315 { 315 } else { mx };
            *CURSOR_Y.lock() = if my >= 195 { 195 } else { my };
        }

        // Draw desktop
        draw_win95_graphics_desktop(current_app, start_menu_open);

        // Draw active application viewport
        match current_app {
            ActiveApp::Login => {
                draw_login_screen();
            }
            ActiveApp::Desktop => {}
            ActiveApp::Bash => {
                draw_gui_window("MS-DOS Prompt", 10, 15, 300, 160);
                BASH.lock().draw();
            }
            ActiveApp::Explorer => {
                draw_gui_window("File Explorer", 10, 15, 300, 160);
                EXPLORER.lock().draw();
            }
            ActiveApp::Text => {
                draw_gui_window("UloCode Studio", 10, 15, 300, 160);
                TEXT_EDITOR.lock().draw();
            }
            ActiveApp::Slides => {
                draw_gui_window("UloSlides", 10, 15, 300, 160);
                SLIDES.lock().draw();
            }
            ActiveApp::Numbers => {
                draw_gui_window("UloNumbers", 10, 15, 300, 160);
                NUMBERS.lock().draw();
            }
            ActiveApp::Mail => {
                draw_gui_window("UloMail Inbox", 10, 15, 300, 160);
                MAIL.lock().draw();
            }
            ActiveApp::Browser => {
                draw_gui_window("Google Chrome Sandbox", 10, 15, 300, 160);
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
            ActiveApp::Weather => {
                draw_gui_window("UloWeather", 10, 15, 300, 160);
                WEATHER.lock().draw();
            }
            ActiveApp::Music => {
                draw_gui_window("UloMusic Synth", 10, 15, 300, 160);
                MUSIC_SYNTH.lock().draw();
            }
            ActiveApp::Keep => {
                draw_gui_window("UloKeep Notes", 10, 15, 300, 160);
                STICKY_KEEP.lock().draw();
            }
            ActiveApp::Ai => {
                draw_gui_window("UloOS AI Copilot", 10, 15, 300, 160);
                CO_PILOT.lock().draw();
            }
        }

        // Draw cursor
        let cx = *CURSOR_X.lock();
        let cy = *CURSOR_Y.lock();
        VGA.draw_cursor(cx, cy);

        // Swap backbuffer
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
                // Setup Login button click detection
                else if current_app == ActiveApp::Login {
                    if click_x >= 110 && click_x <= 210 && click_y >= 136 && click_y <= 152 {
                        let u_len = *USERNAME_LEN.lock();
                        if u_len > 0 {
                            *IS_SETUP.lock() = true;
                            current_app = ActiveApp::Desktop;
                            sound::play_startup_sound();
                        }
                    }
                }
                // Delegate clicks to active app viewport
                else if current_app == ActiveApp::Browser && click_y >= 28 {
                    BROWSER.lock().handle_click(click_x, click_y);
                }
                else if current_app == ActiveApp::Slides && click_y >= 28 {
                    SLIDES.lock().handle_click(click_x, click_y);
                }
                else if current_app == ActiveApp::Numbers && click_y >= 28 {
                    NUMBERS.lock().handle_click(click_x, click_y);
                }
                // UloCode LOAD and SAVE clicks
                else if current_app == ActiveApp::Text && click_y >= 28 && click_y <= 38 {
                    if click_x >= 125 && click_x <= 157 {
                        TEXT_EDITOR.lock().load_from_vfs();
                    } else if click_x >= 162 && click_x <= 194 {
                        TEXT_EDITOR.lock().save_to_vfs();
                    }
                }
                // Start button click
                else if click_y >= 185 && click_y <= 198 && click_x >= 63 && click_x <= 97 {
                    start_menu_open = !start_menu_open;
                }
                // Start Menu popups
                else if start_menu_open {
                    if click_x >= 50 && click_x <= 270 {
                        if click_y >= 36 && click_y <= 46 {
                            if click_x < 150 { current_app = ActiveApp::Bash; } 
                            else { current_app = ActiveApp::Explorer; }
                            start_menu_open = false;
                        }
                        else if click_y >= 48 && click_y <= 58 {
                            if click_x < 150 { current_app = ActiveApp::Text; } 
                            else { current_app = ActiveApp::Slides; }
                            start_menu_open = false;
                        }
                        else if click_y >= 60 && click_y <= 70 {
                            if click_x < 150 { current_app = ActiveApp::Numbers; } 
                            else { current_app = ActiveApp::Mail; }
                            start_menu_open = false;
                        }
                        else if click_y >= 72 && click_y <= 82 {
                            if click_x < 150 { current_app = ActiveApp::Browser; } 
                            else { current_app = ActiveApp::Doom; }
                            start_menu_open = false;
                        }
                        else if click_y >= 84 && click_y <= 94 {
                            if click_x < 150 { current_app = ActiveApp::Store; } 
                            else { current_app = ActiveApp::Settings; }
                            start_menu_open = false;
                        }
                        else if click_y >= 96 && click_y <= 106 {
                            if click_x < 150 { current_app = ActiveApp::Weather; }
                            else { current_app = ActiveApp::Music; }
                            start_menu_open = false;
                        }
                        else if click_y >= 108 && click_y <= 118 {
                             if click_x < 150 { current_app = ActiveApp::Keep; }
                             else { current_app = ActiveApp::Ai; }
                             start_menu_open = false;
                         }
                        else if click_y >= 130 && click_y <= 142 {
                            // Hardware Shutdown QEMU Virtual Machine!
                            start_menu_open = false;
                            
                            // Play shutdown tone
                            sound::play_tone(400);
                            for _ in 0..15_000 { unsafe { core::arch::asm!("nop") } }
                            sound::play_tone(200);
                            for _ in 0..15_000 { unsafe { core::arch::asm!("nop") } }
                            sound::stop_speaker();
                            
                            unsafe {
                                use core::arch::asm;
                                // QEMU ACPI shutdown (port 0x604 with 0x2000)
                                asm!("out dx, ax", in("dx") 0x604u16, in("ax") 0x2000u16);
                                // Older QEMU ACPI shutdown (port 0xb004 with 0x2000)
                                asm!("out dx, ax", in("dx") 0xb004u16, in("ax") 0x2000u16);
                                // QEMU isa-debug-exit shutdown (port 0xf4)
                                asm!("out dx, al", in("dx") 0xf4u16, in("al") 0u8);
                            }
                        }
                    }
                }
                // Desktop Shortcut clicks
                else if current_app == ActiveApp::Desktop {
                    if click_x >= 10 && click_x <= 120 {
                        if click_y >= 10 && click_y <= 24 { current_app = ActiveApp::Explorer; }
                        else if click_y >= 30 && click_y <= 44 { current_app = ActiveApp::Bash; }
                        else if click_y >= 50 && click_y <= 64 { current_app = ActiveApp::Text; }
                        else if click_y >= 70 && click_y <= 84 { current_app = ActiveApp::Doom; }
                        else if click_y >= 90 && click_y <= 104 { current_app = ActiveApp::Store; }
                        else if click_y >= 110 && click_y <= 124 { current_app = ActiveApp::Weather; }
                        else if click_y >= 130 && click_y <= 144 { current_app = ActiveApp::Music; }
                        else if click_y >= 150 && click_y <= 164 { current_app = ActiveApp::Keep; }
                        else if click_y >= 170 && click_y <= 184 { current_app = ActiveApp::Ai; }
                    }
                }
            }
        }

        // Read Keyboard events
        if let Some(key) = get_key() {
            match key {
                DecodedKey::Ascii(c) => {
                    match current_app {
                        ActiveApp::Login => {
                            let mut len = USERNAME_LEN.lock();
                            if *len < 12 && c >= ' ' && c <= '~' {
                                USERNAME.lock()[*len] = c as u8;
                                *len += 1;
                            }
                        }
                        ActiveApp::Bash => BASH.lock().add_char(c),
                        ActiveApp::Text => TEXT_EDITOR.lock().handle_key(c),
                        ActiveApp::Explorer => EXPLORER.lock().handle_input(c),
                        ActiveApp::Numbers => NUMBERS.lock().handle_input(c),
                        ActiveApp::Slides => SLIDES.lock().handle_input(c),
                        ActiveApp::Mail => MAIL.lock().handle_input(c),
                        ActiveApp::Doom => DOOM_GAME.lock().handle_input(c),
                        ActiveApp::Settings => SYSTEM_SETTINGS.lock().handle_input(c),
                        ActiveApp::Store => APP_STORE.lock().handle_input(c),
                        ActiveApp::Weather => WEATHER.lock().handle_input(c),
                        ActiveApp::Music => MUSIC_SYNTH.lock().handle_input(c),
                        ActiveApp::Keep => STICKY_KEEP.lock().handle_input(c),
                        ActiveApp::Browser => BROWSER.lock().handle_input(c),
                        ActiveApp::Ai => {
                            if c >= '1' && c <= '5' {
                                CO_PILOT.lock().handle_preset((c as usize) - ('0' as usize));
                            } else {
                                CO_PILOT.lock().handle_key(c);
                            }
                        }
                        _ => {}
                    }
                }
                DecodedKey::Backspace => {
                    match current_app {
                        ActiveApp::Login => {
                            let mut len = USERNAME_LEN.lock();
                            if *len > 0 {
                                *len -= 1;
                            }
                        }
                        ActiveApp::Bash => BASH.lock().handle_backspace(),
                        ActiveApp::Text => TEXT_EDITOR.lock().handle_backspace(),
                        ActiveApp::Browser => BROWSER.lock().handle_backspace(),
                        ActiveApp::Explorer => EXPLORER.lock().handle_backspace(),
                        ActiveApp::Keep => STICKY_KEEP.lock().handle_backspace(),
                        ActiveApp::Slides => SLIDES.lock().handle_backspace(),
                        ActiveApp::Numbers => NUMBERS.lock().handle_backspace(),
                        ActiveApp::Ai => CO_PILOT.lock().handle_backspace(),
                        _ => {}
                    }
                }
                DecodedKey::Enter => {
                    match current_app {
                        ActiveApp::Login => {
                            let u_len = *USERNAME_LEN.lock();
                            if u_len > 0 {
                                *IS_SETUP.lock() = true;
                                current_app = ActiveApp::Desktop;
                                sound::play_startup_sound();
                            }
                        }
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
                        ActiveApp::Explorer => EXPLORER.lock().handle_enter(),
                        ActiveApp::Text => TEXT_EDITOR.lock().handle_enter(),
                        ActiveApp::Slides => SLIDES.lock().next(),
                        ActiveApp::Mail => MAIL.lock().toggle(),
                        ActiveApp::Browser => BROWSER.lock().handle_enter(),
                        ActiveApp::Keep => STICKY_KEEP.lock().handle_enter(),
                        ActiveApp::Ai => CO_PILOT.lock().handle_enter(),
                        _ => {}
                    }
                }
                DecodedKey::Escape => {
                    match current_app {
                        ActiveApp::Explorer => EXPLORER.lock().handle_escape(),
                        ActiveApp::Browser => {
                            let mut b = BROWSER.lock();
                            if b.page == BrowserPage::Home {
                                current_app = ActiveApp::Desktop;
                            } else {
                                b.go_home();
                            }
                        }
                        _ => {
                            current_app = ActiveApp::Desktop;
                        }
                    }
                }
                DecodedKey::Tab => {
                    match current_app {
                        ActiveApp::Browser => BROWSER.lock().handle_input('\t'),
                        ActiveApp::Text => TEXT_EDITOR.lock().cycle_tab(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Delay to regulate speed
        for _ in 0..50 {
            unsafe { core::arch::asm!("nop") }
        }
    }
}


unsafe fn rtc_read(reg: u8) -> u8 {
    core::arch::asm!("out 0x70, al", in("al") reg, options(nomem, nostack, preserves_flags));
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
    
    let offsets = [0, -300, -480, 60, 330, 480, 540];
    let offset_minutes = offsets[SYSTEM_SETTINGS.lock().timezone_index];
    
    let mut total_minutes = (h_raw as i32) * 60 + (m_raw as i32) + offset_minutes;
    
    if total_minutes < 0 {
        total_minutes += 1440;
    }
    total_minutes = total_minutes % 1440;
    
    let h_local = (total_minutes / 60) as u8;
    let m_local = (total_minutes % 60) as u8;
    
    (h_local, m_local, s)
}

// Draws the modern Windows 11 style desktop
fn draw_win95_graphics_desktop(active: ActiveApp, start_open: bool) {
    // Dark background
    VGA.draw_rect(0, 0, 320, 185, 0); 

    // Carbon grid dots
    for gy in (4..180).step_by(10) {
        for gx in (4..316).step_by(10) {
            VGA.draw_rect(gx, gy, 1, 1, 8);
        }
    }

    // Desktop shortcuts
    draw_gui_icon("FILES", 10, 12, 10);
    draw_gui_icon("DOS", 10, 32, 14);
    draw_gui_icon("CODE", 10, 52, 11);
    draw_gui_icon("DOOM", 10, 72, 12);
    draw_gui_icon("STORE", 10, 92, 13);
    draw_gui_icon("WEATHER", 10, 112, 14);
    draw_gui_icon("MUSIC", 10, 132, 11);
    draw_gui_icon("KEEP", 10, 152, 12);
    draw_gui_icon("AI", 10, 172, 11);

    // Clock widget
    VGA.draw_rect(248, 12, 62, 14, 8);
    VGA.draw_rect(249, 13, 60, 12, 0);
    
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

    // Dock
    VGA.draw_rect(78, 186, 164, 12, 11);
    VGA.draw_rect(79, 187, 162, 10, 8);
    VGA.draw_rect(80, 188, 160, 8, 8);

    // Small avatar inside taskbar dock
    VGA.draw_rect(228, 188, 8, 8, 9); // Light Blue background
    VGA.draw_rect(231, 189, 2, 2, 15); // White silhouette head
    VGA.draw_rect(229, 192, 6, 3, 15); // White silhouette body

    // Start icon
    VGA.draw_rect(84, 188, 20, 8, 9);
    VGA.draw_string(86, 188, "ToT", 15);

    // Active app label
    let icon_offset_x = 115;
    match active {
        ActiveApp::Desktop => VGA.draw_string(icon_offset_x, 188, "[Desktop]", 11),
        ActiveApp::Bash => VGA.draw_string(icon_offset_x, 188, "[MS-DOS]", 11),
        ActiveApp::Explorer => VGA.draw_string(icon_offset_x, 188, "[Explorer]", 11),
        ActiveApp::Text => VGA.draw_string(icon_offset_x, 188, "[UloCode]", 11),
        ActiveApp::Slides => VGA.draw_string(icon_offset_x, 188, "[Slides]", 11),
        ActiveApp::Numbers => VGA.draw_string(icon_offset_x, 188, "[Numbers]", 11),
        ActiveApp::Mail => VGA.draw_string(icon_offset_x, 188, "[UloMail]", 11),
        ActiveApp::Browser => VGA.draw_string(icon_offset_x, 188, "[Chrome]", 11),
        ActiveApp::Doom => VGA.draw_string(icon_offset_x, 188, "[DOOM]", 11),
        ActiveApp::Settings => VGA.draw_string(icon_offset_x, 188, "[Settings]", 11),
        ActiveApp::Store => VGA.draw_string(icon_offset_x, 188, "[Store]", 11),
        ActiveApp::Weather => VGA.draw_string(icon_offset_x, 188, "[Weather]", 11),
        ActiveApp::Music => VGA.draw_string(icon_offset_x, 188, "[Music]", 11),
        ActiveApp::Keep => VGA.draw_string(icon_offset_x, 188, "[UloKeep]", 11),
        ActiveApp::Ai => VGA.draw_string(icon_offset_x, 188, "[Copilot]", 11),
        ActiveApp::Login => VGA.draw_string(icon_offset_x, 188, "[Lock]", 11),
    }

    // Start Menu
    if start_open {
        VGA.draw_rect(50, 18, 220, 166, 11);
        VGA.draw_rect(51, 19, 218, 164, 8);
        VGA.draw_rect(52, 20, 216, 162, 8);

        VGA.draw_rect(55, 23, 210, 10, 9);
        VGA.draw_string(58, 24, "UloOS 1.2 Pinned Apps", 15);

        VGA.draw_string(58, 38, "1. Bash Shell  2. Explorer", 15);
        VGA.draw_string(58, 50, "3. UloCode     4. UloSlides", 15);
        VGA.draw_string(58, 62, "5. UloNumbers  6. UloMail", 15);
        VGA.draw_string(58, 74, "7. Chrome      8. TUI DOOM", 15);
        VGA.draw_string(58, 86, "9. App Store   0. Settings", 15);
        VGA.draw_string(58, 98, "A. Weather     B. Music Synth", 15);
        VGA.draw_string(58, 110, "C. UloKeep     D. AI Copilot", 15);
        
        VGA.draw_rect(55, 122, 210, 1, 0);
        VGA.draw_string(58, 126, "Restart UloOS Machine", 12);

        // Profile Footer inside Start Menu
        VGA.draw_rect(55, 138, 210, 1, 0); // separator
        
        // Circular profile silhouette avatar in footer
        VGA.draw_rect(58, 144, 16, 16, 9); // Light Blue circle
        VGA.draw_rect(63, 146, 6, 6, 15);  // head
        VGA.draw_rect(60, 153, 12, 6, 15); // body

        // Username text
        if let Ok(name) = core::str::from_utf8(&USERNAME.lock()[..*USERNAME_LEN.lock()]) {
            VGA.draw_string(80, 148, name, 15);
        } else {
            VGA.draw_string(80, 148, "Guest", 15);
        }
        
        // Developer / Pro badge
        VGA.draw_rect(160, 146, 40, 12, 2); // Green badge
        VGA.draw_string(164, 148, "DEV PRO", 15);
    }
}

// Premium window with rounded corners
fn draw_gui_window(title: &str, x: usize, y: usize, w: usize, h: usize) {
    VGA.draw_rect(x, y, w, h, 8);
    VGA.draw_rect(x + 1, y + 1, w - 2, h - 2, 0);

    // Rounded corners
    VGA.draw_rect(x, y, 1, 1, 0); 
    VGA.draw_rect(x + w - 1, y, 1, 1, 0); 
    VGA.draw_rect(x, y + h - 1, 1, 1, 0); 
    VGA.draw_rect(x + w - 1, y + h - 1, 1, 1, 0); 

    // Header bar
    VGA.draw_rect(x + 2, y + 2, w - 4, 12, 8); 
    VGA.draw_string(x + 6, y + 4, title, 15);

    // Close button [X]
    VGA.draw_rect(x + w - 14, y + 4, 10, 8, 4);
    VGA.draw_string(x + w - 12, y + 4, "X", 15);
}

// Desktop icon helper
fn draw_gui_icon(label: &str, x: usize, y: usize, _color: u8) {
    let theme_bg = if SYSTEM_SETTINGS.lock().active_theme == 0 { 0 } else { 8 };
    VGA.draw_rect(x - 2, y - 2, 54, 20, theme_bg);

    match label {
        "FILES" => {
            VGA.draw_rect(x, y, 10, 8, 9);
            VGA.draw_rect(x + 2, y - 2, 4, 2, 9);
        }
        "DOS" => {
            VGA.draw_rect(x, y, 10, 8, 8);
            VGA.draw_rect(x + 2, y + 2, 6, 4, 0);
        }
        "CODE" => {
            VGA.draw_rect(x, y, 10, 10, 1); // Blue backing
            VGA.draw_rect(x + 2, y + 2, 6, 6, 0); // Inner box
            VGA.draw_string(x + 3, y + 2, ">", 9); // code bracket
        }
        "DOOM" => {
            VGA.draw_rect(x, y, 10, 10, 12);
            VGA.draw_rect(x + 2, y + 4, 6, 6, 15);
        }
        "STORE" => {
            VGA.draw_rect(x, y, 10, 9, 14);
            VGA.draw_rect(x + 2, y - 2, 6, 2, 0);
        }
        "WEATHER" => {
            VGA.draw_rect(x + 2, y, 6, 6, 14);
            VGA.draw_rect(x, y + 2, 10, 5, 15);
        }
        "MUSIC" => {
            VGA.draw_rect(x, y + 6, 4, 3, 13);
            VGA.draw_rect(x + 6, y + 6, 4, 3, 13);
            VGA.draw_rect(x + 3, y, 1, 7, 15);
            VGA.draw_rect(x + 9, y, 1, 7, 15);
            VGA.draw_rect(x + 3, y, 7, 2, 15);
        }
        "KEEP" => {
            VGA.draw_rect(x, y, 10, 10, 14);
            VGA.draw_rect(x + 2, y + 2, 6, 1, 0);
            VGA.draw_rect(x + 2, y + 5, 4, 1, 0);
        }
        "AI" => {
            VGA.draw_rect(x, y, 10, 10, 11); // Cyan backing head
            VGA.draw_rect(x + 2, y + 2, 6, 6, 0);  // Inner face (black)
            VGA.draw_rect(x + 3, y + 4, 1, 1, 15); // Left white eye
            VGA.draw_rect(x + 6, y + 4, 1, 1, 15); // Right white eye
        }
        _ => {}
    }

    VGA.draw_string(x + 14, y, label, 15);
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    VGA.draw_rect(0, 0, 320, 200, 4);
    VGA.draw_string(10, 10, "[SYSTEM PANIC] UloOS crashed!", 15);
    VGA.draw_string(10, 30, "Please restart the QEMU machine.", 15);
    VGA.swap_buffers();
    loop {}
}

pub struct SystemSettings {
    pub simulated_resolution: usize,
    pub mouse_sensitivity: usize,
    pub active_theme: usize,
    pub timezone_index: usize,
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
        VGA.draw_rect(12, 28, 296, 144, 7);

        VGA.draw_string(20, 34, "System Settings Dashboard", 1);
        VGA.draw_string(20, 44, "--------------------------", 8);

        VGA.draw_string(20, 58, "1. Screen Size: ", 0);
        if self.simulated_resolution == 0 {
            VGA.draw_string(140, 58, "[ 320x200 Standard ]", 1);
        } else {
            VGA.draw_string(140, 58, "[ 2560x1600 (4M Px) ]", 2);
        }

        VGA.draw_string(20, 78, "2. Mouse Speed: ", 0);
        if self.mouse_sensitivity == 0 {
            VGA.draw_string(140, 78, "[ Normal 1.0x ]", 1);
        } else if self.mouse_sensitivity == 1 {
            VGA.draw_string(140, 78, "[ Smooth 2.0x ]", 1);
        } else {
            VGA.draw_string(140, 78, "[ Butter 4.0x ]", 2);
        }

        VGA.draw_string(20, 98, "3. UI Theme:   ", 0);
        match self.active_theme {
            0 => VGA.draw_string(140, 98, "[ Sky Blue Fluent ]", 9),
            1 => VGA.draw_string(140, 98, "[ Cyber Purple Neon ]", 13),
            2 => VGA.draw_string(140, 98, "[ Emerald Mint Linux ]", 10),
            3 => VGA.draw_string(140, 98, "[ Coral Sunset Aura ]", 12),
            _ => VGA.draw_string(140, 98, "[ Classic Light Mode ]", 0),
        }

        VGA.draw_string(20, 118, "4. Time Zone:  ", 0);
        let tz_names = ["UTC +0:00", "EST -5:00", "PST -8:00", "CET +1:00", "IST +5:30", "CST +8:00", "JST +9:00"];
        VGA.draw_string(140, 118, tz_names[self.timezone_index], 1);

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
                self.active_theme = (self.active_theme + 1) % 5;
                vga_mode::set_dynamic_vga_palette(self.active_theme);
            }
            'z' | 'Z' => {
                self.timezone_index = (self.timezone_index + 1) % 7;
            }
            _ => {}
        }
    }
}

pub struct AppStore {
    pub installed: [bool; 4],
    pub selected: usize,
    pub published_app_name: [u8; 15],
    pub published_app_len: usize,
    pub has_published: bool,
}

impl AppStore {
    pub const fn new() -> Self {
        AppStore {
            installed: [false, false, false, false],
            selected: 0,
            published_app_name: [0; 15],
            published_app_len: 0,
            has_published: false,
        }
    }

    pub fn draw(&self) {
        VGA.draw_rect(12, 28, 296, 144, 15);

        // Header bar
        VGA.draw_rect(12, 28, 296, 15, 1);
        VGA.draw_string(20, 31, "UloOS App Store", 15);

        // Logged-in profile badge in store header
        VGA.draw_string(170, 31, "Dev: ", 11);
        if let Ok(name) = core::str::from_utf8(&USERNAME.lock()[..*USERNAME_LEN.lock()]) {
            VGA.draw_string(205, 31, name, 15);
        } else {
            VGA.draw_string(205, 31, "Guest", 15);
        }

        let apps = [
            ("1. UloPaint", "Vector graphics editor", 10),
            ("2. UloMusic", "Chime sequencer tool", 11),
            ("3. UloCalc", "Accounting calculator", 12),
        ];

        let max_idx = if self.has_published { 4 } else { 3 };

        for i in 0..max_idx {
            let row_y = 48 + i * 26;
            let is_selected = self.selected == i;
            let bg_col = if is_selected { 11 } else { 7 };
            let fg_col = if is_selected { 0 } else { 8 };

            VGA.draw_rect(16, row_y - 2, 288, 22, bg_col);

            if i < 3 {
                VGA.draw_string(20, row_y + 2, apps[i].0, fg_col);
                VGA.draw_string(110, row_y + 2, apps[i].1, 8);
            } else {
                // Draw community published app
                VGA.draw_string(20, row_y + 2, "4. VFS Published", fg_col);
                if let Ok(name) = core::str::from_utf8(&self.published_app_name[..self.published_app_len]) {
                    VGA.draw_string(110, row_y + 2, name, 8);
                }
            }

            if self.installed[i] {
                VGA.draw_string(230, row_y + 2, "[ INSTALLED ]", 2);
            } else {
                VGA.draw_string(230, row_y + 2, "[ GET ]", 1);
            }
        }

        VGA.draw_string(16, 154, "[W/S] Navigate  [G] Install  [P] Publish VFS File", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.selected > 0 { self.selected -= 1; } }
            's' | 'S' => {
                let max_idx = if self.has_published { 3 } else { 2 };
                if self.selected < max_idx { self.selected += 1; }
            }
            'g' | 'G' => {
                self.installed[self.selected] = !self.installed[self.selected];
            }
            'p' | 'P' => {
                // Publish from VFS selected index
                let explorer = EXPLORER.lock();
                if let Some(idx) = explorer.get_selected_entry_index() {
                    let entry = &explorer.entries[idx];
                    if !entry.is_dir {
                        let copy_len = if entry.name_len > 12 { 12 } else { entry.name_len };
                        self.published_app_name = [0; 15];
                        self.published_app_name[..copy_len].copy_from_slice(&entry.name[..copy_len]);
                        self.published_app_len = copy_len;
                        self.has_published = true;
                        self.installed[3] = false; // not yet installed by the public
                    }
                }
            }
            _ => {}
        }
    }
}

fn draw_login_screen() {
    // Elegant deep purple/blue gradient or sleek carbon background
    VGA.draw_rect(0, 0, 320, 200, 0); // Black backing
    
    // Abstract fluid shapes
    for r in 0..60 {
        VGA.draw_rect(0, 200 - r, 320, 1, 8); // Gray gradient
    }

    // Windows 11 Fluent Card
    VGA.draw_rect(50, 30, 220, 140, 7); // Light gray card background
    VGA.draw_rect(51, 31, 218, 138, 8); // Dark gray inner border
    VGA.draw_rect(52, 32, 216, 136, 0); // Deep charcoal body

    // Circular Profile Avatar (fluent design silhouette)
    VGA.draw_rect(145, 42, 30, 30, 9); // Light Blue avatar frame
    VGA.draw_rect(155, 47, 10, 10, 15); // Silhouette head (white)
    VGA.draw_rect(150, 59, 20, 10, 15); // Silhouette body (white)

    VGA.draw_string(92, 80, "UloOS Account Setup", 15); // White header
    VGA.draw_string(72, 92, "-------------------------", 8);

    // Text Input Container
    VGA.draw_rect(80, 104, 160, 16, 7); // outer white border
    VGA.draw_rect(81, 105, 158, 14, 0); // black inner input box
    
    VGA.draw_string(85, 108, "Username:", 11); // cyan prompt
    if let Ok(name) = core::str::from_utf8(&USERNAME.lock()[..*USERNAME_LEN.lock()]) {
        VGA.draw_string(160, 108, name, 15); // custom username
    }

    // Blinking cursor
    let cur_x = 160 + (*USERNAME_LEN.lock() * 6);
    if cur_x < 235 {
        VGA.draw_rect(cur_x, 116, 5, 2, 11);
    }

    // Button: [ LOG IN ]
    VGA.draw_rect(110, 136, 100, 16, 1); // Blue button
    VGA.draw_rect(111, 137, 98, 14, 9); // cyan inner highlight
    VGA.draw_string(144, 140, "LOG IN", 15);

    VGA.draw_string(74, 158, "Press Enter or click LOG IN", 8);
}
