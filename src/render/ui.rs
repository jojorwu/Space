use macroquad::prelude::*;

pub struct SciFiUi;

impl SciFiUi {
    /// Renders a sci-fi panel with a glassmorphism dark background and neon border
    pub fn draw_panel(x: f32, y: f32, w: f32, h: f32, title: Option<&str>, border_color: Color) {
        // Deep glass background
        draw_rectangle(x, y, w, h, Color::new(0.04, 0.06, 0.10, 0.92));
        
        // Double frame: outer subtle, inner bright
        draw_rectangle_lines(x - 2.0, y - 2.0, w + 4.0, h + 4.0, 1.0, Color::new(border_color.r, border_color.g, border_color.b, 0.25));
        draw_rectangle_lines(x, y, w, h, 1.5, border_color);

        // Corner accents
        let c_len = 10.0;
        draw_line(x, y, x + c_len, y, 3.0, border_color);
        draw_line(x, y, x, y + c_len, 3.0, border_color);
        draw_line(x + w, y, x + w - c_len, y, 3.0, border_color);
        draw_line(x + w, y, x + w, y + c_len, 3.0, border_color);
        draw_line(x, y + h, x + c_len, y + h, 3.0, border_color);
        draw_line(x, y + h, x, y + h - c_len, 3.0, border_color);
        draw_line(x + w, y + h, x + w - c_len, y + h, 3.0, border_color);
        draw_line(x + w, y + h, x + w, y + h - c_len, 3.0, border_color);

        // Header bar if title provided
        if let Some(t) = title {
            let header_h = 30.0;
            draw_rectangle(x + 1.0, y + 1.0, w - 2.0, header_h, Color::new(border_color.r * 0.2, border_color.g * 0.2, border_color.b * 0.2, 0.7));
            draw_line(x, y + header_h, x + w, y + header_h, 1.5, border_color);
            draw_text(t, x + 14.0, y + 21.0, 16.0, border_color);
        }
    }

    /// Interactive sci-fi button with hover glow and click detection
    pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, accent_color: Color) -> bool {
        let mouse = Vec2::new(mouse_position().0, mouse_position().1);
        let is_hover = mouse.x >= x && mouse.x <= x + w && mouse.y >= y && mouse.y <= y + h;
        let is_clicked = is_hover && is_mouse_button_pressed(MouseButton::Left);

        let (bg_col, border_col, text_col) = if is_clicked {
            (
                Color::new(accent_color.r, accent_color.g, accent_color.b, 0.6),
                WHITE,
                WHITE,
            )
        } else if is_hover {
            (
                Color::new(accent_color.r * 0.25, accent_color.g * 0.25, accent_color.b * 0.25, 0.85),
                Color::new(accent_color.r * 1.2, accent_color.g * 1.2, accent_color.b * 1.2, 1.0),
                WHITE,
            )
        } else {
            (
                Color::new(0.06, 0.08, 0.13, 0.85),
                Color::new(accent_color.r * 0.7, accent_color.g * 0.7, accent_color.b * 0.7, 0.6),
                Color::new(0.85, 0.90, 0.95, 0.9),
            )
        };

        // Draw button body
        draw_rectangle(x, y, w, h, bg_col);
        draw_rectangle_lines(x, y, w, h, if is_hover { 2.0 } else { 1.2 }, border_col);

        // Chamfered left indicator bar
        draw_rectangle(x, y, 4.0, h, if is_hover { accent_color } else { Color::new(accent_color.r, accent_color.g, accent_color.b, 0.4) });

        // Center text
        let font_size = 15;
        let tw = measure_text(text, None, font_size, 1.0).width;
        let text_x = x + (w - tw) * 0.5;
        let text_y = y + h * 0.62;
        draw_text(text, text_x, text_y, font_size as f32, text_col);

        is_clicked
    }

    /// Interactive slider component for values 0.0 to 1.0
    pub fn slider(x: f32, y: f32, w: f32, h: f32, label: &str, value: &mut f32, accent: Color) {
        let mouse = Vec2::new(mouse_position().0, mouse_position().1);
        let track_y = y + h * 0.6;
        let track_h = 6.0;

        // Label and value percentage
        draw_text(label, x, y + 14.0, 13.0, LIGHTGRAY);
        let val_pct = format!("{:.0}%", *value * 100.0);
        let tw = measure_text(&val_pct, None, 13, 1.0).width;
        draw_text(&val_pct, x + w - tw, y + 14.0, 13.0, accent);

        // Track
        draw_rectangle(x, track_y, w, track_h, Color::new(0.15, 0.18, 0.25, 0.9));
        draw_rectangle(x, track_y, w * *value, track_h, accent);

        // Thumb / Handle
        let handle_x = x + w * *value;
        let handle_r = 8.0;
        let is_hover = mouse.distance(Vec2::new(handle_x, track_y + track_h * 0.5)) <= handle_r + 4.0
            || (mouse.x >= x && mouse.x <= x + w && (mouse.y - track_y).abs() <= 12.0);

        if is_hover && is_mouse_button_down(MouseButton::Left) {
            *value = ((mouse.x - x) / w).clamp(0.0, 1.0);
        }

        draw_circle(handle_x, track_y + track_h * 0.5, handle_r, if is_hover { WHITE } else { accent });
        draw_circle_lines(handle_x, track_y + track_h * 0.5, handle_r, 1.5, DARKGRAY);
    }

    /// Sleek segmented option selector (e.g. [0.5x] [1x] [2x] [4x])
    pub fn segment_selector(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        options: &[&str],
        selected_index: &mut usize,
        accent: Color,
    ) {
        draw_text(label, x, y + 14.0, 13.0, LIGHTGRAY);

        let seg_y = y + 22.0;
        let seg_w = (w - (options.len() as f32 - 1.0) * 4.0) / options.len() as f32;

        for (idx, opt) in options.iter().enumerate() {
            let opt_x = x + (idx as f32) * (seg_w + 4.0);
            let is_selected = *selected_index == idx;

            let button_accent = if is_selected { accent } else { Color::new(0.3, 0.4, 0.5, 0.5) };
            if Self::button(opt_x, seg_y, seg_w, h - 22.0, opt, button_accent) {
                *selected_index = idx;
            }
        }
    }

    /// Toggle switch
    pub fn toggle(x: f32, y: f32, w: f32, label: &str, active: &mut bool, accent: Color) {
        draw_text(label, x, y + 16.0, 14.0, WHITE);

        let toggle_w = 48.0;
        let toggle_h = 22.0;
        let toggle_x = x + w - toggle_w;

        let mouse = Vec2::new(mouse_position().0, mouse_position().1);
        let is_hover = mouse.x >= toggle_x && mouse.x <= toggle_x + toggle_w && mouse.y >= y && mouse.y <= y + toggle_h;

        if is_hover && is_mouse_button_pressed(MouseButton::Left) {
            *active = !*active;
        }

        // Pill background
        let bg_col = if *active {
            Color::new(accent.r * 0.4, accent.g * 0.4, accent.b * 0.4, 0.8)
        } else {
            Color::new(0.15, 0.18, 0.24, 0.8)
        };
        draw_rectangle(toggle_x, y, toggle_w, toggle_h, bg_col);
        draw_rectangle_lines(toggle_x, y, toggle_w, toggle_h, 1.2, if *active { accent } else { DARKGRAY });

        // Thumb circle
        let thumb_x = if *active { toggle_x + toggle_w - 12.0 } else { toggle_x + 12.0 };
        draw_circle(thumb_x, y + toggle_h * 0.5, 8.0, if *active { accent } else { LIGHTGRAY });
    }
}
