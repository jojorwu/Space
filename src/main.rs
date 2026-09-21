use std::path::Path;
use glam::Vec2 as GlamVec2;
use macroquad::prelude::*;
use ::rand::{rngs::StdRng, SeedableRng};

use star_core::factions::FactionManager;
use star_core::galaxy::Galaxy;
use star_core::render::camera::PanZoomCamera;
use star_core::render::galaxy_view::GalaxyRenderer;
use star_core::render::menu::{GameSettings, MainMenuAction, MenuRenderer};
use star_core::render::particles::ParticleSystem;
use star_core::render::ship_view::ShipRenderer;
use star_core::render::ui::SciFiUi;
use star_core::scripting::ScriptEngine;
use star_core::ship::{Character, ShipGrid};
use star_core::sim::world::GalacticWorld;
use star_core::traders::TradeFleet;

#[derive(Debug, PartialEq, Eq)]
enum ViewMode {
    Galaxy,
    Ship,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AppState {
    MainMenu,
    InGame,
    Settings(Box<AppState>),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Project Star-Core: 2D Living Galaxy Simulation".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut rng = StdRng::seed_from_u64(42);

    // 1. Settings & State
    let mut settings = GameSettings::default();
    let mut app_state = AppState::MainMenu;

    // 2. Initialize Lua Scripting & Load Assets
    let script_engine = ScriptEngine::new();
    let items = script_engine
        .load_items(Path::new("scripts/items.lua"))
        .expect("Failed to load scripts/items.lua");
    let (_zones, _legacy_factions) = script_engine
        .load_factions_and_zones(Path::new("scripts/factions.lua"))
        .expect("Failed to load scripts/factions.lua");
    let blocks = script_engine
        .load_blocks(Path::new("scripts/blocks.lua"))
        .expect("Failed to load scripts/blocks.lua");
    let personalities = script_engine
        .load_personalities(Path::new("scripts/ai_personalities.lua"))
        .unwrap_or_default();

    // 3. Procedural Galaxy Generation: 100 planets across 3 zones
    let mut galaxy = Galaxy::generate_procedural(&mut rng, &items);

    // 4. Procedural Factions, Flags, and Heraldry
    let mut faction_mgr = FactionManager::new();
    faction_mgr.generate_procedural(&mut rng, 4, &mut galaxy);

    // 5. Autonomous NPC Trader Fleets
    let mut trade_fleet = TradeFleet::new();
    trade_fleet.spawn_trader(1, "Atlas Hauler #01", 0, 15000.0, 80.0, 35.0);
    trade_fleet.spawn_trader(2, "Starlight Freighter", 15, 25000.0, 120.0, 28.0);
    trade_fleet.spawn_trader(3, "Rim Runner", 50, 8000.0, 40.0, 45.0);
    trade_fleet.spawn_trader(4, "Frontier Express", 85, 12000.0, 60.0, 38.0);

    // 6. Encapsulated Simulation World (with Spatial Index & Event Bus)
    let mut world = GalacticWorld::new(
        galaxy,
        faction_mgr,
        personalities,
        trade_fleet,
        items,
    );

    // 7. Player's Modular Block Ship
    let mut player_ship = ShipGrid::new("Aegis-One", GlamVec2::new(50.0, 50.0));
    player_ship.place_block(0, 2, blocks.get("cockpit").unwrap());
    player_ship.place_block(0, 1, blocks.get("corridor").unwrap());
    player_ship.place_block(0, 0, blocks.get("fusion_reactor").unwrap());
    player_ship.place_block(-1, 0, blocks.get("ion_thruster").unwrap());
    player_ship.place_block(1, 0, blocks.get("ion_thruster").unwrap());
    player_ship.place_block(1, 1, blocks.get("cargo_bay").unwrap());
    player_ship.place_block(0, -1, blocks.get("airlock").unwrap());

    // 8. Character
    let mut character = Character {
        name: "Captain Alex".to_string(),
        in_ship: true,
        local_pos: GlamVec2::new(0.0, 1.0),
        world_pos: GlamVec2::new(50.0, 50.0),
        velocity: GlamVec2::ZERO,
        oxygen: 100.0,
        in_eva_suit: true,
    };

    // 9. Renderers & Camera
    let mut camera = PanZoomCamera::new(Vec2::ZERO, 0.45);
    let mut galaxy_renderer = GalaxyRenderer::new();
    let mut ship_renderer = ShipRenderer::new();
    let mut particles = ParticleSystem::new(800);

    let mut view_mode = ViewMode::Galaxy;
    let mut is_paused = false;
    let mut cycle_timer = 0.0f32;

    loop {
        let dt = get_frame_time();
        particles.update(dt);

        match &app_state {
            AppState::MainMenu => {
                match MenuRenderer::draw_main_menu() {
                    MainMenuAction::StartGame => {
                        app_state = AppState::InGame;
                    }
                    MainMenuAction::OpenSettings => {
                        app_state = AppState::Settings(Box::new(AppState::MainMenu));
                    }
                    MainMenuAction::ExitGame => {
                        std::process::exit(0);
                    }
                    MainMenuAction::None => {}
                }
            }
            AppState::Settings(prev_state) => {
                // If InGame, draw the background world in paused state
                if **prev_state == AppState::InGame {
                    match view_mode {
                        ViewMode::Galaxy => {
                            galaxy_renderer.draw(&camera, &world.galaxy, &world.factions, &world.traders, &world.items, &settings, &mut particles);
                        }
                        ViewMode::Ship => {
                            ship_renderer.draw(&player_ship, &character);
                        }
                    }
                }

                let close_clicked = MenuRenderer::draw_settings_modal(&mut settings);
                if close_clicked || is_key_pressed(KeyCode::Escape) {
                    app_state = (**prev_state).clone();
                }
            }
            AppState::InGame => {
                // Hotkeys
                if is_key_pressed(KeyCode::Escape) {
                    app_state = AppState::Settings(Box::new(AppState::InGame));
                }

                if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Key1) && view_mode != ViewMode::Galaxy || is_key_pressed(KeyCode::Key2) && view_mode != ViewMode::Ship {
                    view_mode = match view_mode {
                        ViewMode::Galaxy => ViewMode::Ship,
                        ViewMode::Ship => ViewMode::Galaxy,
                    };
                }

                if is_key_pressed(KeyCode::Space) {
                    is_paused = !is_paused;
                }

                // Simulation cycle tick (adjusted by speed multiplier!)
                if !is_paused {
                    cycle_timer += dt * settings.speed_multiplier();
                    if cycle_timer >= 1.2 {
                        cycle_timer = 0.0;
                        world.tick(&mut rng);
                    }
                }

                // Render current view
                match view_mode {
                    ViewMode::Galaxy => {
                        camera.update();
                        galaxy_renderer.handle_click(&camera, &world.galaxy);
                        galaxy_renderer.draw(
                            &camera,
                            &world.galaxy,
                            &world.factions,
                            &world.traders,
                            &world.items,
                            &settings,
                            &mut particles,
                        );
                    }
                    ViewMode::Ship => {
                        ship_renderer.update_character_input(&mut player_ship, &mut character, dt, &mut particles);
                        ship_renderer.draw(&player_ship, &character);
                    }
                }

                // Top Glassmorphism Status Bar with Settings button
                let open_settings = draw_top_bar(
                    view_mode == ViewMode::Galaxy,
                    is_paused,
                    world.cycle_count,
                    settings.speed_multiplier(),
                    &world.factions,
                    &world.galaxy,
                );
                if open_settings {
                    app_state = AppState::Settings(Box::new(AppState::InGame));
                }

                // Bottom Navigation Dock
                draw_bottom_dock(&mut view_mode, &mut is_paused);

                // Recent Events Ticker HUD (top right)
                let recent_logs = world.event_bus.recent_strings(5);
                draw_event_ticker(&recent_logs);
            }
        }

        next_frame().await;
    }
}

