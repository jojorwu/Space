use macroquad::prelude::*;

pub struct PanZoomCamera {
    pub position: Vec2,
    pub zoom: f32,
    pub dragging: bool,
    pub last_mouse: Vec2,
}

impl PanZoomCamera {
    pub fn new(initial_pos: Vec2, initial_zoom: f32) -> Self {
        Self {
            position: initial_pos,
            zoom: initial_zoom,
            dragging: false,
            last_mouse: Vec2::ZERO,
        }
    }

    pub fn update(&mut self) {
        let mouse = Vec2::new(mouse_position().0, mouse_position().1);

        // Right mouse click or middle click to drag / pan
        if is_mouse_button_pressed(MouseButton::Right) || is_mouse_button_pressed(MouseButton::Middle) {
            self.dragging = true;
            self.last_mouse = mouse;
        }

        if is_mouse_button_released(MouseButton::Right) || is_mouse_button_released(MouseButton::Middle) {
            self.dragging = false;
        }

        if self.dragging {
            let delta = mouse - self.last_mouse;
            self.position -= delta / self.zoom;
            self.last_mouse = mouse;
        }

        // Mouse wheel zooming
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            let world_before = self.screen_to_world(mouse);
            let zoom_factor = if wheel > 0.0 { 1.15 } else { 0.87 };
            self.zoom = (self.zoom * zoom_factor).clamp(0.15, 6.0);
            let world_after = self.screen_to_world(mouse);
            self.position += world_before - world_after;
        }

        // Keyboard panning (Arrow keys or WASD when in Galaxy mode)
        let speed = 600.0 / self.zoom * get_frame_time();
        if is_key_down(KeyCode::Up) {
            self.position.y -= speed;
        }
        if is_key_down(KeyCode::Down) {
            self.position.y += speed;
        }
        if is_key_down(KeyCode::Left) {
            self.position.x -= speed;
        }
        if is_key_down(KeyCode::Right) {
            self.position.x += speed;
        }
    }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
        (world_pos - self.position) * self.zoom + screen_center
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
        (screen_pos - screen_center) / self.zoom + self.position
    }
}
