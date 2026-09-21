use std::collections::{HashMap, HashSet, VecDeque};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::scripting::BlockDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInstance {
    pub def_id: String,
    pub current_hp: f32,
    pub max_hp: f32,
    pub mass: f32,
    pub walkable: bool,
    pub sealed: bool,
    pub is_eva_exit: bool,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub in_ship: bool,
    pub local_pos: Vec2,  // Grid units when inside ship
    pub world_pos: Vec2,  // Space coordinates when in EVA
    pub velocity: Vec2,
    pub oxygen: f32,
    pub in_eva_suit: bool,
}

pub struct ShipGrid {
    pub name: String,
    pub blocks: HashMap<(i32, i32), BlockInstance>,
    pub world_position: Vec2,
    pub rotation: f32,      // in radians
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub center_of_mass: Vec2,
    pub moment_of_inertia: f32,
    pub total_mass: f32,
}

impl ShipGrid {
    pub fn new(name: &str, world_position: Vec2) -> Self {
        Self {
            name: name.to_string(),
            blocks: HashMap::new(),
            world_position,
            rotation: 0.0,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            center_of_mass: Vec2::ZERO,
            moment_of_inertia: 1.0,
            total_mass: 0.0,
        }
    }

    /// Place a block onto the ship grid
    pub fn place_block(&mut self, x: i32, y: i32, def: &BlockDef) {
        self.blocks.insert(
            (x, y),
            BlockInstance {
                def_id: def.id.clone(),
                current_hp: def.max_hp,
                max_hp: def.max_hp,
                mass: def.mass,
                walkable: def.walkable,
                sealed: def.sealed,
                is_eva_exit: def.is_eva_exit,
            },
        );
        self.recalculate_physics();
    }

    /// Damage or destroy a block at (x, y)
    pub fn damage_block(&mut self, x: i32, y: i32, damage: f32) -> bool {
        let mut destroyed = false;
        if let Some(block) = self.blocks.get_mut(&(x, y)) {
            block.current_hp -= damage;
            if block.current_hp <= 0.0 {
                destroyed = true;
            }
        }
        if destroyed {
            self.blocks.remove(&(x, y));
            self.recalculate_physics();
        }
        destroyed
    }

    /// Recalculate Center of Mass (CoM), Total Mass, and Moment of Inertia (I)
    pub fn recalculate_physics(&mut self) {
        let mut total_mass = 0.0f32;
        let mut weighted_pos = Vec2::ZERO;

        for (&(x, y), block) in self.blocks.iter() {
            total_mass += block.mass;
            weighted_pos += Vec2::new(x as f32, y as f32) * block.mass;
        }

        if total_mass <= 0.0 {
            self.total_mass = 0.0;
            self.center_of_mass = Vec2::ZERO;
            self.moment_of_inertia = 1.0;
            return;
        }

        self.total_mass = total_mass;
        self.center_of_mass = weighted_pos / total_mass;

        // Moment of Inertia: sum(m_i * r_i^2)
        let mut inertia = 0.0f32;
        for (&(x, y), block) in self.blocks.iter() {
            let r = Vec2::new(x as f32, y as f32) - self.center_of_mass;
            inertia += block.mass * r.length_squared();
        }

        self.moment_of_inertia = inertia.max(10.0);
    }

