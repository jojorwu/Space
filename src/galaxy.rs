use std::collections::HashMap;
use std::f32::consts::PI;
use glam::Vec2;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::economy::PlanetaryMarket;
use crate::items::ItemDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneType {
    Core,
    Neutral,
    Frontier,
}

impl std::fmt::Display for ZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZoneType::Core => write!(f, "Core"),
            ZoneType::Neutral => write!(f, "Neutral"),
            ZoneType::Frontier => write!(f, "Frontier"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanetType {
    AgriWorld,
    MiningColony,
    IndustrialForge,
    HighTechHub,
    FrontierOutpost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Planet {
    pub id: usize,
    pub name: String,
    pub position: Vec2,
    pub zone: ZoneType,
    pub planet_type: PlanetType,
    pub population_millions: f64,
    pub market: PlanetaryMarket,
    pub landed_ships: Vec<u64>,
    pub owner_faction: Option<String>,
    pub planetary_defense: f32,
    pub max_defense: f32,
    pub orbital_defense_level: u8,
    pub planetary_shield: bool,
}

pub struct Galaxy {
    pub planets: Vec<Planet>,
}

impl Galaxy {
    /// Generates 100 planets divided across the 3 zones
    pub fn generate_procedural<R: Rng>(rng: &mut R, _items: &HashMap<String, ItemDef>) -> Self {
        let mut planets = Vec::with_capacity(100);

        let prefix_names = [
            "Aethel", "Nova", "Zephyr", "Astra", "Kronos", "Hyperion", "Titan", "Valkyrie",
            "Seraph", "Vanguard", "Orion", "Helios", "Elysium", "Apex", "Cestus", "Tiber",
            "Cygnus", "Boreas", "Vega", "Sirius", "Altair", "Deneb", "Castor", "Pollux",
            "Rigel", "Betelgeuse", "Solara", "Krynn", "Obsidian", "Avalon",
        ];

        let suffix_names = [
            "Prime", "Major", "Minor", "Secundus", "Station", "Sanctuary", "Reach", "Haven",
            "Colony", "Forge", "Deep", "Depot", "Outpost", "Terminus", "Spire", "Gateway",
        ];

        let mut id_counter = 0;

        // 1. Generate Core Zone (30 planets)
        for i in 0..30 {
            let angle = rng.gen_range(0.0..(2.0 * PI));
            let dist = rng.gen_range(20.0..320.0);
            let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

            let p_type = match i % 3 {
                0 => PlanetType::HighTechHub,
                1 => PlanetType::IndustrialForge,
                _ => PlanetType::AgriWorld,
            };

            let name = format!(
                "{} {}",
                prefix_names[i % prefix_names.len()],
                suffix_names[rng.gen_range(0..suffix_names.len())]
            );

            let pop = rng.gen_range(500.0..3000.0);
            let mut market = PlanetaryMarket::new(rng.gen_range(50000.0..250000.0));
            Self::setup_market(&mut market, p_type, pop, ZoneType::Core);

            planets.push(Planet {
                id: id_counter,
                name,
                position: pos,
                zone: ZoneType::Core,
                planet_type: p_type,
                population_millions: pop,
                market,
                landed_ships: Vec::new(),
                owner_faction: None,
                planetary_defense: 100.0,
                max_defense: 250.0,
                orbital_defense_level: 0,
                planetary_shield: false,
            });
            id_counter += 1;
        }

        // 2. Generate Neutral Zone (40 planets)
        for i in 0..40 {
            let angle = rng.gen_range(0.0..(2.0 * PI));
            let dist = rng.gen_range(350.0..720.0);
            let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

            let p_type = match i % 4 {
                0 => PlanetType::IndustrialForge,
                1 => PlanetType::MiningColony,
                2 => PlanetType::AgriWorld,
                _ => PlanetType::FrontierOutpost,
            };

            let name = format!(
                "{} {}",
                prefix_names[(i + 5) % prefix_names.len()],
                suffix_names[rng.gen_range(0..suffix_names.len())]
            );

            let pop = rng.gen_range(80.0..800.0);
            let mut market = PlanetaryMarket::new(rng.gen_range(20000.0..100000.0));
            Self::setup_market(&mut market, p_type, pop, ZoneType::Neutral);

            planets.push(Planet {
                id: id_counter,
                name,
                position: pos,
                zone: ZoneType::Neutral,
                planet_type: p_type,
                population_millions: pop,
                market,
                landed_ships: Vec::new(),
                owner_faction: None,
                planetary_defense: 50.0,
                max_defense: 150.0,
                orbital_defense_level: 0,
                planetary_shield: false,
            });
            id_counter += 1;
        }

        // 3. Generate Frontier Zone (30 planets)
        for i in 0..30 {
            let angle = rng.gen_range(0.0..(2.0 * PI));
            let dist = rng.gen_range(750.0..1200.0);
            let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

            let p_type = match i % 3 {
                0 => PlanetType::MiningColony,
                1 => PlanetType::FrontierOutpost,
                _ => PlanetType::AgriWorld,
            };

            let name = format!(
                "{} {}",
                prefix_names[(i + 11) % prefix_names.len()],
                suffix_names[rng.gen_range(0..suffix_names.len())]
            );

            let pop = rng.gen_range(5.0..150.0);
            let mut market = PlanetaryMarket::new(rng.gen_range(5000.0..40000.0));
            Self::setup_market(&mut market, p_type, pop, ZoneType::Frontier);

            planets.push(Planet {
                id: id_counter,
                name,
                position: pos,
                zone: ZoneType::Frontier,
                planet_type: p_type,
                population_millions: pop,
                market,
                landed_ships: Vec::new(),
                owner_faction: None,
                planetary_defense: 30.0,
                max_defense: 100.0,
                orbital_defense_level: 0,
                planetary_shield: false,
            });
            id_counter += 1;
        }

        Galaxy { planets }
    }

    fn setup_market(
        market: &mut PlanetaryMarket,
        p_type: PlanetType,
        pop: f64,
        _zone: ZoneType,
    ) {
        let base_need = (pop * 0.05).max(5.0);

        // General life support demand for all populated worlds
        market.register_commodity("purified_water", base_need * 2.0, base_need * 2.5, 0.0, base_need * 0.5);
        market.register_commodity("hydroponic_food", base_need * 1.5, base_need * 2.0, 0.0, base_need * 0.4);
        market.register_commodity("fuel_cells", base_need * 1.0, base_need * 1.2, 0.0, base_need * 0.2);

        match p_type {
            PlanetType::AgriWorld => {
                // High production of food and water
                market.register_commodity("purified_water", base_need * 10.0, base_need * 4.0, base_need * 1.5, base_need * 0.2);
                market.register_commodity("hydroponic_food", base_need * 8.0, base_need * 3.0, base_need * 1.8, base_need * 0.2);
                market.register_commodity("medical_supplies", base_need * 0.5, base_need * 1.0, 0.0, base_need * 0.1);
            }
            PlanetType::MiningColony => {
                // High production of raw iron and titanium ore, high consumption of fuel and food
                market.register_commodity("iron_ore", base_need * 15.0, base_need * 5.0, base_need * 3.0, 0.0);
                market.register_commodity("titanium_ore", base_need * 8.0, base_need * 2.0, base_need * 1.5, 0.0);
                market.register_commodity("medical_supplies", base_need * 0.2, base_need * 1.5, 0.0, base_need * 0.15);
            }
            PlanetType::IndustrialForge => {
                // Consumes ore, produces refined metal and plasma batteries
                market.register_commodity("iron_ore", base_need * 2.0, base_need * 8.0, 0.0, base_need * 2.0);
                market.register_commodity("refined_metal", base_need * 10.0, base_need * 3.0, base_need * 2.0, base_need * 0.3);
                market.register_commodity("plasma_batteries", base_need * 5.0, base_need * 2.0, base_need * 0.8, base_need * 0.1);
            }
            PlanetType::HighTechHub => {
                // Consumes refined metal, produces microelectronics and quantum cores
                market.register_commodity("refined_metal", base_need * 1.0, base_need * 6.0, 0.0, base_need * 1.5);
                market.register_commodity("microelectronics", base_need * 6.0, base_need * 2.0, base_need * 1.2, base_need * 0.2);
                market.register_commodity("quantum_cores", base_need * 4.0, base_need * 1.5, base_need * 0.5, 0.0);
                market.register_commodity("medical_supplies", base_need * 5.0, base_need * 2.0, base_need * 1.0, base_need * 0.3);
            }
            PlanetType::FrontierOutpost => {
                // Frontier mining & high demand for tech and medicine
                market.register_commodity("titanium_ore", base_need * 10.0, base_need * 2.0, base_need * 2.0, 0.0);
                market.register_commodity("medical_supplies", base_need * 0.1, base_need * 2.0, 0.0, base_need * 0.2);
                market.register_commodity("microelectronics", base_need * 0.2, base_need * 1.5, 0.0, base_need * 0.1);
            }
        }
    }

    /// Step economic tick for all 100 planets
    pub fn tick_economy(&mut self) {
        for planet in self.planets.iter_mut() {
            planet.market.tick();
        }
    }

    pub fn get_planet(&self, id: usize) -> Option<&Planet> {
        self.planets.get(id)
    }

    pub fn get_planet_mut(&mut self, id: usize) -> Option<&mut Planet> {
        self.planets.get_mut(id)
    }

    pub fn distance(&self, id_a: usize, id_b: usize) -> f32 {
        if let (Some(a), Some(b)) = (self.planets.get(id_a), self.planets.get(id_b)) {
            a.position.distance(b.position)
        } else {
            f32::MAX
        }
    }
}
