use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleType {
    EngineExhaust,
    LaserTracer,
    ExplosionSpark,
    EvaGasPuff,
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub p_type: ParticleType,
    pub active: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            color: WHITE,
            size: 2.0,
            lifetime: 0.0,
            max_lifetime: 1.0,
            p_type: ParticleType::EngineExhaust,
            active: false,
        }
    }
}

pub struct ParticleSystem {
    pool: Vec<Particle>,
    next_idx: usize,
    capacity: usize,
}

impl ParticleSystem {
    pub fn new(capacity: usize) -> Self {
        let pool = vec![Particle::default(); capacity];
        Self {
            pool,
            next_idx: 0,
            capacity,
        }
    }

    /// Emits a single particle into the ring pool (Zero allocation!)
    pub fn emit(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        color: Color,
        size: f32,
        lifetime: f32,
        p_type: ParticleType,
    ) {
        let idx = self.next_idx;
        self.next_idx = (self.next_idx + 1) % self.capacity;

        let p = &mut self.pool[idx];
        p.position = position;
        p.velocity = velocity;
        p.color = color;
        p.size = size;
        p.lifetime = lifetime;
        p.max_lifetime = lifetime;
        p.p_type = p_type;
        p.active = true;
    }

    /// Emits an engine exhaust particle puff
    pub fn emit_engine_trail(&mut self, pos: Vec2, dir: Vec2, color: Color, speed: f32) {
        let spread = Vec2::new(
            (::rand::random::<f32>() - 0.5) * 8.0,
            (::rand::random::<f32>() - 0.5) * 8.0,
        );
        let vel = -dir * speed + spread;
        let size = ::rand::random::<f32>() * 2.0 + 2.5;
        let life = ::rand::random::<f32>() * 0.4 + 0.3;
        self.emit(pos, vel, color, size, life, ParticleType::EngineExhaust);
    }

    /// Emits an orbital battle laser beam / tracer
    pub fn emit_laser_tracer(&mut self, from: Vec2, to: Vec2, color: Color) {
        let vel = (to - from).normalize_or_zero() * 450.0;
        let dist = from.distance(to);
        let life = (dist / 450.0).clamp(0.1, 0.8);
        self.emit(from, vel, color, 2.5, life, ParticleType::LaserTracer);
    }

    /// Emits an explosion spark burst
    pub fn emit_explosion_spark(&mut self, pos: Vec2, color: Color) {
        for _ in 0..6 {
            let angle = ::rand::random::<f32>() * std::f32::consts::TAU;
            let speed = ::rand::random::<f32>() * 80.0 + 30.0;
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let life = ::rand::random::<f32>() * 0.5 + 0.3;
            let size = ::rand::random::<f32>() * 2.5 + 1.5;
            self.emit(pos, vel, color, size, life, ParticleType::ExplosionSpark);
        }
    }

    /// Emits EVA jetpack gas puff
    pub fn emit_eva_puff(&mut self, pos: Vec2, vel_offset: Vec2) {
        let spread = Vec2::new(
            (::rand::random::<f32>() - 0.5) * 12.0,
            (::rand::random::<f32>() - 0.5) * 12.0,
        );
        let vel = -vel_offset * 15.0 + spread;
        let life = ::rand::random::<f32>() * 0.35 + 0.2;
        self.emit(pos, vel, Color::new(0.3, 0.85, 1.0, 0.7), 3.0, life, ParticleType::EvaGasPuff);
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.pool.iter_mut() {
            if !p.active {
                continue;
            }
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                p.active = false;
            } else {
                p.position += p.velocity * dt;
                // Fade alpha proportionally to remaining lifetime
                let progress = p.lifetime / p.max_lifetime;
                match p.p_type {
                    ParticleType::EngineExhaust | ParticleType::EvaGasPuff => {
                        p.color.a = progress * 0.8;
                        p.size *= 0.98;
                    }
                    ParticleType::ExplosionSpark => {
                        p.color.a = progress;
                        p.velocity *= 0.92; // air / field resistance
                    }
                    ParticleType::LaserTracer => {
                        p.color.a = progress.clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    pub fn draw_world_space<F>(&self, world_to_screen: F, zoom: f32)
    where
        F: Fn(Vec2) -> Vec2,
    {
        let sw = screen_width();
        let sh = screen_height();

        for p in self.pool.iter() {
            if !p.active {
                continue;
            }

            let s_pos = world_to_screen(p.position);
            // Strict Frustum Culling!
            if s_pos.x < -20.0 || s_pos.x > sw + 20.0 || s_pos.y < -20.0 || s_pos.y > sh + 20.0 {
                continue;
            }

            let s_size = (p.size * zoom.clamp(0.5, 2.0)).max(1.0);

            match p.p_type {
                ParticleType::EngineExhaust | ParticleType::EvaGasPuff => {
                    draw_circle(s_pos.x, s_pos.y, s_size, p.color);
                }
                ParticleType::ExplosionSpark => {
                    draw_circle(s_pos.x, s_pos.y, s_size, p.color);
                    draw_circle(s_pos.x, s_pos.y, s_size * 0.5, WHITE);
                }
                ParticleType::LaserTracer => {
                    let trail = p.velocity.normalize_or_zero() * 12.0 * zoom.clamp(0.5, 1.5);
                    draw_line(s_pos.x, s_pos.y, s_pos.x - trail.x, s_pos.y - trail.y, s_size, p.color);
                }
            }
        }
    }
}
