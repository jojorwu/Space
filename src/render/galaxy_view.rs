use std::collections::HashMap;
use macroquad::prelude::*;
use crate::factions::{FactionManager, FleetMission};
use crate::galaxy::{Galaxy, PlanetType};
use crate::items::ItemDef;
use crate::render::camera::PanZoomCamera;
use crate::render::menu::GameSettings;
use crate::render::particles::ParticleSystem;
use crate::render::ui::SciFiUi;
use crate::traders::TradeFleet;

pub struct NebulaCloud {
    pub position: Vec2,
    pub radius: f32,
    pub color: Color,
}

pub struct GalaxyRenderer {
    pub selected_planet: Option<usize>,
    pub background_stars: Vec<(Vec2, f32, f32)>, // (position, size, brightness)
    pub nebulae: Vec<NebulaCloud>,
}

impl GalaxyRenderer {
    pub fn new() -> Self {
        let mut stars = Vec::with_capacity(320);
        for i in 0..320 {
            let x = ((i * 7919) % 4000) as f32 - 2000.0;
            let y = ((i * 4973) % 4000) as f32 - 2000.0;
            let size = ((i % 5) as f32) * 0.4 + 0.8;
            let brightness = 0.35 + ((i % 7) as f32) * 0.09;
            stars.push((Vec2::new(x, y), size, brightness));
        }

        // Procedural Cosmic Nebulae
        let mut nebulae = Vec::new();
        // Core blue/cyan clouds
        nebulae.push(NebulaCloud { position: Vec2::new(-80.0, -50.0), radius: 360.0, color: Color::new(0.0, 0.45, 0.85, 0.09) });
        nebulae.push(NebulaCloud { position: Vec2::new(120.0, 90.0), radius: 310.0, color: Color::new(0.1, 0.65, 0.95, 0.08) });
        // Neutral emerald/teal clouds
        nebulae.push(NebulaCloud { position: Vec2::new(-450.0, 280.0), radius: 420.0, color: Color::new(0.05, 0.75, 0.45, 0.07) });
        nebulae.push(NebulaCloud { position: Vec2::new(380.0, -320.0), radius: 380.0, color: Color::new(0.1, 0.80, 0.60, 0.06) });
        // Frontier violet/crimson rift clouds
        nebulae.push(NebulaCloud { position: Vec2::new(750.0, 520.0), radius: 520.0, color: Color::new(0.70, 0.15, 0.55, 0.08) });
        nebulae.push(NebulaCloud { position: Vec2::new(-680.0, -620.0), radius: 480.0, color: Color::new(0.85, 0.25, 0.25, 0.07) });
        nebulae.push(NebulaCloud { position: Vec2::new(150.0, 850.0), radius: 460.0, color: Color::new(0.55, 0.10, 0.80, 0.08) });

        Self {
            selected_planet: None,
            background_stars: stars,
            nebulae,
        }
    }

    pub fn handle_click(&mut self, camera: &PanZoomCamera, galaxy: &Galaxy) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = Vec2::new(mouse_position().0, mouse_position().1);

            // Check if clicked inside inspector card close button
            if self.selected_planet.is_some() {
                let card_x = 20.0;
                let card_y = screen_height() - 275.0;
                let card_w = 410.0;
                let card_h = 255.0;

                // Close button area
                if mouse.x >= card_x + card_w - 35.0 && mouse.x <= card_x + card_w - 5.0
                    && mouse.y >= card_y + 5.0 && mouse.y <= card_y + 30.0
                {
                    self.selected_planet = None;
                    return;
                }

                if mouse.x >= card_x && mouse.x <= card_x + card_w && mouse.y >= card_y && mouse.y <= card_y + card_h {
                    return;
                }
            }

