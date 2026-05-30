use crate::vga_driver::VGA;

pub struct TuiDoom {
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32, // in radians
    pub map: [[u8; 8]; 8],
    pub score: i32,
    pub ammo: i32,
    pub health: i32,
    pub gun_recoil: usize, // 0: standby, 1..3: muzzle flash recoil frames
    pub enemy_x: f32,
    pub enemy_y: f32,
    pub enemy_health: i32,
    pub enemy_dead: bool,
}

fn my_cos(x: f32) -> f32 {
    let pi = 3.14159265;
    let mut val = x;
    while val > pi {
        val -= 2.0 * pi;
    }
    while val < -pi {
        val += 2.0 * pi;
    }
    let x2 = val * val;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    1.0 - (x2 / 2.0) + (x4 / 24.0) - (x6 / 720.0)
}

fn my_sin(x: f32) -> f32 {
    let pi = 3.14159265;
    let mut val = x;
    while val > pi {
        val -= 2.0 * pi;
    }
    while val < -pi {
        val += 2.0 * pi;
    }
    let x2 = val * val;
    let x3 = val * x2;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    val - (x3 / 6.0) + (x5 / 120.0) - (x7 / 5040.0)
}

fn my_atan2(y: f32, x: f32) -> f32 {
    let pi = 3.14159265;
    let half_pi = pi / 2.0;

    if x == 0.0 {
        if y > 0.0 {
            return half_pi;
        } else if y < 0.0 {
            return -half_pi;
        } else {
            return 0.0;
        }
    }

    let z = y / x;
    let abs_z = if z < 0.0 { -z } else { z };

    let mut atan = if abs_z <= 1.0 {
        z * (pi / 4.0 + 0.273 * (1.0 - abs_z))
    } else {
        let rec = 1.0 / z;
        let abs_rec = if rec < 0.0 { -rec } else { rec };
        let term = rec * (pi / 4.0 + 0.273 * (1.0 - abs_rec));
        if z > 0.0 {
            half_pi - term
        } else {
            -half_pi - term
        }
    };

    if x < 0.0 {
        if y >= 0.0 {
            atan += pi;
        } else {
            atan -= pi;
        }
    }

    atan
}


impl TuiDoom {
    pub const fn new() -> Self {
        TuiDoom {
            player_x: 2.2,
            player_y: 2.2,
            player_angle: 0.0,
            map: [
                [1, 1, 1, 1, 1, 1, 1, 1],
                [1, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 1, 1, 0, 1, 0, 1],
                [1, 0, 1, 0, 0, 1, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 1, 0, 1, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1],
            ],
            score: 0,
            ammo: 50,
            health: 100,
            gun_recoil: 0,
            enemy_x: 5.5,
            enemy_y: 5.5,
            enemy_health: 50,
            enemy_dead: false,
        }
    }

