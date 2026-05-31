use crate::vga_driver::VGA;

pub struct TuiDoom {
    // Player coordinates & stats
    pub px: f32,
    pub py: f32,
    pub p_dir_x: f32,
    pub p_dir_y: f32,
    pub health: i32,     // Starts at 6 (3 full hearts)
    pub max_health: i32, // Starts at 6 (3 heart containers)
    pub score: i32,
    pub coins: i32,
    pub bombs: i32,
    pub keys: i32,
    pub current_room: i32,
    
    // Weapon stats (Isaac items!)
    pub fire_delay: usize,
    pub fire_cooldown: usize, // Firing speed
    pub damage: i32,
    pub bullet_speed: f32,
    pub active_item: &'static str,

    // Active Room State
    pub room_cleared: bool,
    pub room_type: u8, // 0: Start, 1: Combat, 2: Treasure, 3: Boss
    pub door_n: bool,  // Open door flags
    pub door_s: bool,
    pub door_e: bool,
    pub door_w: bool,

    // Isaac Item pedestal (x, y, item_id, active)
    pub pedestal_x: f32,
    pub pedestal_y: f32,
    pub pedestal_item: u8, // 0: BFG-9000, 1: Sad Onion, 2: Spoon Bender
    pub pedestal_active: bool,

    // Tears/Bullets (x, y, dx, dy, active)
    pub tears: [(f32, f32, f32, f32, bool); 8],

    // Enemies (x, y, hp, max_hp, type, active)
    pub enemies_x: [f32; 3],
    pub enemies_y: [f32; 3],
    pub enemies_hp: [i32; 3],
    pub enemies_max_hp: [i32; 3],
    pub enemies_type: [u8; 3], // 0: Fly Cacodemon, 1: Tear Imp, 2: Cyber-Monstro (Boss)
    pub enemies_active: [bool; 3],

    // Pickup Chests/Consumables (x, y, type, active) - 0: Heart, 1: Coin, 2: Bomb, 3: Key
    pub pickups_x: [f32; 3],
    pub pickups_y: [f32; 3],
    pub pickups_type: [u8; 3],
    pub pickups_active: [bool; 3],

    pub map: [[u8; 9]; 9],
}

fn my_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x;
    for _ in 0..8 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
}

impl TuiDoom {
    pub const fn new() -> Self {
        TuiDoom {
            px: 4.5,
            py: 4.5,
            p_dir_x: 0.0,
            p_dir_y: -1.0,
            health: 6,      // 3 Hearts (1 heart = 2 HP)
            max_health: 6,
            score: 0,
            coins: 0,
            bombs: 1,
            keys: 0,
            current_room: 1,
            
            fire_delay: 0,
            fire_cooldown: 12, // Lower is faster
            damage: 10,
            bullet_speed: 0.35,
            active_item: "Standard Pistol",

            room_cleared: true,
            room_type: 0, // Start room
            door_n: true,
            door_s: true,
            door_e: true,
            door_w: true,

            pedestal_x: 4.5,
            pedestal_y: 4.5,
            pedestal_item: 0,
            pedestal_active: false,

            tears: [(0.0, 0.0, 0.0, 0.0, false); 8],

            enemies_x: [0.0; 3],
            enemies_y: [0.0; 3],
            enemies_hp: [0; 3],
            enemies_max_hp: [0; 3],
            enemies_type: [0; 3],
            enemies_active: [false; 3],

            pickups_x: [0.0; 3],
            pickups_y: [0.0; 3],
            pickups_type: [0; 3],
            pickups_active: [false; 3],

            map: [
                [1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1],
            ],
        }
    }