fn draw_top_bar(
    is_galaxy_view: bool,
    is_paused: bool,
    cycle: usize,
    speed_mult: f32,
    factions: &FactionManager,
    galaxy: &Galaxy,
) -> bool {
    let bar_h = 44.0;
    let sw = screen_width();

    // Dark glass bar
    draw_rectangle(0.0, 0.0, sw, bar_h, Color::new(0.04, 0.06, 0.10, 0.95));
    draw_line(0.0, bar_h, sw, bar_h, 1.5, Color::new(0.0, 0.85, 1.0, 0.7));

    // View badge pill
    let view_text = if is_galaxy_view { "MAP // GALAXY" } else { "SHIP // INTERIOR & EVA" };
    let view_col = if is_galaxy_view { Color::new(0.0, 0.85, 1.0, 1.0) } else { Color::new(1.0, 0.75, 0.2, 1.0) };
    draw_rectangle(15.0, 8.0, 180.0, 28.0, Color::new(view_col.r * 0.2, view_col.g * 0.2, view_col.b * 0.2, 0.8));
    draw_rectangle_lines(15.0, 8.0, 180.0, 28.0, 1.0, view_col);
    draw_text(view_text, 28.0, 27.0, 13.0, view_col);

    // Status & Speed Pill
    let (status_str, status_col) = if is_paused {
        ("|| PAUSED", YELLOW)
    } else {
        (">> RUNNING", Color::new(0.2, 0.9, 0.4, 1.0))
    };
    draw_text(status_str, 215.0, 27.0, 13.0, status_col);
    draw_text(&format!("({:.1}x)", speed_mult), 305.0, 27.0, 12.0, LIGHTGRAY);

    // Stats
    let active_wars = factions.diplomacy.relations.values().filter(|r| r.state == star_core::factions::DiplomaticState::War).count();
    let unaligned = galaxy.planets.iter().filter(|p| p.owner_faction.is_none()).count();
    draw_text(
        &format!("CYCLE: {:03}  |  WARS: {}  |  FREE WORLDS: {}  |  FLEETS: {}", cycle, active_wars, unaligned, factions.active_fleets.len()),
        360.0, 27.0, 12.0, Color::new(0.85, 0.9, 0.95, 0.9),
    );

    // Settings Button [⚙ SETTINGS] in top right
    let btn_w = 110.0;
    let btn_h = 28.0;
    let btn_x = sw - btn_w - 15.0;
    SciFiUi::button(btn_x, 8.0, btn_w, btn_h, "⚙ SETTINGS", Color::new(0.0, 0.85, 1.0, 1.0))
}

