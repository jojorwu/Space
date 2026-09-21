use macroquad::prelude::*;
use crate::render::ui::SciFiUi;

#[derive(Debug, Clone)]
pub struct GameSettings {
    pub sim_speed_idx: usize, // 0: 0.5x, 1: 1.0x, 2: 2.0x, 3: 4.0x
    pub planet_names_idx: usize, // 0: On Zoom, 1: Always, 2: Disabled
    pub show_trade_routes: bool,
    pub show_fleet_trails: bool,
    pub master_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sim_speed_idx: 1, // 1.0x normal
            planet_names_idx: 0, // on zoom
            show_trade_routes: true,
            show_fleet_trails: true,
            master_volume: 0.8,
        }
    }
}

impl GameSettings {
    pub fn speed_multiplier(&self) -> f32 {
        match self.sim_speed_idx {
            0 => 0.5,
            1 => 1.0,
            2 => 2.0,
            3 => 4.0,
            _ => 1.0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MainMenuAction {
    StartGame,
    OpenSettings,
    ExitGame,
    None,
}

pub struct MenuRenderer;

impl MenuRenderer {
    pub fn draw_main_menu() -> MainMenuAction {
        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

        let sw = screen_width();
        let sh = screen_height();
        let time = get_time() as f32;

        // 1. Cosmic Background Animation
        for i in 0..120 {
            let sx = (i * 137) as f32 % sw;
            let sy = (i * 229) as f32 % sh;
            let twinkle = (time * 2.0 + i as f32 * 0.7).sin() * 0.3 + 0.7;
            draw_circle(sx, sy, 1.3, Color::new(0.7, 0.8, 1.0, 0.4 * twinkle));
        }

        // Rotating Orbital Rings in Background Center
        let cx = sw * 0.5;
        let cy = sh * 0.45;
        let ring_r = 220.0;
        let ring_pulse = (time * 0.8).sin() * 15.0;
        draw_circle_lines(cx, cy, ring_r + ring_pulse, 1.5, Color::new(0.15, 0.45, 0.8, 0.25));
        draw_circle_lines(cx, cy, ring_r * 0.65 - ring_pulse * 0.5, 1.0, Color::new(0.0, 0.8, 0.9, 0.20));

        // 2. Title & Subtitle Card
        let title = "PROJECT STAR-CORE";
        let font_title = 42;
        let tw = measure_text(title, None, font_title, 1.0).width;
        let glow = (time * 3.0).sin() * 0.15 + 0.85;

        // Title glow effect
        draw_text(title, cx - tw * 0.5 - 1.0, cy - 110.0, font_title as f32, Color::new(0.0, 0.85, 1.0, 0.4 * glow));
        draw_text(title, cx - tw * 0.5, cy - 110.0, font_title as f32, Color::new(0.9, 0.96, 1.0, 1.0));

        let subtitle = "2D LIVING GALAXY SIMULATION | RUST & LUA CORE";
        let font_sub = 14;
        let stw = measure_text(subtitle, None, font_sub, 1.0).width;
        draw_text(subtitle, cx - stw * 0.5, cy - 80.0, font_sub as f32, Color::new(0.2, 0.8, 0.5, 0.85));

        // 3. Menu Buttons
        let btn_w = 320.0;
        let btn_h = 44.0;
        let btn_x = cx - btn_w * 0.5;
        let btn_y_start = cy - 20.0;
        let spacing = 58.0;

        let mut action = MainMenuAction::None;

        if SciFiUi::button(btn_x, btn_y_start, btn_w, btn_h, "START SIMULATION", Color::new(0.0, 0.85, 1.0, 1.0)) {
            action = MainMenuAction::StartGame;
        }

        if SciFiUi::button(btn_x, btn_y_start + spacing, btn_w, btn_h, "SETTINGS & CONFIG", Color::new(1.0, 0.75, 0.15, 1.0)) {
            action = MainMenuAction::OpenSettings;
        }

        if SciFiUi::button(btn_x, btn_y_start + spacing * 2.0, btn_w, btn_h, "EXIT TO DESKTOP", Color::new(0.9, 0.3, 0.35, 1.0)) {
            action = MainMenuAction::ExitGame;
        }

        // 4. Footer Information
        let footer = "v0.3.0 | 100 Procedural Planets | Agent-based Economy | Block Ships & EVA";
        let ftw = measure_text(footer, None, 12, 1.0).width;
        draw_text(footer, cx - ftw * 0.5, sh - 25.0, 12.0, Color::new(0.5, 0.6, 0.7, 0.6));

        action
    }

    pub fn draw_settings_modal(settings: &mut GameSettings) -> bool {
        let sw = screen_width();
        let sh = screen_height();

        // Dark backdrop overlay
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.02, 0.03, 0.05, 0.85));

        let modal_w = 480.0;
        let modal_h = 430.0;
        let modal_x = (sw - modal_w) * 0.5;
        let modal_y = (sh - modal_h) * 0.5;

        SciFiUi::draw_panel(
            modal_x,
            modal_y,
            modal_w,
            modal_h,
            Some("SIMULATION & DISPLAY CONFIGURATION"),
            Color::new(0.0, 0.85, 1.0, 0.85),
        );

        let content_x = modal_x + 30.0;
        let content_w = modal_w - 60.0;
        let mut cur_y = modal_y + 55.0;

        // 1. Sim Speed Selector
        let speeds = ["0.5x", "1.0x (Norm)", "2.0x (Fast)", "4.0x (Turbo)"];
        SciFiUi::segment_selector(
            content_x,
            cur_y,
            content_w,
            55.0,
            "Simulation Clock Speed:",
            &speeds,
            &mut settings.sim_speed_idx,
            Color::new(0.0, 0.85, 1.0, 1.0),
        );
        cur_y += 68.0;

        // 2. Planet Labels Mode
        let label_modes = ["On Zoom", "Always On", "Hidden"];
        SciFiUi::segment_selector(
            content_x,
            cur_y,
            content_w,
            55.0,
            "Planet Name Tags:",
            &label_modes,
            &mut settings.planet_names_idx,
            Color::new(0.2, 0.8, 0.5, 1.0),
        );
        cur_y += 68.0;

        // 3. Trade Route Overlays Toggle
        SciFiUi::toggle(
            content_x,
            cur_y,
            content_w,
            "Show Merchant Trade Lines:",
            &mut settings.show_trade_routes,
            Color::new(0.2, 0.9, 0.4, 1.0),
        );
        cur_y += 40.0;

        // 4. Fleet Trails Toggle
        SciFiUi::toggle(
            content_x,
            cur_y,
            content_w,
            "Show War Armada & Colony Trails:",
            &mut settings.show_fleet_trails,
            Color::new(1.0, 0.35, 0.35, 1.0),
        );
        cur_y += 42.0;

        // 5. Volume Slider
        SciFiUi::slider(
            content_x,
            cur_y,
            content_w,
            30.0,
            "Audio & Effects Master Level:",
            &mut settings.master_volume,
            Color::new(1.0, 0.75, 0.2, 1.0),
        );
        cur_y += 50.0;

        // Apply & Back Button
        SciFiUi::button(
            content_x + 40.0,
            cur_y,
            content_w - 80.0,
            38.0,
            "APPLY & RETURN",
            Color::new(0.0, 0.85, 1.0, 1.0),
        )
    }
}