    // Generate a fresh random dungeon room layout!
    pub fn generate_new_room(&mut self, entrance_door: char) {
        self.current_room += 1;
        self.tears = [(0.0, 0.0, 0.0, 0.0, false); 8];
        self.pickups_active = [false; 3];
        self.pedestal_active = false;

        // Position player at the opposite side of entered door
        match entrance_door {
            'N' => { self.px = 4.5; self.py = 7.2; }
            'S' => { self.px = 4.5; self.py = 1.8; }
            'E' => { self.px = 1.8; self.py = 4.5; }
            _   => { self.px = 7.2; self.py = 4.5; }
        }

        // Determine room type based on current progress
        if self.current_room % 4 == 0 {
            // Boss Room!
            self.room_type = 3;
            self.room_cleared = false;
            
            // Spawn Cyber-Monstro!
            self.enemies_x[0] = 4.5;
            self.enemies_y[0] = 3.0;
            self.enemies_hp[0] = 200;
            self.enemies_max_hp[0] = 200;
            self.enemies_type[0] = 2; // Boss
            self.enemies_active[0] = true;

            self.enemies_active[1] = false;
            self.enemies_active[2] = false;

            // Lock doors until boss is slain
            self.door_n = false;
            self.door_s = false;
            self.door_e = false;
            self.door_w = false;
        } 
        else if self.current_room % 3 == 0 {
            // Treasure Room!
            self.room_type = 2;
            self.room_cleared = true;
            self.enemies_active = [false; 3];

            // Spawn random item pedestal
            self.pedestal_active = true;
            self.pedestal_item = (self.current_room % 3) as u8; // Cycle items
            
            // Open all doors
            self.door_n = true;
            self.door_s = true;
            self.door_e = true;
            self.door_w = true;
        } 
        else {
            // Standard Combat Room!
            self.room_type = 1;
            self.room_cleared = false;

            // Spawn standard Isaac/Doom crossover monsters
            self.enemies_x = [2.0, 7.0, 4.5];
            self.enemies_y = [2.5, 2.5, 6.5];
            self.enemies_hp = [25, 25, 35];
            self.enemies_max_hp = [25, 25, 35];
            self.enemies_type = [0, 0, 1]; // Fly Cacodemon, Fly Cacodemon, Tear Imp
            self.enemies_active = [true, true, true];

            // Lock doors during combat
            self.door_n = false;
            self.door_s = false;
            self.door_e = false;
            self.door_w = false;
        }
    }

    // Helper to draw cute pixel hearts
    fn draw_heart(&self, x: usize, y: usize, half: bool) {
        if half {
            // Half Red Heart
            VGA.draw_rect(x, y + 1, 2, 3, 12);
            VGA.draw_rect(x + 1, y, 1, 1, 12);
            VGA.draw_rect(x + 1, y + 4, 1, 1, 12);
            // Gray backing half
            VGA.draw_rect(x + 2, y + 1, 2, 3, 8);
            VGA.draw_rect(x + 2, y, 1, 1, 8);
            VGA.draw_rect(x + 2, y + 4, 1, 1, 8);
        } else {
            // Full Red Heart
            VGA.draw_rect(x + 1, y, 2, 1, 12);
            VGA.draw_rect(x + 4, y, 2, 1, 12);
            VGA.draw_rect(x, y + 1, 7, 3, 12);
            VGA.draw_rect(x + 1, y + 4, 5, 1, 12);
            VGA.draw_rect(x + 2, y + 5, 3, 1, 12);
            VGA.draw_rect(x + 3, y + 6, 1, 1, 12);
        }
    }