fn draw_bottom_dock(view_mode: &mut ViewMode, is_paused: &mut bool) {
    let sw = screen_width();
    let sh = screen_height();

    let dock_w = 520.0;
    let dock_h = 42.0;
    let dock_x = (sw - dock_w) * 0.5;
    let dock_y = sh - dock_h - 12.0;

    // Glass panel for bottom dock
    draw_rectangle(dock_x, dock_y, dock_w, dock_h, Color::new(0.04, 0.06, 0.10, 0.92));
    draw_rectangle_lines(dock_x, dock_y, dock_w, dock_h, 1.2, Color::new(0.0, 0.85, 1.0, 0.5));

    let btn_w = 120.0;
    let btn_h = 30.0;
    let btn_y = dock_y + 6.0;

    // 1. Galaxy Map
    let map_accent = if *view_mode == ViewMode::Galaxy { Color::new(0.0, 0.85, 1.0, 1.0) } else { DARKGRAY };
    if SciFiUi::button(dock_x + 10.0, btn_y, btn_w, btn_h, "[1] GALAXY", map_accent) {
        *view_mode = ViewMode::Galaxy;
    }

    // 2. Ship View
    let ship_accent = if *view_mode == ViewMode::Ship { Color::new(1.0, 0.75, 0.2, 1.0) } else { DARKGRAY };
    if SciFiUi::button(dock_x + 138.0, btn_y, btn_w, btn_h, "[2] SHIP & EVA", ship_accent) {
        *view_mode = ViewMode::Ship;
    }

    // 3. Pause / Play
    let (pause_label, pause_col) = if *is_paused {
        ("[SPACE] RESUME", Color::new(0.2, 0.9, 0.4, 1.0))
    } else {
        ("[SPACE] PAUSE", Color::new(1.0, 0.75, 0.2, 1.0))
    };
    if SciFiUi::button(dock_x + 266.0, btn_y, btn_w, btn_h, pause_label, pause_col) {
        *is_paused = !*is_paused;
    }

    // 4. Controls Hint
    let hint_text = "[TAB] Switch | [R-Mouse] Pan | [Wheel] Zoom";
    draw_text(hint_text, dock_x + 395.0, dock_y + 26.0, 11.0, GRAY);
}

fn draw_event_ticker(events: &[String]) {
    let start_y = 54.0;
    let x = screen_width() - 475.0;
    let w = 460.0;
    let h = (events.len() as f32 * 19.0) + 12.0;

    if !events.is_empty() {
        draw_rectangle(x, start_y, w, h, Color::new(0.03, 0.05, 0.09, 0.82));
        draw_rectangle_lines(x, start_y, w, h, 1.0, Color::new(0.0, 0.85, 1.0, 0.4));

        let mut row_y = start_y + 16.0;
        for ev in events {
            let col = if ev.contains("WAR") {
                Color::new(1.0, 0.35, 0.35, 1.0)
            } else if ev.contains("CONQUEST") {
                Color::new(1.0, 0.55, 0.2, 1.0)
            } else if ev.contains("INTERCEPTION") {
                Color::new(1.0, 0.85, 0.2, 1.0)
            } else if ev.contains("COLONIZED") || ev.contains("CLAIMED") {
                Color::new(0.0, 0.85, 1.0, 1.0)
            } else if ev.contains("PEACE") {
                Color::new(0.3, 0.95, 0.45, 1.0)
            } else {
                LIGHTGRAY
            };
            draw_text(ev, x + 12.0, row_y, 11.0, col);
            row_y += 19.0;
        }
    }
}
