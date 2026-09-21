use macroquad::prelude::*;
use crate::ship::{Character, ShipGrid};

pub struct ShipRenderer {
    pub cell_size: f32,
    pub view_offset: Vec2,
}

impl ShipRenderer {
    pub fn new() -> Self {
        Self {
            cell_size: 48.0,
            view_offset: Vec2::ZERO,
        }
    }

    pub fn update_character_input(
        &mut self,
        ship: &mut ShipGrid,
        character: &mut Character,
        dt: f32,
        particles: &mut crate::render::particles::ParticleSystem,
    ) {
        if character.in_ship {
            let mut move_dir = Vec2::ZERO;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                move_dir.y += 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                move_dir.y -= 1.0;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                move_dir.x -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                move_dir.x += 1.0;
            }

            if move_dir != Vec2::ZERO {
                let step = move_dir.normalize() * 3.5 * dt;
                let next_pos = character.local_pos + step;

                // Check collision with walkable ship blocks
                let grid_x = next_pos.x.round() as i32;
                let grid_y = next_pos.y.round() as i32;
                if ship.can_walk_to(grid_x, grid_y) {
                    character.local_pos = next_pos;
                }
            }

            // Check if standing on airlock block for EVA exit
            let cur_x = character.local_pos.x.round() as i32;
            let cur_y = character.local_pos.y.round() as i32;
            if let Some(b) = ship.blocks.get(&(cur_x, cur_y)) {
                if b.is_eva_exit && is_key_pressed(KeyCode::E) {
                    let _ = ship.exit_to_eva(character, (cur_x, cur_y));
                }
            }
        } else {
            // Character is in EVA (spacewalking outside ship)
            let mut eva_thrust = Vec2::ZERO;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                eva_thrust.y += 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                eva_thrust.y -= 1.0;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                eva_thrust.x -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                eva_thrust.x += 1.0;
            }

            if eva_thrust != Vec2::ZERO {
                let norm_thrust = eva_thrust.normalize();
                character.velocity += norm_thrust * 4.0 * dt;
                particles.emit_eva_puff(character.world_pos, norm_thrust);
            }
            character.velocity *= 0.98; // slight space damping
            character.world_pos += character.velocity * dt * 30.0;