    pub fn draw(&mut self) {
        // Clear viewport with a nice brown/dirt floor background (Isaac style)
        VGA.draw_rect(12, 28, 296, 144, 6); // Brown backing

        // Room drawing coordinate config
        let start_x = 22;
        let start_y = 38;
        let cell_size = 13;

        // 1. Draw solid walls & custom Isaac styled corner rocks
        for r in 0..9 {
            for c in 0..9 {
                let cx = start_x + c * cell_size;
                let cy = start_y + r * cell_size;

                if self.map[r][c] == 1 {
                    // Check if middle of border for doors
                    let is_door_cell = (r == 0 && c == 4) || (r == 8 && c == 4) || (r == 4 && c == 0) || (r == 4 && c == 8);
                    
                    if is_door_cell {
                        // Open empty doorway or locked black boundary
                        let is_open = (r == 0 && self.door_n) || (r == 8 && self.door_s) || (c == 8 && self.door_e) || (c == 0 && self.door_w);
                        if is_open {
                            VGA.draw_rect(cx, cy, cell_size - 1, cell_size - 1, 0); // Open black door passage
                            VGA.draw_string(cx + 3, cy + 2, "o", 10); // Green exit marker
                        } else {
                            VGA.draw_rect(cx, cy, cell_size - 1, cell_size - 1, 0); // Locked door
                            VGA.draw_rect(cx + 2, cy + 2, cell_size - 5, cell_size - 5, 12); // Red lock
                        }
                    } else {
                        // Solid stone/brick border
                        VGA.draw_rect(cx, cy, cell_size - 1, cell_size - 1, 8); // Charcoal stone
                        VGA.draw_rect(cx + 2, cy + 2, cell_size - 5, cell_size - 5, 7); // Light gray highlight
                    }
                } else {
                    // Dirt/Floor tile details
                    VGA.draw_rect(cx, cy, cell_size - 1, cell_size - 1, 6); // Brown floor
                    if (r + c) % 3 == 0 {
                        VGA.draw_rect(cx + 3, cy + 4, 1, 1, 14); // Little pebbles
                    }
                }
            }
        }

        // 2. Draw Pedestal in Treasure Room
        if self.pedestal_active {
            let px = (start_x as f32 + self.pedestal_x * cell_size as f32) as usize;
            let py = (start_y as f32 + self.pedestal_y * cell_size as f32) as usize;

            // Gray stone pillar pedestal
            VGA.draw_rect(px - 4, py + 2, 9, 6, 8);
            VGA.draw_rect(px - 6, py + 7, 13, 2, 7);

            // Floating item detail above pedestal
            match self.pedestal_item {
                0 => {
                    // BFG-9000 (Giant plasma cannon)
                    VGA.draw_rect(px - 5, py - 4, 11, 4, 2); // Green gun body
                    VGA.draw_rect(px - 1, py - 3, 3, 2, 11); // Blue plasma chamber
                }
                1 => {
                    // Sad Onion (Tear drop face)
                    VGA.draw_rect(px - 4, py - 4, 9, 7, 15); // White onion body
                    VGA.draw_rect(px - 2, py - 2, 1, 2, 9);  // Tear drop details
                    VGA.draw_rect(px + 1, py - 2, 1, 2, 9);
                }
                _ => {
                    // Spoon Bender (Bent silver spoon)
                    VGA.draw_rect(px - 2, py - 5, 5, 2, 7);
                    VGA.draw_rect(px, py - 3, 1, 5, 7);
                }
            }
        }

        // 3. Update & Draw Pickups
        for i in 0..3 {
            if self.pickups_active[i] {
                let pick_x = (start_x as f32 + self.pickups_x[i] * cell_size as f32) as usize;
                let pick_y = (start_y as f32 + self.pickups_y[i] * cell_size as f32) as usize;

                match self.pickups_type[i] {
                    0 => {
                        // Heart pickup
                        self.draw_heart(pick_x - 3, pick_y - 3, false);
                    }
                    1 => {
                        // Coin pickup
                        VGA.draw_rect(pick_x - 3, pick_y - 3, 7, 7, 14); // Shiny gold circle
                        VGA.draw_rect(pick_x - 1, pick_y - 1, 3, 3, 15);
                    }
                    2 => {
                        // Bomb pickup
                        VGA.draw_rect(pick_x - 3, pick_y - 2, 7, 6, 0); // Black bomb
                        VGA.draw_rect(pick_x - 1, pick_y - 4, 2, 2, 14); // Sparking wick
                    }
                    _ => {
                        // Key pickup
                        VGA.draw_rect(pick_x - 1, pick_y - 4, 3, 3, 7); // Ring
                        VGA.draw_rect(pick_x, pick_y - 1, 1, 5, 7);   // shaft
                        VGA.draw_rect(pick_x + 1, pick_y + 2, 2, 1, 7); // teeth
                    }
                }
            }
        }

        // 4. Update & Draw Tears/Bullets
        for t in 0..8 {
            if self.tears[t].4 {
                // Move tears
                self.tears[t].0 += self.tears[t].2;
                self.tears[t].1 += self.tears[t].3;

                let tx = self.tears[t].0;
                let ty = self.tears[t].1;

                // Wall boundary check
                if self.map[ty as usize][tx as usize] == 1 {
                    self.tears[t].4 = false;
                    continue;
                }

                // Enemy collision check
                for e in 0..3 {
                    if self.enemies_active[e] && self.enemies_hp[e] > 0 {
                        let dx = tx - self.enemies_x[e];
                        let dy = ty - self.enemies_y[e];
                        
                        if (dx * dx + dy * dy) < 0.22 {
                            self.enemies_hp[e] -= self.damage;
                            self.tears[t].4 = false;

                            // Splash sound
                            unsafe {
                                crate::sound::play_tone(400);
                                for _ in 0..3_000 { core::arch::asm!("nop") }
                                crate::sound::stop_speaker();
                            }

                            if self.enemies_hp[e] <= 0 {
                                self.enemies_active[e] = false;
                                self.score += 100 * (self.enemies_type[e] as i32 + 1);

                                // Check if room is cleared!
                                let mut cleared = true;
                                for check in 0..3 {
                                    if self.enemies_active[check] && self.enemies_hp[check] > 0 {
                                        cleared = false;
                                    }
                                }

                                if cleared {
                                    self.room_cleared = true;
                                    // Unlock all doors
                                    self.door_n = true;
                                    self.door_s = true;
                                    self.door_e = true;
                                    self.door_w = true;

                                    // Spawn random drop (heart/coin/bomb/key) in center!
                                    self.pickups_x[0] = 4.5;
                                    self.pickups_y[0] = 4.5;
                                    self.pickups_type[0] = (self.current_room % 4) as u8;
                                    self.pickups_active[0] = true;

                                    // Room clear chime melody!
                                    unsafe {
                                        crate::sound::play_tone(523);
                                        for _ in 0..8_000 { core::arch::asm!("nop") }
                                        crate::sound::play_tone(659);
                                        for _ in 0..8_000 { core::arch::asm!("nop") }
                                        crate::sound::play_tone(784);
                                        for _ in 0..12_000 { core::arch::asm!("nop") }
                                        crate::sound::stop_speaker();
                                    }
                                }
                            }
                            break;
                        }
                    }
                }

                // Render tears (glowing neon blue / plasma bullets!)
                let scr_tx = (start_x as f32 + tx * cell_size as f32) as usize;
                let scr_ty = (start_y as f32 + ty * cell_size as f32) as usize;
                VGA.draw_rect(scr_tx - 2, scr_ty - 2, 4, 4, 9); // Light blue
                VGA.draw_rect(scr_tx - 1, scr_ty - 1, 2, 2, 15); // White inner core
            }
        }

        // 5. Update & Draw Enemies
        for e in 0..3 {
            if self.enemies_active[e] && self.enemies_hp[e] > 0 {
                // Isaac AI: Float and wander towards player
                let dx = self.px - self.enemies_x[e];
                let dy = self.py - self.enemies_y[e];
                let dist = my_sqrt(dx * dx + dy * dy);

                if dist > 0.28 {
                    let step = if self.enemies_type[e] == 2 { 0.015 } else { 0.024 }; // boss moves slightly slower
                    self.enemies_x[e] += (dx / dist) * step;
                    self.enemies_y[e] += (dy / dist) * step;
                } else {
                    // Deal damage to Doomguy!
                    if self.health > 0 {
                        self.health -= 1;
                        // Doomguy pain sound
                        unsafe {
                            crate::sound::play_tone(90);
                            for _ in 0..2_500 { core::arch::asm!("nop") }
                            crate::sound::stop_speaker();
                        }
                    }
                }

                let ex = (start_x as f32 + self.enemies_x[e] * cell_size as f32) as usize;
                let ey = (start_y as f32 + self.enemies_y[e] * cell_size as f32) as usize;

                match self.enemies_type[e] {
                    0 => {
                        // Fly Cacodemon: Small round red fly
                        VGA.draw_rect(ex - 4, ey - 4, 9, 9, 12); // Red body
                        VGA.draw_rect(ex - 6, ey - 2, 2, 4, 15); // Left white wing
                        VGA.draw_rect(ex + 4, ey - 2, 2, 4, 15); // Right wing
                        VGA.draw_rect(ex - 1, ey - 2, 3, 3, 14); // Yellow mono eye
                    }
                    1 => {
                        // Tear Imp: Brown crying imp
                        VGA.draw_rect(ex - 5, ey - 5, 11, 10, 6); // Brown face
                        VGA.draw_rect(ex - 2, ey - 2, 1, 3, 9);   // Blue tears stream
                        VGA.draw_rect(ex + 2, ey - 2, 1, 3, 9);
                    }
                    _ => {
                        // Cyber-Monstro (Giant Cacodemon Boss!)
                        VGA.draw_rect(ex - 12, ey - 12, 25, 25, 12); // Huge red body
                        VGA.draw_rect(ex - 3, ey - 3, 7, 7, 14);     // Huge yellow central eye
                        VGA.draw_rect(ex - 8, ey + 4, 17, 4, 0);     // Wide open maw
                        VGA.draw_rect(ex - 6, ey - 15, 4, 4, 7);     // Cybernetic horns!
                        VGA.draw_rect(ex + 2, ey - 15, 4, 4, 7);

                        // Draw Boss Health Bar at top of room!
                        VGA.draw_rect(start_x + 10, start_y + 4, 95, 4, 0); // Background
                        let hp_w = (self.enemies_hp[e] as f32 / self.enemies_max_hp[e] as f32 * 93.0) as usize;
                        VGA.draw_rect(start_x + 11, start_y + 5, hp_w, 2, 12); // Red Boss HP fill
                    }
                }
            }
        }

        // 6. Draw Player (Doomguy Isaac version)
        let player_scr_x = (start_x as f32 + self.px * cell_size as f32) as usize;
        let player_scr_y = (start_y as f32 + self.py * cell_size as f32) as usize;

        // Big round head (Isaac styled Doomguy helmet!)
        VGA.draw_rect(player_scr_x - 5, player_scr_y - 6, 11, 10, 2); // Green helmet
        VGA.draw_rect(player_scr_x - 3, player_scr_y - 4, 7, 5, 11);  // Blue visor
        // Tiny cute body
        VGA.draw_rect(player_scr_x - 3, player_scr_y + 4, 7, 5, 8);  // Silver armor chest

        // Draw player pointing indicator
        let aim_x = (player_scr_x as f32 + self.p_dir_x * 8.0) as usize;
        let aim_y = (player_scr_y as f32 + self.p_dir_y * 8.0) as usize;
        VGA.draw_rect(aim_x, aim_y, 1, 1, 10); // Aim dot

        // 7. Draw the complete Stats Dashboard Panel (Right Side)
        let hud_x = 152;
        VGA.draw_rect(hud_x, 38, 148, 122, 0); // Black backing
        VGA.draw_rect(hud_x + 2, 40, 144, 118, 8); // Charcoal inner

        VGA.draw_string(hud_x + 8, 46, "ISLE OF DOOM", 12); // Crimson red title
        VGA.draw_string(hud_x + 8, 54, "==============", 0);

        // Render full hearts list (1 full heart = 2 HP)
        VGA.draw_string(hud_x + 8, 66, "HP:", 15);
        for h in 0..(self.max_health / 2) as usize {
            let hx = hud_x + 36 + h * 9;
            let hy = 66;
            if self.health >= (h as i32 + 1) * 2 {
                self.draw_heart(hx, hy, false); // Full Red Heart
            } else if self.health == (h as i32 * 2) + 1 {
                self.draw_heart(hx, hy, true); // Half Heart
            } else {
                // Empty Heart container outline
                VGA.draw_rect(hx, hy + 1, 6, 4, 0);
                VGA.draw_rect(hx + 1, hy, 4, 1, 0);
            }
        }

        // Stats Counters
        VGA.draw_string(hud_x + 8, 78, "COINS:", 14); // Yellow
        let mut coin_str = [b'0'; 2];
        coin_str[0] = b'0' + (self.coins / 10) as u8;
        coin_str[1] = b'0' + (self.coins % 10) as u8;
        VGA.draw_string(hud_x + 60, 78, core::str::from_utf8(&coin_str).unwrap(), 14);

        VGA.draw_string(hud_x + 8, 90, "BOMBS:", 7); // Light gray
        let mut bomb_str = [b'0'; 2];
        bomb_str[0] = b'0' + (self.bombs / 10) as u8;
        bomb_str[1] = b'0' + (self.bombs % 10) as u8;
        VGA.draw_string(hud_x + 60, 90, core::str::from_utf8(&bomb_str).unwrap(), 7);

        VGA.draw_string(hud_x + 8, 102, "KEYS :", 11); // Blue/cyan keys
        let mut key_str = [b'0'; 2];
        key_str[0] = b'0' + (self.keys / 10) as u8;
        key_str[1] = b'0' + (self.keys % 10) as u8;
        VGA.draw_string(hud_x + 60, 102, core::str::from_utf8(&key_str).unwrap(), 11);

        // Room/Progress stats
        VGA.draw_string(hud_x + 8, 116, "ROOM :", 15);
        let mut room_str = [b'0'; 2];
        room_str[0] = b'0' + (self.current_room / 10) as u8;
        room_str[1] = b'0' + (self.current_room % 10) as u8;
        VGA.draw_string(hud_x + 60, 116, core::str::from_utf8(&room_str).unwrap(), 15);

        // Active item
        VGA.draw_string(hud_x + 8, 128, "ITEM :", 10); // Green
        VGA.draw_string(hud_x + 8, 138, self.active_item, 10);

        // Game instructions
        VGA.draw_string(hud_x + 8, 148, "[WASD] Move [R] Restart", 9);

        // Game Over screen check
        if self.health <= 0 {
            VGA.draw_rect(start_x + 6, start_y + 35, 106, 46, 12); // Crimson banner
            VGA.draw_rect(start_x + 8, start_y + 37, 102, 42, 0); // Black inner
            VGA.draw_string(start_x + 20, start_y + 44, "GAME OVER", 12);
            VGA.draw_string(start_x + 14, start_y + 56, "Doomguy Fainted", 15);
            VGA.draw_string(start_x + 16, start_y + 66, "Press [R] Retry", 15);
        }
    }

