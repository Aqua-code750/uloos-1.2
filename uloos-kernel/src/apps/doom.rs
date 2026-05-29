use crate::vga_driver::VGA;

pub struct TuiDoom {
    pub player_x: i32,
    pub player_y: i32,
    pub enemies: [(i32, i32, bool); 3],
    pub score: i32,
    pub ammo: i32,
    pub health: i32,
}

impl TuiDoom {
    pub const fn new() -> Self {
        TuiDoom {
            player_x: 3,
            player_y: 5,
            enemies: [
                (7, 3, true),
                (12, 6, true),
                (15, 4, true),
            ],
            score: 0,
            ammo: 30,
            health: 100,
        }
    }

    pub fn draw(&self) {
        // Red background base
        VGA.draw_rect(12, 28, 296, 144, 4);

        // Draw HUD details
        VGA.draw_string(16, 32, "SCORE: ", 15);
        let mut score_buf = [b'0'; 3];
        score_buf[2] = b'0' + (self.score % 10) as u8;
        score_buf[1] = b'0' + ((self.score / 10) % 10) as u8;
        VGA.draw_string(64, 32, core::str::from_utf8(&score_buf).unwrap(), 14);

        VGA.draw_string(110, 32, "AMMO: ", 15);
        let mut ammo_buf = [b'0'; 2];
        ammo_buf[0] = b'0' + (self.ammo / 10) as u8;
        ammo_buf[1] = b'0' + (self.ammo % 10) as u8;
        VGA.draw_string(150, 32, core::str::from_utf8(&ammo_buf).unwrap(), 14);

        VGA.draw_string(190, 32, "HP: ", 15);
        let mut health_buf = [b'1', b'0', b'0'];
        if self.health < 100 {
            health_buf[0] = b' ';
            health_buf[1] = b'0' + (self.health / 10) as u8;
            health_buf[2] = b'0' + (self.health % 10) as u8;
        }
        VGA.draw_string(220, 32, core::str::from_utf8(&health_buf).unwrap(), 14);

        // Simulated game viewport (Black screen)
        VGA.draw_rect(16, 44, 288, 120, 0);

        // Draw active enemies
        for &(ex, ey, alive) in self.enemies.iter() {
            if alive {
                let px = 20 + ex * 12;
                let py = 50 + ey * 8;
                VGA.draw_string(px as usize, py as usize, "Caco (E)", 12);
            }
        }

        // Draw Player
        let px = 20 + self.player_x * 12;
        let py = 50 + self.player_y * 8;
        VGA.draw_string(px as usize, py as usize, "(ToT)", 10); // Green player
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            'w' | 'W' => { if self.player_y > 1 { self.player_y -= 1; } }
            's' | 'S' => { if self.player_y < 12 { self.player_y += 1; } }
            'a' | 'A' => { if self.player_x > 1 { self.player_x -= 1; } }
            'd' | 'D' => { if self.player_x < 20 { self.player_x += 1; } }
            ' ' => {
                if self.ammo > 0 {
                    self.ammo -= 1;
                    for idx in 0..3 {
                        let (ex, ey, alive) = self.enemies[idx];
                        if alive && ey == self.player_y && ex > self.player_x {
                            self.enemies[idx].2 = false;
                            self.score += 10;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
