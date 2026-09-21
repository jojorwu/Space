use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::galaxy::{Galaxy, ZoneType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ideology {
    Militarist,
    Mercantile,
    Technocrat,
    Pirate,
}

impl Ideology {
    pub fn war_threshold(&self) -> f32 {
        match self {
            Ideology::Militarist => -20.0,
            Ideology::Pirate => -10.0,
            Ideology::Technocrat => -45.0,
            Ideology::Mercantile => -60.0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Ideology::Militarist => "Militarist Expansionist",
            Ideology::Mercantile => "Trade Consortium",
            Ideology::Technocrat => "Technocratic Enclave",
            Ideology::Pirate => "Lawless Corsairs",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionFlag {
    pub symbol: String,
    pub name: String,
    pub primary_color: [f32; 3],
    pub secondary_color: [f32; 3],
}

impl FactionFlag {
    pub fn badge(&self) -> String {
        format!("{} {}", self.symbol, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub ideology: Ideology,
    pub flag: FactionFlag,
    pub treasury: f64,
    pub military_power: f32,
    pub home_zone: ZoneType,
    pub controlled_planets: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplomaticState {
    War,
    Hostile,
    Neutral,
    TradePact,
    Allied,
}

impl std::fmt::Display for DiplomaticState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiplomaticState::War => write!(f, "WAR"),
            DiplomaticState::Hostile => write!(f, "Hostile"),
            DiplomaticState::Neutral => write!(f, "Neutral"),
            DiplomaticState::TradePact => write!(f, "Trade Pact"),
            DiplomaticState::Allied => write!(f, "Allied"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub score: f32, // -100.0 to 100.0
    pub state: DiplomaticState,
    pub war_exhaustion: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomacyMatrix {
    pub relations: HashMap<(String, String), Relationship>,
}

impl DiplomacyMatrix {
    pub fn new() -> Self {
        Self {
            relations: HashMap::new(),
        }
    }

    fn key(a: &str, b: &str) -> (String, String) {
        if a < b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    pub fn set_relation(&mut self, a: &str, b: &str, score: f32, state: DiplomaticState) {
        let key = Self::key(a, b);
        self.relations.insert(
            key,
            Relationship {
                score: score.clamp(-100.0, 100.0),
                state,
                war_exhaustion: 0.0,
            },
        );
    }

    pub fn get_relation(&self, a: &str, b: &str) -> Option<&Relationship> {
        let key = Self::key(a, b);
        self.relations.get(&key)
    }

    pub fn get_relation_mut(&mut self, a: &str, b: &str) -> Option<&mut Relationship> {
        let key = Self::key(a, b);
        self.relations.get_mut(&key)
    }

    pub fn declare_war(&mut self, a: &str, b: &str) {
        if let Some(rel) = self.get_relation_mut(a, b) {
            rel.state = DiplomaticState::War;
            rel.score = -100.0;
            rel.war_exhaustion = 0.0;
        }
    }

    pub fn sign_peace(&mut self, a: &str, b: &str) {
        if let Some(rel) = self.get_relation_mut(a, b) {
            rel.state = DiplomaticState::Hostile;
            rel.score = -20.0;
            rel.war_exhaustion = 0.0;
        }
    }

    pub fn sign_trade_pact(&mut self, a: &str, b: &str) {
        if let Some(rel) = self.get_relation_mut(a, b) {
            rel.state = DiplomaticState::TradePact;
            rel.score = rel.score.max(30.0);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FleetMission {
    Colonization {
        colony_supplies: f32,
    },
    MilitaryInvasion {
        firepower: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskForceRole {
    SiegeArmada,
    HunterKillerPatrol,
    RaiderSquadron,
    BorderGarrison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionFleet {
    pub id: u64,
    pub owner_faction: String,
    pub origin_planet: usize,
    pub target_planet: usize,
    pub progress: f32, // 0.0 to 1.0
    pub speed: f32,
    pub mission: FleetMission,
    pub role: TaskForceRole,
    pub doctrine: crate::sim::combat::CombatDoctrine,
    pub supplies: f32,
}

pub struct FactionManager {
    pub factions: HashMap<String, Faction>,
    pub diplomacy: DiplomacyMatrix,
    pub active_fleets: Vec<FactionFleet>,
    pub next_fleet_id: u64,
}

impl FactionManager {
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
            diplomacy: DiplomacyMatrix::new(),
            active_fleets: Vec::new(),
            next_fleet_id: 1,
        }
    }

    /// Procedurally generate factions with randomized names, heraldry, colors, and ideologies
    pub fn generate_procedural<R: Rng>(
        &mut self,
        rng: &mut R,
        count: usize,
        galaxy: &mut Galaxy,
    ) {
        let prefixes = [
            "Solar", "Iron", "Crimson", "Cyber", "Astral", "Free", "Sovereign",
            "Stellar", "Obsidian", "Vanguard", "Titan", "Zenith", "Phoenix",
            "Nova", "Aegis", "Eclipse", "Celestial", "Apex", "Nexus", "Valiant",
        ];

        let suffixes = [
            "Empire", "Syndicate", "Federation", "Hegemony", "Commonwealth",
            "Dominion", "Directorate", "Enclave", "Ascendancy", "Republic",
            "Conglomerate", "Coalition", "Alliance", "Order", "Marauders",
        ];

        let flag_symbols = [
            ("[*]", "Star Crest"),
            ("[X]", "Crossed Sabers"),
            ("[$]", "Merchant Seal"),
            ("[#]", "Quantum Core"),
            ("[^]", "Crown of Dominion"),
            ("[o]", "Aegis Bulwark"),
            ("[~]", "Celestial Falcon"),
            ("[%]", "Orbital Gear"),
        ];

        let palettes = [
            ("Solar Gold", [1.0, 0.84, 0.0], [0.2, 0.2, 0.2]),
            ("Crimson Blood", [0.86, 0.08, 0.24], [0.1, 0.1, 0.1]),
            ("Deep Cobalt", [0.0, 0.28, 0.67], [0.9, 0.9, 0.9]),
            ("Emerald Jade", [0.0, 0.75, 0.35], [0.1, 0.2, 0.1]),
            ("Void Violet", [0.55, 0.0, 0.7], [0.8, 0.7, 1.0]),
            ("Cyber Cyan", [0.0, 0.85, 0.9], [0.05, 0.1, 0.15]),
            ("Obsidian Dark", [0.15, 0.15, 0.15], [0.9, 0.2, 0.2]),
        ];

        let ideologies = [
            Ideology::Militarist,
            Ideology::Mercantile,
            Ideology::Technocrat,
            Ideology::Pirate,
        ];

        let zone_order = [ZoneType::Core, ZoneType::Neutral, ZoneType::Frontier];

        let mut generated_ids = Vec::new();

        for i in 0..count {
            let pref = prefixes[rng.gen_range(0..prefixes.len())];
            let suff = suffixes[rng.gen_range(0..suffixes.len())];
            let name = format!("{} {}", pref, suff);
            let id = format!("{}_{}_{}", pref.to_lowercase(), suff.to_lowercase(), i);

            let (symbol, sym_name) = flag_symbols[rng.gen_range(0..flag_symbols.len())];
            let (pal_name, p_col, s_col) = palettes[rng.gen_range(0..palettes.len())];
            let flag = FactionFlag {
                symbol: symbol.to_string(),
                name: format!("{} ({})", sym_name, pal_name),
                primary_color: p_col,
                secondary_color: s_col,
            };

            let ideology = ideologies[i % ideologies.len()];
            let home_zone = zone_order[i % zone_order.len()];

            let faction = Faction {
                id: id.clone(),
                name,
                ideology,
                flag,
                treasury: rng.gen_range(50000.0..150000.0),
                military_power: match ideology {
                    Ideology::Militarist => rng.gen_range(600.0..1000.0),
                    Ideology::Pirate => rng.gen_range(400.0..750.0),
                    Ideology::Technocrat => rng.gen_range(500.0..800.0),
                    Ideology::Mercantile => rng.gen_range(300.0..500.0),
                },
                home_zone,
                controlled_planets: Vec::new(),
            };

            self.factions.insert(id.clone(), faction);
            generated_ids.push(id);
        }

        // Initialize bilateral relationships
        for i in 0..generated_ids.len() {
            for j in (i + 1)..generated_ids.len() {
                let a = &generated_ids[i];
                let b = &generated_ids[j];
                let f_a = &self.factions[a];
                let f_b = &self.factions[b];

                let base_score = match (f_a.ideology, f_b.ideology) {
                    (Ideology::Pirate, _) | (_, Ideology::Pirate) => -40.0,
                    (Ideology::Mercantile, Ideology::Mercantile) => 40.0,
                    (Ideology::Militarist, Ideology::Militarist) => -30.0,
                    _ => 0.0,
                };

                let state = if base_score >= 30.0 {
                    DiplomaticState::TradePact
                } else if base_score <= -30.0 {
                    DiplomaticState::Hostile
                } else {
                    DiplomaticState::Neutral
                };

                self.diplomacy.set_relation(a, b, base_score, state);
            }
        }

        // Distribute initial planet ownership:
        // ~50% of planets are claimed by factions, ~50% remain UNALIGNED/INDEPENDENT!
        let mut faction_idx = 0;
        for planet in galaxy.planets.iter_mut() {
            // Unclaimed / independent chance:
            // Core zone: 20% independent, 80% claimed
            // Neutral zone: 50% independent, 50% claimed
            // Frontier zone: 70% independent, 30% claimed
            let unaligned_chance = match planet.zone {
                ZoneType::Core => 0.20,
                ZoneType::Neutral => 0.50,
                ZoneType::Frontier => 0.70,
            };

            if rng.gen_bool(unaligned_chance) {
                planet.owner_faction = None;
                planet.planetary_defense = 50.0;
                planet.max_defense = 100.0;
            } else {
                let f_id = &generated_ids[faction_idx % generated_ids.len()];
                planet.owner_faction = Some(f_id.clone());
                planet.planetary_defense = 250.0;
                planet.max_defense = 300.0;

                if let Some(faction) = self.factions.get_mut(f_id) {
                    faction.controlled_planets.push(planet.id);
                }
                faction_idx += 1;
            }
        }
    }

    /// Simulate one strategic turn of diplomacy, war, colonization and expansion
    pub fn tick_strategic_turn<R: Rng>(
        &mut self,
        rng: &mut R,
        galaxy: &mut Galaxy,
    ) -> Vec<String> {
        let mut events = Vec::new();

        // 1. Planetary tax collection & defense regeneration
        for planet in galaxy.planets.iter_mut() {
            if let Some(ref owner) = planet.owner_faction {
                let tax = (planet.population_millions * 1.5).min(1500.0);
                if let Some(faction) = self.factions.get_mut(owner) {
                    faction.treasury += tax;
                }
                // Regenerate defense
                planet.planetary_defense = (planet.planetary_defense + 10.0).min(planet.max_defense);
            }
        }

        // 2. Diplomacy evolution: drift relations, handle wars and peace
        let faction_ids: Vec<String> = self.factions.keys().cloned().collect();
        for i in 0..faction_ids.len() {
            for j in (i + 1)..faction_ids.len() {
                let id_a = &faction_ids[i];
                let id_b = &faction_ids[j];
                let f_a = self.factions.get(id_a).unwrap().clone();
                let f_b = self.factions.get(id_b).unwrap().clone();

                if let Some(rel) = self.diplomacy.get_relation_mut(id_a, id_b) {
                    match rel.state {
                        DiplomaticState::War => {
                            rel.war_exhaustion += rng.gen_range(3.0..8.0);
                            if rel.war_exhaustion >= 100.0 {
                                // Sign peace treaty due to exhaustion!
                                rel.state = DiplomaticState::Hostile;
                                rel.score = -20.0;
                                rel.war_exhaustion = 0.0;
                                events.push(format!(
                                    "[DIPLOMACY: PEACE] {} and {} signed a Ceasefire Treaty due to heavy war fatigue.",
                                    f_a.name, f_b.name
                                ));
                            }
                        }
                        DiplomaticState::Hostile | DiplomaticState::Neutral => {
                            // Check if war should be declared
                            let min_threshold = f_a.ideology.war_threshold().max(f_b.ideology.war_threshold());
                            
                            // Tension drift
                            if f_a.ideology == Ideology::Militarist || f_b.ideology == Ideology::Militarist {
                                rel.score -= rng.gen_range(1.0..4.0);
                            } else if f_a.ideology == Ideology::Pirate || f_b.ideology == Ideology::Pirate {
                                rel.score -= rng.gen_range(2.0..6.0);
                            }

                            if rel.score <= min_threshold {
                                rel.state = DiplomaticState::War;
                                rel.war_exhaustion = 0.0;
                                events.push(format!(
                                    "[DIPLOMACY: WAR DECLARED] {} declared WAR on {}! Hostilities commenced.",
                                    f_a.name, f_b.name
                                ));
                            }
                        }
                        DiplomaticState::TradePact => {
                            rel.score = (rel.score + 1.0).min(100.0);
                        }
                        DiplomaticState::Allied => {
                            rel.score = (rel.score + 0.5).min(100.0);
                        }
                    }
                }
            }
        }

        // 3. Faction AI decisions: Colonization of independent worlds or Invasions of enemies
        for f_id in &faction_ids {
            let (treasury, military_power, origin_planet_id, f_name) = match self.factions.get(f_id) {
                Some(f) if f.treasury >= 10000.0 && !f.controlled_planets.is_empty() => (
                    f.treasury,
                    f.military_power,
                    f.controlled_planets[0],
                    f.name.clone(),
                ),
                _ => continue,
            };

            // A) Check for unaligned / independent planets to colonize!
            let unaligned_target = galaxy.planets.iter().find(|p| {
                p.owner_faction.is_none() && galaxy.distance(origin_planet_id, p.id) < 500.0
            });

            if let Some(target) = unaligned_target {
                let cost = 8000.0;
                if treasury >= cost && rng.gen_bool(0.40) {
                    if let Some(f) = self.factions.get_mut(f_id) {
                        f.treasury -= cost;
                    }
                    let target_id = target.id;
                    let target_name = target.name.clone();

                    let fleet = FactionFleet {
                        id: self.next_fleet_id,
                        owner_faction: f_id.clone(),
                        origin_planet: origin_planet_id,
                        target_planet: target_id,
                        progress: 0.0,
                        speed: 35.0,
                        mission: FleetMission::Colonization {
                            colony_supplies: 100.0,
                        },
                        role: TaskForceRole::SiegeArmada,
                        doctrine: crate::sim::combat::CombatDoctrine::ScreenEscort,
                        supplies: 100.0,
                    };
                    self.next_fleet_id += 1;
                    self.active_fleets.push(fleet);

                    events.push(format!(
                        "[COLONIZATION EXPEDITION] {} dispatched a Colony Fleet to independent world '{}'",
                        f_name, target_name
                    ));
                    continue;
                }
            }

            // B) If at war, launch a Military Invasion Fleet against enemy planet!
            let mut war_target: Option<usize> = None;
            for other_id in &faction_ids {
                if other_id == f_id {
                    continue;
                }
                if let Some(rel) = self.diplomacy.get_relation(f_id, other_id) {
                    if rel.state == DiplomaticState::War {
                        if let Some(enemy) = self.factions.get(other_id) {
                            if let Some(&enemy_planet_id) = enemy.controlled_planets.first() {
                                war_target = Some(enemy_planet_id);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(target_id) = war_target {
                let cost = 15000.0;
                if treasury >= cost && military_power > 150.0 && rng.gen_bool(0.50) {
                    let fleet_firepower = 180.0;
                    if let Some(f) = self.factions.get_mut(f_id) {
                        f.treasury -= cost;
                        f.military_power -= 50.0;
                    }

                    let target_name = galaxy.get_planet(target_id).map(|p| p.name.clone()).unwrap_or_default();

                    let fleet = FactionFleet {
                        id: self.next_fleet_id,
                        owner_faction: f_id.clone(),
                        origin_planet: origin_planet_id,
                        target_planet: target_id,
                        progress: 0.0,
                        speed: 30.0,
                        mission: FleetMission::MilitaryInvasion {
                            firepower: fleet_firepower,
                        },
                        role: TaskForceRole::SiegeArmada,
                        doctrine: crate::sim::combat::CombatDoctrine::BrawlingAssault,
                        supplies: 100.0,
                    };
                    self.next_fleet_id += 1;
                    self.active_fleets.push(fleet);

                    events.push(format!(
                        "[WAR ARMADA] {} launched an Invasion Armada (Firepower {:.0}) against enemy bastion '{}'",
                        f_name, fleet_firepower, target_name
                    ));
                }
            }
        }

        // 3. Advance active fleets and resolve arrival encounters
        let mut completed_fleet_indices = Vec::new();

        for (idx, fleet) in self.active_fleets.iter_mut().enumerate() {
            let dist = galaxy.distance(fleet.origin_planet, fleet.target_planet).max(1.0);
            let step = fleet.speed / dist;
            fleet.progress += step;

            if fleet.progress >= 1.0 {
                completed_fleet_indices.push(idx);
                let target_planet_id = fleet.target_planet;
                let owner_fid = fleet.owner_faction.clone();

                match fleet.mission {
                    FleetMission::Colonization { colony_supplies } => {
                        if let Some(planet) = galaxy.get_planet_mut(target_planet_id) {
                            if planet.owner_faction.is_none() {
                                planet.owner_faction = Some(owner_fid.clone());
                                planet.planetary_defense = colony_supplies;
                                planet.max_defense = 200.0;
                                planet.orbital_defense_level = 0;
                                planet.planetary_shield = false;

                                if let Some(faction) = self.factions.get_mut(&owner_fid) {
                                    faction.controlled_planets.push(target_planet_id);
                                    events.push(format!(
                                        "[PLANET CLAIMED] {} established a colony on '{}'! Flag raised: {}",
                                        faction.name, planet.name, faction.flag.badge()
                                    ));
                                }
                            } else {
                                events.push(format!(
                                    "[COLONY ABORTED] Planet '{}' was claimed before colony arrival.",
                                    planet.name
                                ));
                            }
                        }
                    }
                    FleetMission::MilitaryInvasion { firepower } => {
                        let mut planet_captured = false;
                        let mut defender_fid: Option<String> = None;
                        let mut p_name = String::new();

                        if let Some(planet) = galaxy.get_planet_mut(target_planet_id) {
                            p_name = planet.name.clone();
                            defender_fid = planet.owner_faction.clone();

                            // Mitigation from planetary energy shield and orbital platforms
                            let shielded_firepower = if planet.planetary_shield {
                                firepower * 0.65 // Shield absorbs 35% of bombardment
                            } else {
                                firepower
                            };
                            let counter_damage = match planet.orbital_defense_level {
                                1 => 40.0,
                                2 => 90.0,
                                _ => 0.0,
                            };
                            let net_firepower = (shielded_firepower - counter_damage).max(10.0);

                            if planet.planetary_defense <= net_firepower {
                                // Conquered!
                                planet_captured = true;
                                planet.owner_faction = Some(owner_fid.clone());
                                planet.planetary_defense = (net_firepower - planet.planetary_defense).max(80.0);
                                planet.max_defense = 250.0;
                                planet.orbital_defense_level = 0;
                                planet.planetary_shield = false;
                                // Economic shock: market inventory damaged
                                for item in planet.market.inventory.values_mut() {
                                    item.current_stock *= 0.5;
                                }
                            } else {
                                planet.planetary_defense -= net_firepower;
                            }
                        }

                        let attacker_name = self.factions.get(&owner_fid).map(|f| f.name.clone()).unwrap_or_default();
                        let flag_badge = self.factions.get(&owner_fid).map(|f| f.flag.badge()).unwrap_or_default();

                        if planet_captured {
                            if let Some(old_owner) = defender_fid {
                                if let Some(old_f) = self.factions.get_mut(&old_owner) {
                                    old_f.controlled_planets.retain(|&id| id != target_planet_id);
                                    events.push(format!(
                                        "[PLANETARY CONQUEST] {} successfully conquered '{}' from {}! New banner: {}",
                                        attacker_name, p_name, old_f.name, flag_badge
                                    ));
                                }
                            }
                            if let Some(new_f) = self.factions.get_mut(&owner_fid) {
                                new_f.controlled_planets.push(target_planet_id);
                            }
                        } else {
                            events.push(format!(
                                "[SIEGE REPELLED] Planetary garrisons on '{}' repelled {}'s invasion force!",
                                p_name, attacker_name
                            ));
                        }
                    }
                }
            }
        }

        // Remove finished fleets in reverse order
        for &idx in completed_fleet_indices.iter().rev() {
            self.active_fleets.remove(idx);
        }

        events
    }
}