    /// Check structural connectivity via BFS.
    /// If disconnected groups are found (e.g. ship cut in two), returns detached chunks.
    pub fn split_disconnected_chunks(&mut self, core_block_pos: (i32, i32)) -> Vec<ShipGrid> {
        if self.blocks.is_empty() {
            return Vec::new();
        }

        // 1. BFS to find all blocks connected to core
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        let start_pos = if self.blocks.contains_key(&core_block_pos) {
            core_block_pos
        } else if let Some(&pos) = self.blocks.keys().next() {
            pos
        } else {
            return Vec::new();
        };

        queue.push_back(start_pos);
        visited.insert(start_pos);

        let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        while let Some((cx, cy)) = queue.pop_front() {
            for (dx, dy) in neighbors.iter() {
                let next_pos = (cx + dx, cy + dy);
                if self.blocks.contains_key(&next_pos) && !visited.contains(&next_pos) {
                    visited.insert(next_pos);
                    queue.push_back(next_pos);
                }
            }
        }

        // 2. Separate all non-visited blocks into detached debris chunks
        let all_positions: Vec<(i32, i32)> = self.blocks.keys().copied().collect();
        let mut detached_positions = Vec::new();

        for pos in all_positions {
            if !visited.contains(&pos) {
                detached_positions.push(pos);
            }
        }

        if detached_positions.is_empty() {
            return Vec::new();
        }

        // Group detached blocks into connected clusters
        let mut detached_chunks = Vec::new();
        let mut cluster_visited = HashSet::new();

        for &d_pos in &detached_positions {
            if cluster_visited.contains(&d_pos) {
                continue;
            }

            let mut cluster = Vec::new();
            let mut c_queue = VecDeque::new();
            c_queue.push_back(d_pos);
            cluster_visited.insert(d_pos);

            while let Some(cur) = c_queue.pop_front() {
                cluster.push(cur);
                for (dx, dy) in neighbors.iter() {
                    let next_pos = (cur.0 + dx, cur.1 + dy);
                    if self.blocks.contains_key(&next_pos)
                        && !visited.contains(&next_pos)
                        && !cluster_visited.contains(&next_pos)
                    {
                        cluster_visited.insert(next_pos);
                        c_queue.push_back(next_pos);
                    }
                }
            }

            // Create new debris chunk
            let mut chunk_ship = ShipGrid::new(
                &format!("{}_Debris", self.name),
                self.world_position,
            );
            chunk_ship.rotation = self.rotation;
            chunk_ship.velocity = self.velocity;

            for pos in cluster {
                if let Some(block) = self.blocks.remove(&pos) {
                    chunk_ship.blocks.insert(pos, block);
                }
            }
            chunk_ship.recalculate_physics();
            detached_chunks.push(chunk_ship);
        }

        self.recalculate_physics();
        detached_chunks
    }

    /// Physics step integration: update linear position and rotation based on velocities & inertia
    pub fn update_physics(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        // Integrate linear position
        self.world_position += self.velocity * dt;

        // Integrate rotation and apply angular damping based on inertia
        self.rotation += self.angular_velocity * dt;
        // Normalize rotation to [0, 2PI)
        self.rotation = self.rotation.rem_euclid(std::f32::consts::TAU);

        let damping = (1.0 - 0.5 * dt).clamp(0.0, 1.0);
        self.angular_velocity *= damping;
        if self.angular_velocity.abs() < 1e-6 {
            self.angular_velocity = 0.0;
        }
    }

    /// Convert local ship coordinates to world coordinates
    pub fn local_to_world(&self, local_pos: Vec2) -> Vec2 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rotated = Vec2::new(
            local_pos.x * cos_r - local_pos.y * sin_r,
            local_pos.x * sin_r + local_pos.y * cos_r,
        );
        self.world_position + rotated
    }

    /// Convert world coordinates to local ship coordinates
    pub fn world_to_local(&self, world_pos: Vec2) -> Vec2 {
        let rel = world_pos - self.world_position;
        let cos_r = self.rotation.cos();
        let sin_r = (-self.rotation).sin();
        Vec2::new(
            rel.x * cos_r - rel.y * sin_r,
            rel.x * sin_r + rel.y * cos_r,
        )
    }

    /// Handle character walking inside ship: verifies collision / walkability
    pub fn can_walk_to(&self, grid_x: i32, grid_y: i32) -> bool {
        if let Some(block) = self.blocks.get(&(grid_x, grid_y)) {
            block.walkable
        } else {
            false // Cannot walk into void without EVA
        }
    }

    /// Player exits ship into EVA
    pub fn exit_to_eva(&self, character: &mut Character, airlock_pos: (i32, i32)) -> Result<(), String> {
        if let Some(block) = self.blocks.get(&airlock_pos) {
            if !block.is_eva_exit {
                return Err("Selected block is not an airlock/EVA exit".to_string());
            }

            character.in_ship = false;
            let local_v = Vec2::new(airlock_pos.0 as f32, airlock_pos.1 as f32);
            character.world_pos = self.local_to_world(local_v);
            // Inherit ship's velocity + slight outward impulse
            character.velocity = self.velocity;
            Ok(())
        } else {
            Err("No block at specified airlock coordinate".to_string())
        }
    }

    /// Player boards / re-enters ship from EVA
    pub fn board_ship(&self, character: &mut Character, airlock_pos: (i32, i32)) -> Result<(), String> {
        let airlock_world = self.local_to_world(Vec2::new(airlock_pos.0 as f32, airlock_pos.1 as f32));
        if character.world_pos.distance(airlock_world) > 3.0 {
            return Err("Too far from airlock to board".to_string());
        }

        character.in_ship = true;
        character.local_pos = Vec2::new(airlock_pos.0 as f32, airlock_pos.1 as f32);
        character.velocity = Vec2::ZERO;
        Ok(())
    }
}