    pub fn handle_input(&mut self, key: char) {
        if self.health <= 0 {
            if key == 'r' || key == 'R' {
                *self = TuiDoom::new();
            }
            return;
        }

        let move_speed = 0.15;

        // Firing delay cooldown updates
        if self.fire_delay > 0 {
            self.fire_delay -= 1;
        }

        match key {
            'w' | 'W' => {
                let ny = self.py - move_speed;
                self.p_dir_x = 0.0;
                self.p_dir_y = -1.0;

                // Check for North door transition!
                if self.room_cleared && self.py < 1.4 && self.px >= 3.8 && self.px <= 5.2 {
                    self.generate_new_room('N');
                    return;
                }

                if self.map[ny as usize][self.px as usize] == 0 {
                    self.py = ny;
                }
                self.pickup_check();
            }
            's' | 'S' => {
                let ny = self.py + move_speed;
                self.p_dir_x = 0.0;
                self.p_dir_y = 1.0;

                // Check for South door transition!
                if self.room_cleared && self.py > 7.6 && self.px >= 3.8 && self.px <= 5.2 {
                    self.generate_new_room('S');
                    return;
                }

                if self.map[ny as usize][self.px as usize] == 0 {
                    self.py = ny;
                }
                self.pickup_check();
            }
            'a' | 'A' => {
                let nx = self.px - move_speed;
                self.p_dir_x = -1.0;
                self.p_dir_y = 0.0;

                // Check for West door transition!
                if self.room_cleared && self.px < 1.4 && self.py >= 3.8 && self.py <= 5.2 {
                    self.generate_new_room('W');
                    return;
                }

                if self.map[self.py as usize][nx as usize] == 0 {
                    self.px = nx;
                }
                self.pickup_check();
            }
            'd' | 'D' => {
                let nx = self.px + move_speed;
                self.p_dir_x = 1.0;
                self.p_dir_y = 0.0;

                // Check for East door transition!
                if self.room_cleared && self.px > 7.6 && self.py >= 3.8 && self.py <= 5.2 {
                    self.generate_new_room('E');
                    return;
                }

                if self.map[self.py as usize][nx as usize] == 0 {
                    self.px = nx;
                }
                self.pickup_check();
            }
            ' ' => {
                // Shoot Bullets/Tears in facing direction!
                if self.fire_delay == 0 {
                    self.fire_delay = self.fire_cooldown;

                    // Find inactive tear slot
                    for t in 0..8 {
                        if !self.tears[t].4 {
                            self.tears[t] = (
                                self.px,
                                self.py,
                                self.p_dir_x * self.bullet_speed,
                                self.p_dir_y * self.bullet_speed,
                                true,
                            );

                            // Shooting squishy splash sound
                            unsafe {
                                crate::sound::play_tone(600);
                                for _ in 0..2_500 { core::arch::asm!("nop") }
                                crate::sound::play_tone(300);
                                for _ in 0..3_500 { core::arch::asm!("nop") }
                                crate::sound::stop_speaker();
                            }
                            break;
                        }
                    }
                }
            }
            'r' | 'R' => {
                *self = TuiDoom::new();
            }
            _ => {}
        }
    }