            let world_mouse = camera.screen_to_world(mouse);
            let mut clicked = None;
            for planet in &galaxy.planets {
                let planet_pos = Vec2::new(planet.position.x, planet.position.y);
                let hit_radius = (planet.population_millions.sqrt() as f32 * 0.5).max(18.0) / camera.zoom.max(0.5);
                if world_mouse.distance(planet_pos) <= hit_radius {
                    clicked = Some(planet.id);
                    break;
                }
            }
            self.selected_planet = clicked;
        }
    }

    pub fn draw(
        &mut self,
        camera: &PanZoomCamera,
        galaxy: &Galaxy,
        factions: &FactionManager,
        traders: &TradeFleet,
        items: &HashMap<String, ItemDef>,
        settings: &GameSettings,
        particles: &mut ParticleSystem,
    ) {
        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

        // 1. Draw Cosmic Deep Space Nebulae
        for neb in &self.nebulae {
            let s_pos = camera.world_to_screen(neb.position);
            let s_rad = neb.radius * camera.zoom;
            // Draw soft layered gradients
            for layer in (1..=4).rev() {
                let factor = layer as f32 / 4.0;
                let mut c = neb.color;
                c.a = neb.color.a * (1.0 - factor * 0.4);
                draw_circle(s_pos.x, s_pos.y, s_rad * factor, c);
            }
        }

        // 2. Draw Starfield
        for &(pos, size, bright) in &self.background_stars {
            let screen_pos = camera.world_to_screen(pos);
            if screen_pos.x >= -10.0 && screen_pos.x <= screen_width() + 10.0
                && screen_pos.y >= -10.0 && screen_pos.y <= screen_height() + 10.0
            {
                draw_circle(screen_pos.x, screen_pos.y, size * camera.zoom.clamp(0.8, 1.5), Color::new(bright, bright, bright * 1.1, bright));
            }
        }

        // 3. Draw Galactic Zone boundaries
        let center_screen = camera.world_to_screen(Vec2::ZERO);
        let core_r = 350.0 * camera.zoom;
        let neutral_r = 750.0 * camera.zoom;
        let frontier_r = 1200.0 * camera.zoom;

        draw_circle_lines(center_screen.x, center_screen.y, frontier_r, 1.5, Color::new(0.85, 0.2, 0.2, 0.20));
        draw_circle_lines(center_screen.x, center_screen.y, neutral_r, 1.5, Color::new(0.2, 0.85, 0.4, 0.20));
        draw_circle_lines(center_screen.x, center_screen.y, core_r, 2.0, Color::new(0.2, 0.65, 1.0, 0.28));

        if camera.zoom > 0.25 {
            draw_text("FRONTIER RIM", center_screen.x - 50.0, center_screen.y - frontier_r + 18.0, 14.0, Color::new(0.9, 0.3, 0.3, 0.5));
            draw_text("NEUTRAL TRADE EXPANSE", center_screen.x - 80.0, center_screen.y - neutral_r + 18.0, 14.0, Color::new(0.3, 0.9, 0.5, 0.5));
            draw_text("SOLAR CORE", center_screen.x - 40.0, center_screen.y - core_r + 18.0, 14.0, Color::new(0.3, 0.7, 1.0, 0.6));
        }

        // 4. Draw Trade Routes, Moving Merchant Ships & Engine VFX
        for trader in &traders.ships {
            if let Some(dest_id) = trader.destination_planet {
                if let (Some(p_from), Some(p_to)) = (galaxy.get_planet(trader.current_planet), galaxy.get_planet(dest_id)) {
                    let from_world = Vec2::new(p_from.position.x, p_from.position.y);
                    let to_world = Vec2::new(p_to.position.x, p_to.position.y);

                    if settings.show_trade_routes {
                        let from_screen = camera.world_to_screen(from_world);
                        let to_screen = camera.world_to_screen(to_world);
                        draw_line(from_screen.x, from_screen.y, to_screen.x, to_screen.y, 1.0, Color::new(0.2, 0.9, 0.4, 0.25));
                    }

                    // Ship world position
                    let ship_world = from_world.lerp(to_world, trader.travel_progress);
                    let ship_screen = camera.world_to_screen(ship_world);

                    // Emit engine exhaust particles in world space
                    let travel_dir = (to_world - from_world).normalize_or_zero();
                    particles.emit_engine_trail(ship_world, travel_dir, Color::new(0.2, 0.8, 1.0, 0.8), 25.0);

                    // Draw trader vessel
                    draw_circle(ship_screen.x, ship_screen.y, 4.2 * camera.zoom.clamp(0.8, 1.8), Color::new(0.3, 1.0, 0.5, 0.95));
                }
            }
        }

        // 5. Draw Military Armadas, Colony Fleets & Combat VFX
        for fleet in &factions.active_fleets {
            if let (Some(p_from), Some(p_to)) = (galaxy.get_planet(fleet.origin_planet), galaxy.get_planet(fleet.target_planet)) {
                let from_world = Vec2::new(p_from.position.x, p_from.position.y);
                let to_world = Vec2::new(p_to.position.x, p_to.position.y);

                let (line_col, ship_col, trail_col, icon) = match fleet.mission {
                    FleetMission::Colonization { .. } => (
                        Color::new(0.2, 0.8, 1.0, 0.35),
                        Color::new(0.3, 0.9, 1.0, 1.0),
                        Color::new(0.2, 0.8, 1.0, 0.9),
                        "[C]",
                    ),
                    FleetMission::MilitaryInvasion { .. } => (
                        Color::new(1.0, 0.2, 0.2, 0.45),
                        Color::new(1.0, 0.3, 0.3, 1.0),
                        Color::new(1.0, 0.45, 0.1, 0.9),
                        "[W]",
                    ),
                };

                if settings.show_fleet_trails {
                    let from_screen = camera.world_to_screen(from_world);
                    let to_screen = camera.world_to_screen(to_world);
                    draw_line(from_screen.x, from_screen.y, to_screen.x, to_screen.y, 1.5, line_col);
                }

                let fleet_world = from_world.lerp(to_world, fleet.progress);
                let fleet_screen = camera.world_to_screen(fleet_world);

                // Emit fleet engine trail
                let fleet_dir = (to_world - from_world).normalize_or_zero();
                particles.emit_engine_trail(fleet_world, fleet_dir, trail_col, 30.0);

                draw_circle(fleet_screen.x, fleet_screen.y, 5.0 * camera.zoom.clamp(0.8, 2.0), ship_col);
                if camera.zoom > 0.4 {
                    draw_text(icon, fleet_screen.x + 6.0, fleet_screen.y + 4.0, 12.0, ship_col);
                }
            }
        }

        // 6. Draw 100 Planets with Detailed Procedural Aesthetics
        let should_draw_names = match settings.planet_names_idx {
            1 => true,
            2 => false,
            _ => camera.zoom > 0.45,
        };

        for planet in &galaxy.planets {
            let p_world = Vec2::new(planet.position.x, planet.position.y);
            let p_screen = camera.world_to_screen(p_world);

            if p_screen.x < -60.0 || p_screen.x > screen_width() + 60.0
                || p_screen.y < -60.0 || p_screen.y > screen_height() + 60.0
            {
                continue;
            }

            let base_radius = (5.0 + (planet.population_millions.sqrt() as f32 * 0.22)).clamp(6.0, 22.0);
            let screen_radius = base_radius * camera.zoom.clamp(0.3, 3.0);

            let faction_color = if let Some(ref f_id) = planet.owner_faction {
                if let Some(faction) = factions.factions.get(f_id) {
                    let c = faction.flag.primary_color;
                    Color::new(c[0], c[1], c[2], 1.0)
                } else {
                    Color::new(0.6, 0.6, 0.6, 1.0)
                }
            } else {
                Color::new(0.6, 0.62, 0.7, 0.8)
            };

            let has_rings = planet.id % 5 == 0 && screen_radius >= 6.0;

            // Back half of planetary rings
            if has_rings {
                let rx = screen_radius * 2.2;
                let ry = screen_radius * 0.55;
                draw_ellipse_lines(p_screen.x, p_screen.y, rx, ry, -15.0, 1.5, Color::new(0.8, 0.75, 0.65, 0.35));
            }

            // Atmosphere Glow Ring
            draw_circle_lines(p_screen.x, p_screen.y, screen_radius + 3.0, 1.5, Color::new(faction_color.r, faction_color.g, faction_color.b, 0.45));

            // Surface rendering according to PlanetType
            match planet.planet_type {
                PlanetType::AgriWorld => {
                    // Ocean blue sphere
                    draw_circle(p_screen.x, p_screen.y, screen_radius, Color::new(0.12, 0.45, 0.82, 1.0));
                    // Continent patches
                    draw_circle(p_screen.x - screen_radius * 0.3, p_screen.y - screen_radius * 0.2, screen_radius * 0.45, Color::new(0.2, 0.75, 0.35, 0.9));
                    draw_circle(p_screen.x + screen_radius * 0.35, p_screen.y + screen_radius * 0.25, screen_radius * 0.35, Color::new(0.25, 0.7, 0.3, 0.85));
                    // Cloud bands
                    draw_ellipse(p_screen.x, p_screen.y - screen_radius * 0.1, screen_radius * 0.9, screen_radius * 0.2, 0.0, Color::new(1.0, 1.0, 1.0, 0.3));
                }
                PlanetType::MiningColony => {
                    // Rust-red / copper sphere
                    draw_circle(p_screen.x, p_screen.y, screen_radius, Color::new(0.75, 0.32, 0.18, 1.0));
                    // Dark craters
                    draw_circle(p_screen.x - screen_radius * 0.35, p_screen.y + screen_radius * 0.2, screen_radius * 0.25, Color::new(0.4, 0.15, 0.1, 0.85));
                    draw_circle(p_screen.x + screen_radius * 0.25, p_screen.y - screen_radius * 0.3, screen_radius * 0.20, Color::new(0.35, 0.12, 0.08, 0.8));
                }
                PlanetType::IndustrialForge => {
                    // Volcanic charcoal sphere
                    draw_circle(p_screen.x, p_screen.y, screen_radius, Color::new(0.20, 0.18, 0.22, 1.0));
                    // Glowing molten lava / foundry rivers
                    draw_line(p_screen.x - screen_radius * 0.6, p_screen.y, p_screen.x + screen_radius * 0.6, p_screen.y, 2.0, Color::new(1.0, 0.45, 0.05, 0.9));
                    draw_circle(p_screen.x + screen_radius * 0.2, p_screen.y + screen_radius * 0.3, screen_radius * 0.25, Color::new(1.0, 0.6, 0.1, 0.7));
                }
                PlanetType::HighTechHub => {
                    // Cobalt core sphere
                    draw_circle(p_screen.x, p_screen.y, screen_radius, Color::new(0.10, 0.22, 0.45, 1.0));
                    // Cyber city night lights
                    draw_circle(p_screen.x - screen_radius * 0.25, p_screen.y, 1.8, Color::new(0.2, 0.95, 1.0, 0.95));
                    draw_circle(p_screen.x + screen_radius * 0.3, p_screen.y - screen_radius * 0.2, 1.8, Color::new(1.0, 0.85, 0.2, 0.95));
                    draw_circle(p_screen.x, p_screen.y + screen_radius * 0.35, 1.8, Color::new(0.2, 0.95, 1.0, 0.95));
                }
                PlanetType::FrontierOutpost => {
                    // Glacial cyan / ice sphere
                    draw_circle(p_screen.x, p_screen.y, screen_radius, Color::new(0.55, 0.80, 0.92, 1.0));
                    // Polar ice cap
                    draw_circle(p_screen.x, p_screen.y - screen_radius * 0.6, screen_radius * 0.4, Color::new(0.95, 0.98, 1.0, 0.95));
                }
            }

            // Spherical shadow highlight
            draw_circle(p_screen.x - screen_radius * 0.25, p_screen.y - screen_radius * 0.25, screen_radius * 0.45, Color::new(1.0, 1.0, 1.0, 0.20));

            // Front half of planetary rings
            if has_rings {
                let rx = screen_radius * 2.2;
                let ry = screen_radius * 0.55;
                draw_ellipse_lines(p_screen.x, p_screen.y, rx, ry, -15.0, 1.5, Color::new(0.85, 0.8, 0.7, 0.5));
            }

            // Orbital Battle VFX during siege
            if planet.planetary_defense < planet.max_defense * 0.95 {
                let time = get_time() as f32;
                let flash = ((time * 6.0).sin() * 0.5 + 0.5).abs();
                draw_circle_lines(p_screen.x, p_screen.y, screen_radius + 7.0, 2.0, Color::new(1.0, 0.2, 0.1, flash));

                // Spawn orbital laser beam arcs and explosion sparks!
                if ::rand::random::<f32>() < 0.08 {
                    let angle = ::rand::random::<f32>() * std::f32::consts::TAU;
                    let orbit_offset = Vec2::new(angle.cos(), angle.sin()) * (base_radius + 20.0);
                    let target_offset = Vec2::new(angle.cos(), angle.sin()) * (base_radius * 0.5);
                    particles.emit_laser_tracer(p_world + orbit_offset, p_world + target_offset, Color::new(1.0, 0.3, 0.2, 0.9));
                    particles.emit_explosion_spark(p_world + target_offset, Color::new(1.0, 0.7, 0.2, 1.0));
                }
            }

            // Labels
            if should_draw_names {
                draw_text(&planet.name, p_screen.x - 22.0, p_screen.y + screen_radius + 14.0, 12.0, WHITE);
            }

            // Selection ring
            if self.selected_planet == Some(planet.id) {
                let sel_pulse = ((get_time() * 4.0).sin() as f32 * 2.0).abs();
                draw_circle_lines(p_screen.x, p_screen.y, screen_radius + 8.0 + sel_pulse, 2.0, Color::new(0.0, 0.9, 1.0, 1.0));
            }
        }

        // 7. Draw Active Particle Systems in World Space
        particles.draw_world_space(|w_pos| camera.world_to_screen(w_pos), camera.zoom);

        // 8. Draw Modern Planet Inspector HUD
        if let Some(id) = self.selected_planet {
            if let Some(planet) = galaxy.get_planet(id) {
                self.draw_inspector_hud(planet, factions, items);
            }
        }
    }

    fn draw_inspector_hud(
        &self,
        planet: &crate::galaxy::Planet,
        factions: &FactionManager,
        items: &HashMap<String, ItemDef>,
    ) {
        let x = 20.0;
        let y = screen_height() - 275.0;
        let w = 410.0;
        let h = 255.0;

        let border_col = Color::new(0.0, 0.85, 1.0, 0.85);
        SciFiUi::draw_panel(x, y, w, h, None, border_col);

        draw_text(&format!("PLANET // {}", planet.name.to_uppercase()), x + 16.0, y + 25.0, 17.0, Color::new(0.3, 0.9, 1.0, 1.0));
        draw_text(&format!("Sector: {} | Population: {:.1}M", planet.zone, planet.population_millions), x + 16.0, y + 44.0, 12.0, LIGHTGRAY);

        // Close button [X]
        draw_rectangle(x + w - 30.0, y + 8.0, 22.0, 20.0, Color::new(0.2, 0.25, 0.35, 0.8));
        draw_rectangle_lines(x + w - 30.0, y + 8.0, 22.0, 20.0, 1.0, border_col);
        draw_text("X", x + w - 24.0, y + 23.0, 14.0, WHITE);

        // Allegiance Badge
        let allegiance_y = y + 72.0;
        if let Some(ref f_id) = planet.owner_faction {
            if let Some(faction) = factions.factions.get(f_id) {
                let c = faction.flag.primary_color;
                let col = Color::new(c[0], c[1], c[2], 1.0);
                
                draw_rectangle(x + 16.0, allegiance_y - 14.0, 150.0, 22.0, Color::new(c[0] * 0.25, c[1] * 0.25, c[2] * 0.25, 0.8));
                draw_rectangle_lines(x + 16.0, allegiance_y - 14.0, 150.0, 22.0, 1.0, col);
                draw_text(&faction.name, x + 24.0, allegiance_y + 2.0, 13.0, col);
                draw_text(&format!("Banner: {}", faction.flag.badge()), x + 175.0, allegiance_y + 2.0, 12.0, LIGHTGRAY);
            }
        } else {
            draw_rectangle(x + 16.0, allegiance_y - 14.0, 180.0, 22.0, Color::new(0.25, 0.25, 0.15, 0.8));
            draw_rectangle_lines(x + 16.0, allegiance_y - 14.0, 180.0, 22.0, 1.0, Color::new(1.0, 0.8, 0.2, 0.8));
            draw_text("FREE / INDEPENDENT WORLD", x + 24.0, allegiance_y + 2.0, 12.0, Color::new(1.0, 0.85, 0.3, 1.0));
        }

        // Planetary Defense Bar
        let def_ratio = (planet.planetary_defense / planet.max_defense).clamp(0.0, 1.0);
        let bar_y = y + 104.0;
        draw_text("Planetary Defense:", x + 16.0, bar_y + 11.0, 12.0, WHITE);
        draw_rectangle(x + 140.0, bar_y, 245.0, 14.0, Color::new(0.12, 0.15, 0.20, 1.0));
        let def_col = if def_ratio > 0.6 { Color::new(0.2, 0.85, 0.4, 1.0) } else if def_ratio > 0.3 { Color::new(1.0, 0.75, 0.2, 1.0) } else { Color::new(1.0, 0.3, 0.3, 1.0) };
        draw_rectangle(x + 140.0, bar_y, 245.0 * def_ratio, 14.0, def_col);
        draw_rectangle_lines(x + 140.0, bar_y, 245.0, 14.0, 1.0, DARKGRAY);
        draw_text(&format!("{:.0} / {:.0}", planet.planetary_defense, planet.max_defense), x + 235.0, bar_y + 11.0, 11.0, WHITE);

        // Local Commodity Exchange
        draw_line(x + 16.0, y + 130.0, x + w - 16.0, y + 130.0, 1.0, Color::new(0.2, 0.3, 0.4, 0.5));
        draw_text("COMMODITY EXCHANGE", x + 16.0, y + 147.0, 12.0, Color::new(0.0, 0.85, 1.0, 1.0));

        let commodities = ["titanium_ore", "iron_ore", "microelectronics", "medical_supplies", "fuel_cells"];
        let mut row_y = y + 168.0;
        for &c_id in &commodities {
            if let Some(item_def) = items.get(c_id) {
                let price = planet.market.get_price(c_id, item_def);
                let stock = planet.market.inventory.get(c_id).map(|i| i.current_stock).unwrap_or(0.0);
                let ratio = price / item_def.base_price;

                let (trend_tag, trend_col) = if ratio > 1.25 {
                    ("[DEFICIT]", Color::new(1.0, 0.35, 0.35, 1.0))
                } else if ratio < 0.75 {
                    ("[SURPLUS]", Color::new(0.3, 0.9, 0.4, 1.0))
                } else {
                    ("[FAIR]", GRAY)
                };

                draw_text(&item_def.name, x + 16.0, row_y, 11.0, LIGHTGRAY);
                draw_text(&format!("{:.1} cr", price), x + 175.0, row_y, 11.0, WHITE);
                draw_text(&format!("Stock: {:.0}", stock), x + 250.0, row_y, 11.0, Color::new(0.7, 0.8, 0.9, 0.9));
                draw_text(trend_tag, x + 340.0, row_y, 10.0, trend_col);

                row_y += 18.0;
            }
        }
    }
}
