use crate::vga_driver::VGA;

pub struct TuiDoom {
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32, // in radians
    pub map: [[u8; 8]; 8],
    pub score: i32,
    pub ammo: i32,
    pub health: i32,
}

impl TuiDoom {
    pub const fn new() -> Self {
        TuiDoom {
            player_x: 3.5,
            player_y: 3.5,
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
            ammo: 30,
            health: 100,
        }
    }

    pub fn draw(&self) {
        // Red cockpit base outline (retro HUD style)
        VGA.draw_rect(12, 28, 296, 144, 4);

        // Draw HUD details
        VGA.draw_string(16, 32, "DOOM 3D BARE-METAL", 15);
        
        VGA.draw_string(150, 32, "AMMO: ", 15);
        let mut ammo_buf = [b'0'; 2];
        ammo_buf[0] = b'0' + (self.ammo / 10) as u8;
        ammo_buf[1] = b'0' + (self.ammo % 10) as u8;
        VGA.draw_string(190, 32, core::str::from_utf8(&ammo_buf).unwrap(), 14);

        VGA.draw_string(230, 32, "HP: ", 15);
        let mut health_buf = [b'0'; 3];
        health_buf[2] = b'0' + (self.health % 10) as u8;
        health_buf[1] = b'0' + ((self.health / 10) % 10) as u8;
        VGA.draw_string(254, 32, core::str::from_utf8(&health_buf).unwrap(), 14);

        // Viewport dimensions: x = 16 to 304 (288 px width), y = 44 to 164 (120 px height)
        // Draw sky (blue) and ground (brown) in the viewport first
        VGA.draw_rect(16, 44, 288, 60, 1);  // Blue sky
        VGA.draw_rect(16, 104, 288, 60, 6); // Brown soil ground

        // Render pseudo-3D Raycasted walls!
        // We cast 24 rays across the 288px width viewport (each column is 12px wide)
        let num_rays = 24;
        let fov = 1.047; // 60 degrees in radians
        let half_fov = fov / 2.0;

        for r in 0..num_rays {
            // Calculate angle for current ray
            let ray_angle = self.player_angle - half_fov + (r as f32 / num_rays as f32) * fov;
            
            // Simple DDA / Raymarching through grid map
            let mut dist = 0.0;
            let mut hit_wall = false;
            let step = 0.1;
            
            // Limit ray length to prevent infinite loops
            while dist < 12.0 {
                dist += step;
                let rx = self.player_x + ray_angle.cos() * dist;
                let ry = self.player_y + ray_angle.sin() * dist;
                
                let mx = rx as usize;
                let my = ry as usize;
                
                if mx < 8 && my < 8 {
                    if self.map[my][mx] > 0 {
                        hit_wall = true;
                        break;
                    }
                }
            }

            if hit_wall {
                // Correct fisheye effect
                let corrected_dist = dist * (ray_angle - self.player_angle).cos();
                
                // Calculate projected wall height
                let mut wall_height = (120.0 / corrected_dist) as i32;
                if wall_height > 110 {
                    wall_height = 110;
                }
                
                let wall_top = 104 - (wall_height / 2);
                let col_x = 16 + r * 12;
                
                // Shade color based on distance to give dynamic depth shading!
                let wall_color = if corrected_dist < 3.0 {
                    7  // Light gray (close)
                } else if corrected_dist < 6.0 {
                    8  // Dark gray (medium)
                } else {
                    0  // Black (far)
                };

                VGA.draw_rect(col_x, wall_top as usize, 12, wall_height as usize, wall_color);
            }
        }

        // Draw crosshair in the center of the viewport
        VGA.draw_rect(156, 102, 8, 1, 12); // Red crosshair
        VGA.draw_rect(159, 99, 1, 7, 12);

        // Control instructions footer
        VGA.draw_string(16, 172, "Controls: [W/S] Move | [A/D] Turn | [Space] Fire", 8);
    }

    pub fn handle_input(&mut self, key: char) {
        let move_speed = 0.35;
        let rot_speed = 0.25;

        match key {
            'w' | 'W' => {
                let nx = self.player_x + self.player_angle.cos() * move_speed;
                let ny = self.player_y + self.player_angle.sin() * move_speed;
                
                if self.map[ny as usize][nx as usize] == 0 {
                    self.player_x = nx;
                    self.player_y = ny;
                }
            }
            's' | 'S' => {
                let nx = self.player_x - self.player_angle.cos() * move_speed;
                let ny = self.player_y - self.player_angle.sin() * move_speed;
                
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
                    // Play a quick satisfying bullet tone on PC Speaker hardware!
                    unsafe {
                        crate::sound::play_tone(880);
                        for _ in 0..10_000 { core::arch::asm!("nop") }
                        crate::sound::play_tone(440);
                        for _ in 0..10_000 { core::arch::asm!("nop") }
                        crate::sound::stop_speaker();
                    }
                }
            }
            _ => {}
        }
    }
}