    // Handles item collections & pickups
    fn pickup_check(&mut self) {
        // 1. Check pedestal item collection
        if self.pedestal_active {
            let dx = self.px - self.pedestal_x;
            let dy = self.py - self.pedestal_y;
            if (dx * dx + dy * dy) < 0.25 {
                self.pedestal_active = false;
                
                // Pedestal pickup melody!
                unsafe {
                    crate::sound::play_tone(523);
                    for _ in 0..5_000 { core::arch::asm!("nop") }
                    crate::sound::play_tone(659);
                    for _ in 0..5_000 { core::arch::asm!("nop") }
                    crate::sound::play_tone(784);
                    for _ in 0..5_000 { core::arch::asm!("nop") }
                    crate::sound::play_tone(1046);
                    for _ in 0..10_000 { core::arch::asm!("nop") }
                    crate::sound::stop_speaker();
                }

                // Apply premium Isaac passive upgrades!
                match self.pedestal_item {
                    0 => {
                        self.active_item = "BFG-9000!";
                        self.damage = 35;
                        self.fire_cooldown = 18; // Heavy slow fire
                        self.bullet_speed = 0.45;
                    }
                    1 => {
                        self.active_item = "Sad Onion";
                        self.fire_cooldown = 6;  // CRAZY tears firing speed!
                    }
                    _ => {
                        self.active_item = "Spoon Bender";
                        self.damage = 18;
                        self.bullet_speed = 0.50; // Super high speed tears
                    }
                }
            }
        }

        // 2. Check standard drops & pickups
        for i in 0..3 {
            if self.pickups_active[i] {
                let dx = self.px - self.pickups_x[i];
                let dy = self.py - self.pickups_y[i];
                if (dx * dx + dy * dy) < 0.22 {
                    self.pickups_active[i] = false;

                    // Cute collect ding
                    unsafe {
                        crate::sound::play_tone(900);
                        for _ in 0..4_000 { core::arch::asm!("nop") }
                        crate::sound::stop_speaker();
                    }

                    match self.pickups_type[i] {
                        0 => {
                            // Collect heart (Restore 2 HP / 1 full heart container)
                            self.health = if self.health + 2 > self.max_health { self.max_health } else { self.health + 2 };
                        }
                        1 => { self.coins += 1; }
                        2 => { self.bombs += 1; }
                        _ => { self.keys += 1; }
                    }
                }
            }
        }
    }
}