    pub fn draw(&mut self) {
        // Redraw backdrop: Sky (Black 0) and Floor (Brown 6 or Dark Gray 8)
        VGA.draw_rect(12, 28, 296, 144, 4); // deep red backing container

        // viewport boundaries
        let view_x = 16;
        let view_y = 44;
        let view_w = 288;
        let view_h = 100;

        // Draw Sky (Deep Blue/Black 0) and Ground (Gray 8)
        VGA.draw_rect(view_x, view_y, view_w, view_h / 2, 0);
        VGA.draw_rect(view_x, view_y + view_h / 2, view_w, view_h / 2, 8);

        // Raycasting loop
        let num_rays = 36;
        let fov = 1.047; // 60 degrees fov
        let half_fov = fov / 2.0;

        for r in 0..num_rays {
            let ray_angle = self.player_angle - half_fov + (r as f32 / num_rays as f32) * fov;
            
            let mut dist = 0.0;
            let mut hit_wall = false;
            let mut hit_enemy = false;
            let mut enemy_dist = 0.0;
            
            let step = 0.08;
            
            while dist < 10.0 {
                dist += step;
                let rx = self.player_x + my_cos(ray_angle) * dist;
                let ry = self.player_y + my_sin(ray_angle) * dist;
                
                let mx = rx as usize;
                let my = ry as usize;
                
                // Check if hit enemy (distance to enemy less than 0.22)
                if !self.enemy_dead {
                    let ex = rx - self.enemy_x;
                    let ey = ry - self.enemy_y;
                    if (ex * ex + ey * ey) < 0.06 {
                        hit_enemy = true;
                        enemy_dist = dist;
                    }
                }
                
                if mx < 8 && my < 8 {
                    if self.map[my][mx] > 0 {
                        hit_wall = true;
                        break;
                    }
                }
            }

            // Draw wall column
            if hit_wall && (!hit_enemy || dist < enemy_dist) {
                let corrected_dist = dist * my_cos(ray_angle - self.player_angle);
                let col_w = view_w / num_rays;
                let col_x = view_x + r * col_w;
                
                let mut wall_height = (75.0 / corrected_dist) as i32;
                if wall_height > view_h as i32 {
                    wall_height = view_h as i32;
                }
                
                let wall_top = view_y + (view_h - wall_height as usize) / 2;
                
                // Render beautiful textured colored walls (red bricks in 3D!)
                let color = if corrected_dist < 2.5 {
                    12 // bright red
                } else if corrected_dist < 5.0 {
                    4  // deep red
                } else {
                    0  // shadow black
                };
                
                VGA.draw_rect(col_x, wall_top, col_w, wall_height as usize, color);
                
                // Draw a nice pixel line to simulate brick joints
                if r % 2 == 0 {
                    VGA.draw_rect(col_x, wall_top, 1, wall_height as usize, 15); // white joints
                }
            }
            
            // Draw active dynamic enemy columns in front of walls!
            if hit_enemy && (!hit_wall || enemy_dist < dist) {
                let corrected_enemy_dist = enemy_dist * my_cos(ray_angle - self.player_angle);
                let col_w = view_w / num_rays;
                let col_x = view_x + r * col_w;
                
                let mut enemy_height = (60.0 / corrected_enemy_dist) as i32;
                if enemy_height > 70 { enemy_height = 70; }
                
                let enemy_top = view_y + (view_h - enemy_height as usize) / 2;
                
                // Render 3D red demon columns
                VGA.draw_rect(col_x, enemy_top, col_w, enemy_height as usize, 13); // Violet/Magenta demon body
                VGA.draw_rect(col_x + 1, enemy_top + 4, 1, 2, 14); // Yellow eyes
            }
        }

        // Draw crosshair
        VGA.draw_rect(158, 92, 4, 1, 10); // Green
        VGA.draw_rect(159, 90, 1, 5, 10);

        // ==========================================
        // Animated Weapon Gun Sprite with Recoil Flash!
        // ==========================================
        let gun_base_x = 145;
        let mut gun_base_y = 115;
        
        if self.gun_recoil > 0 {
            // Shake and lower gun on recoil
            gun_base_y += 6;
            self.gun_recoil -= 1;
            
            // Draw HUGE animated yellow/orange muzzle flash star!
            VGA.draw_rect(148, 94, 24, 22, 14); // Yellow backing
            VGA.draw_rect(152, 98, 16, 14, 12); // Red center
            VGA.draw_rect(156, 102, 8, 6, 15);  // White core
        }
        
        // Render Double Barrel Shotgun weapon barrel
        VGA.draw_rect(gun_base_x + 8, gun_base_y, 4, 30, 8);  // Left barrel (charcoal)
        VGA.draw_rect(gun_base_x + 13, gun_base_y, 4, 30, 8); // Right barrel
        VGA.draw_rect(gun_base_x + 6, gun_base_y + 12, 13, 18, 0); // Black backing stock
        VGA.draw_rect(gun_base_x + 10, gun_base_y - 2, 5, 2, 7);  // Silver barrel tips

        // ==========================================
        // Premium HUD Status Dashboard Panel
        // ==========================================
        VGA.draw_rect(16, 144, 288, 22, 0); // black HUD backing
        VGA.draw_rect(17, 145, 286, 20, 8); // charcoal inner

        // Draw mini Doomguy pixel face
        let face_bg = if self.health > 50 { 10 } else { 12 }; // Green for healthy, Red for critical
        VGA.draw_rect(24, 147, 16, 16, face_bg);
        VGA.draw_rect(28, 151, 2, 2, 15); // Left eye
        VGA.draw_rect(34, 151, 2, 2, 15); // Right eye
        VGA.draw_rect(28, 157, 8, 2, 0);  // Smile / Mouth

        VGA.draw_string(46, 151, "SCORE:", 11);
        let mut sc_buf = [b'0'; 4];
        let score_val = if self.score > 9999 { 9999 } else { self.score };
        sc_buf[0] = b'0' + (score_val / 1000) as u8;
        sc_buf[1] = b'0' + ((score_val / 100) % 10) as u8;
        sc_buf[2] = b'0' + ((score_val / 10) % 10) as u8;
        sc_buf[3] = b'0' + (score_val % 10) as u8;
        VGA.draw_string(94, 151, core::str::from_utf8(&sc_buf).unwrap(), 14);

        VGA.draw_string(140, 151, "AMMO:", 11);
        let mut am_buf = [b'0'; 3];
        let ammo_val = if self.ammo > 999 { 999 } else { self.ammo };
        am_buf[0] = b'0' + (ammo_val / 100) as u8;
        am_buf[1] = b'0' + ((ammo_val / 10) % 10) as u8;
        am_buf[2] = b'0' + (ammo_val % 10) as u8;
        VGA.draw_string(180, 151, core::str::from_utf8(&am_buf).unwrap(), 14);

        VGA.draw_string(216, 151, "HEALTH:", 11);
        let mut hp_buf = [b'0'; 3];
        let hp_val = if self.health > 999 { 999 } else { self.health };
        hp_buf[0] = b'0' + (hp_val / 100) as u8;
        hp_buf[1] = b'0' + ((hp_val / 10) % 10) as u8;
        hp_buf[2] = b'0' + (hp_val % 10) as u8;
        VGA.draw_string(268, 151, core::str::from_utf8(&hp_buf).unwrap(), 12);

        // Header Title
        VGA.draw_rect(12, 28, 296, 12, 0); // black bar
        VGA.draw_string(16, 30, "UloDOOM 3D: Bare-Metal Engine", 12); // bright red

        // Render kill victory state info!
        if self.enemy_dead {
            VGA.draw_string(215, 30, "DEMON SLAIN!", 10); // green victory
        } else {
            VGA.draw_string(215, 30, "TARGET ACTIVE", 13);
        }

        // Instructions Gutter
        VGA.draw_string(16, 172, "Controls: [W/S] Move | [A/D] Turn | [Space] Fire Shotgun", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        let move_speed = 0.28;
        let rot_speed = 0.20;

        match key {
            'w' | 'W' => {
                let nx = self.player_x + my_cos(self.player_angle) * move_speed;
                let ny = self.player_y + my_sin(self.player_angle) * move_speed;
                
                if self.map[ny as usize][nx as usize] == 0 {
                    self.player_x = nx;
                    self.player_y = ny;
                }
                
                // If close to enemy, let them attack player!
                if !self.enemy_dead {
                    let ex = self.player_x - self.enemy_x;
                    let ey = self.player_y - self.enemy_y;
                    if (ex * ex + ey * ey) < 0.5 {
                        if self.health > 10 {
                            self.health -= 10;
                        }
                    }
                }
            }
            's' | 'S' => {
                let nx = self.player_x - my_cos(self.player_angle) * move_speed;
                let ny = self.player_y - my_sin(self.player_angle) * move_speed;
                
                if self.map[ny as usize][nx as usize] == 0 {
                    self.player_x = nx;
                    self.player_y = ny;
                }
            }
            'a' | 'A' => {
                self.player_angle -= rot_speed;
            }
            'd' | 'D' => {
                self.player_angle += rot_speed;
            }
            ' ' => {
                if self.ammo > 0 {
                    self.ammo -= 1;
                    self.gun_recoil = 3; // Trigger flash recoil
                    
                    // Gun Shot Sound
                    unsafe {
                        crate::sound::play_tone(150);
                        for _ in 0..15_000 { core::arch::asm!("nop") }
                        crate::sound::play_tone(80);
                        for _ in 0..15_000 { core::arch::asm!("nop") }
                        crate::sound::stop_speaker();
                    }

                    // Check shot hit on the 3D demon!
                    if !self.enemy_dead {
                        // Calculate direction from player to enemy
                        let dx = self.enemy_x - self.player_x;
                        let dy = self.enemy_y - self.player_y;
                        
                        let enemy_dir = my_atan2(dy, dx);
                        let pi = 3.14159265;
                        
                        // normalize angle
                        let mut diff = enemy_dir - self.player_angle;
                        while diff > pi { diff -= 2.0 * pi; }
                        while diff < -pi { diff += 2.0 * pi; }
                        
                        // Hit confirmed if within raycast FOV center
                        if diff.abs() < 0.24 {
                            // plays hit confirm tone!
                            unsafe {
                                crate::sound::play_tone(600);
                                for _ in 0..8_000 { core::arch::asm!("nop") }
                                crate::sound::stop_speaker();
                            }
                            
                            self.enemy_health -= 25;
                            if self.enemy_health <= 0 {
                                self.enemy_dead = true;
                                self.score += 100;
                                
                                // Victory Death Chime!
                                unsafe {
                                    crate::sound::play_tone(440);
                                    for _ in 0..10_000 { core::arch::asm!("nop") }
                                    crate::sound::play_tone(554);
                                    for _ in 0..10_000 { core::arch::asm!("nop") }
                                    crate::sound::play_tone(659);
                                    for _ in 0..15_000 { core::arch::asm!("nop") }
                                    crate::sound::stop_speaker();
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