            // Check if near any ship airlock to board back
            if is_key_pressed(KeyCode::E) {
                for (&(ax, ay), block) in &ship.blocks {
                    if block.is_eva_exit {
                        if ship.board_ship(character, (ax, ay)).is_ok() {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&self, ship: &ShipGrid, character: &Character) {
        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

        let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);

        // 1. Draw Starfield in background
        for i in 0..100 {
            let sx = (i * 37) as f32 % screen_width();
            let sy = (i * 59) as f32 % screen_height();
            draw_circle(sx, sy, 1.2, Color::new(0.8, 0.8, 0.9, 0.4));
        }

        // 2. Draw Ship Blocks
        let time = get_time() as f32;

        for (&(gx, gy), block) in &ship.blocks {
            // Screen position for grid cell (gx, gy)
            // Note: in grid y goes up, in screen y goes down
            let px = screen_center.x + (gx as f32) * self.cell_size - self.cell_size * 0.5;
            let py = screen_center.y - (gy as f32) * self.cell_size - self.cell_size * 0.5;

            match block.def_id.as_str() {
                "cockpit" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.15, 0.35, 0.55, 1.0));
                    draw_rectangle(px + 4.0, py + 4.0, self.cell_size - 8.0, self.cell_size - 8.0, Color::new(0.2, 0.6, 0.85, 1.0));
                    draw_text("BRIDGE", px + 6.0, py + self.cell_size * 0.6, 10.0, WHITE);
                }
                "corridor" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.2, 0.22, 0.28, 1.0));
                    draw_rectangle_lines(px, py, self.cell_size, self.cell_size, 1.5, Color::new(0.3, 0.35, 0.45, 1.0));
                    draw_line(px + 8.0, py + self.cell_size * 0.5, px + self.cell_size - 8.0, py + self.cell_size * 0.5, 1.0, Color::new(0.4, 0.45, 0.55, 0.5));
                }
                "fusion_reactor" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.25, 0.15, 0.1, 1.0));
                    let pulse = (time * 4.0).sin() * 0.2 + 0.8;
                    draw_circle(px + self.cell_size * 0.5, py + self.cell_size * 0.5, self.cell_size * 0.35 * pulse, Color::new(1.0, 0.6, 0.1, 0.9));
                    draw_text("REACTOR", px + 4.0, py + self.cell_size * 0.6, 9.0, WHITE);
                }
                "ion_thruster" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.25, 0.25, 0.3, 1.0));
                    draw_rectangle_lines(px, py, self.cell_size, self.cell_size, 1.5, Color::new(0.4, 0.4, 0.5, 1.0));
                    // Animated thrust flame
                    let flame_len = (time * 12.0).sin().abs() * 16.0 + 12.0;
                    draw_triangle(
                        Vec2::new(px + self.cell_size * 0.2, py + self.cell_size),
                        Vec2::new(px + self.cell_size * 0.8, py + self.cell_size),
                        Vec2::new(px + self.cell_size * 0.5, py + self.cell_size + flame_len),
                        Color::new(0.2, 0.7, 1.0, 0.85),
                    );
                    draw_text("ENG", px + 12.0, py + self.cell_size * 0.6, 10.0, WHITE);
                }
                "cargo_bay" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.35, 0.3, 0.15, 1.0));
                    draw_rectangle_lines(px, py, self.cell_size, self.cell_size, 1.5, Color::new(0.8, 0.7, 0.2, 0.8));
                    draw_text("CARGO", px + 6.0, py + self.cell_size * 0.6, 10.0, YELLOW);
                }
                "airlock" => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.4, 0.35, 0.1, 1.0));
                    draw_rectangle_lines(px, py, self.cell_size, self.cell_size, 2.0, Color::new(1.0, 0.9, 0.1, 1.0));
                    draw_text("AIRLOCK", px + 4.0, py + self.cell_size * 0.6, 9.0, WHITE);
                }
                _ => {
                    draw_rectangle(px, py, self.cell_size, self.cell_size, Color::new(0.3, 0.3, 0.35, 1.0));
                    draw_rectangle_lines(px, py, self.cell_size, self.cell_size, 1.0, GRAY);
                }
            }
        }

        // 3. Draw Center of Mass (CoM) crosshair
        let com_screen_x = screen_center.x + ship.center_of_mass.x * self.cell_size;
        let com_screen_y = screen_center.y - ship.center_of_mass.y * self.cell_size;
        draw_circle_lines(com_screen_x, com_screen_y, 8.0, 1.5, Color::new(1.0, 0.2, 0.2, 0.8));
        draw_line(com_screen_x - 12.0, com_screen_y, com_screen_x + 12.0, com_screen_y, 1.0, Color::new(1.0, 0.2, 0.2, 0.8));
        draw_line(com_screen_x, com_screen_y - 12.0, com_screen_x, com_screen_y + 12.0, 1.0, Color::new(1.0, 0.2, 0.2, 0.8));
        draw_text("CoM", com_screen_x + 10.0, com_screen_y - 5.0, 10.0, Color::new(1.0, 0.4, 0.4, 0.8));

        // 4. Draw Character
        if character.in_ship {
            let char_px = screen_center.x + character.local_pos.x * self.cell_size;
            let char_py = screen_center.y - character.local_pos.y * self.cell_size;

            // Character body (astronaut sprite / circle)
            draw_circle(char_px, char_py, 10.0, Color::new(0.9, 0.9, 0.95, 1.0));
            // Visor
            draw_circle(char_px, char_py - 2.0, 4.5, Color::new(0.2, 0.8, 1.0, 1.0));

            // Name tag
            draw_text(&character.name, char_px - 25.0, char_py - 16.0, 12.0, WHITE);

            // Airlock action hint
            let cur_x = character.local_pos.x.round() as i32;
            let cur_y = character.local_pos.y.round() as i32;
            if let Some(b) = ship.blocks.get(&(cur_x, cur_y)) {
                if b.is_eva_exit {
                    draw_text("[E] EXIT TO OPEN SPACE (EVA)", char_px - 85.0, char_py + 25.0, 13.0, Color::new(1.0, 0.9, 0.2, 1.0));
                }
            }
        } else {
            // Character in EVA (floating outside ship)
            let eva_px = screen_center.x + (character.world_pos.x - ship.world_position.x) * (self.cell_size * 0.15);
            let eva_py = screen_center.y - (character.world_pos.y - ship.world_position.y) * (self.cell_size * 0.15);

            // Jetpack thruster trail
            let jet = (time * 20.0).sin().abs() * 6.0;
            draw_circle(eva_px - 8.0, eva_py + 4.0, 3.0 + jet * 0.3, Color::new(0.3, 0.9, 1.0, 0.7));

            // EVA Astronaut
            draw_circle(eva_px, eva_py, 11.0, Color::new(0.95, 0.95, 1.0, 1.0));
            draw_circle(eva_px + 2.0, eva_py - 2.0, 5.0, Color::new(1.0, 0.8, 0.2, 1.0)); // Gold visor
            draw_text(&format!("{} (EVA)", character.name), eva_px - 35.0, eva_py - 16.0, 12.0, Color::new(0.4, 0.9, 1.0, 1.0));

            draw_text("[E] BOARD VESSEL (near airlock)", eva_px - 80.0, eva_py + 25.0, 12.0, Color::new(1.0, 0.9, 0.2, 1.0));
        }

        // 5. Modern Ship Diagnostics HUD
        let hud_x = 20.0;
        let hud_y = 60.0;
        let hud_w = 340.0;
        let hud_h = 165.0;

        crate::render::ui::SciFiUi::draw_panel(
            hud_x,
            hud_y,
            hud_w,
            hud_h,
            Some(&format!("VESSEL // {}", ship.name.to_uppercase())),
            Color::new(0.0, 0.85, 1.0, 0.85),
        );

        let info_x = hud_x + 16.0;
        draw_text(&format!("Total Mass: {:.1} t  |  Inertia: {:.1}", ship.total_mass, ship.moment_of_inertia), info_x, hud_y + 50.0, 12.0, LIGHTGRAY);
        draw_text(&format!("Center of Mass: ({:.2}, {:.2})", ship.center_of_mass.x, ship.center_of_mass.y), info_x, hud_y + 68.0, 12.0, LIGHTGRAY);

        // Modules count & status
        draw_text(&format!("Hull Modules: {} online", ship.blocks.len()), info_x, hud_y + 88.0, 12.0, Color::new(0.2, 0.9, 0.4, 1.0));

        // Oxygen Bar
        draw_text("Oxygen Supply:", info_x, hud_y + 110.0, 12.0, WHITE);
        let oxy_ratio = (character.oxygen / 100.0).clamp(0.0, 1.0);
        draw_rectangle(info_x + 105.0, hud_y + 100.0, 195.0, 12.0, Color::new(0.12, 0.15, 0.22, 1.0));
        draw_rectangle(info_x + 105.0, hud_y + 100.0, 195.0 * oxy_ratio, 12.0, Color::new(0.0, 0.8, 1.0, 1.0));
        draw_rectangle_lines(info_x + 105.0, hud_y + 100.0, 195.0, 12.0, 1.0, DARKGRAY);

        // Character Status Badge
        let (status_str, status_col) = if character.in_ship {
            ("STATUS: CREW STATIONED (INTERNAL)", Color::new(0.2, 0.9, 0.4, 1.0))
        } else {
            ("STATUS: EVA SPACEWALK (EXTERIOR)", Color::new(1.0, 0.8, 0.2, 1.0))
        };
        draw_text(status_str, info_x, hud_y + 138.0, 12.0, status_col);
    }
}
