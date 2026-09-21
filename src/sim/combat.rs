use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatDoctrine {
    KitingSniper,
    BrawlingAssault,
    ScreenEscort,
    HitAndRun,
}

impl Default for CombatDoctrine {
    fn default() -> Self {
        CombatDoctrine::ScreenEscort
    }
}

impl CombatDoctrine {
    pub fn name(&self) -> &'static str {
        match self {
            CombatDoctrine::KitingSniper => "Kiting & Long-Range Artillery",
            CombatDoctrine::BrawlingAssault => "Brawling Assault",
            CombatDoctrine::ScreenEscort => "Screening Escort",
            CombatDoctrine::HitAndRun => "Hit & Run Guerrilla",
        }
    }

    pub fn retreat_threshold(&self) -> f32 {
        match self {
            CombatDoctrine::KitingSniper => 0.65,
            CombatDoctrine::BrawlingAssault => 0.20,
            CombatDoctrine::ScreenEscort => 0.40,
            CombatDoctrine::HitAndRun => 0.75,
        }
    }

    pub fn disengage_bonus(&self) -> f32 {
        match self {
            CombatDoctrine::KitingSniper => 0.35,
            CombatDoctrine::BrawlingAssault => -0.20,
            CombatDoctrine::ScreenEscort => 0.10,
            CombatDoctrine::HitAndRun => 0.50,
        }
    }

    pub fn accuracy_multiplier(&self) -> f32 {
        match self {
            CombatDoctrine::KitingSniper => 1.25,
            CombatDoctrine::BrawlingAssault => 0.85,
            CombatDoctrine::ScreenEscort => 1.10,
            CombatDoctrine::HitAndRun => 0.95,
        }
    }

    pub fn fire_rate_multiplier(&self) -> f32 {
        match self {
            CombatDoctrine::KitingSniper => 0.90,
            CombatDoctrine::BrawlingAssault => 1.30,
            CombatDoctrine::ScreenEscort => 1.05,
            CombatDoctrine::HitAndRun => 1.15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSystem {
    GeneralHull,
    Engines,
    Weapons,
    Reactor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipClass {
    pub name: String,
    pub role: String,
    pub max_shields: f32,
    pub shields: f32,
    pub max_armor: f32,
    pub armor: f32,
    pub max_hull: f32,
    pub hull: f32,
    pub weapon_energy: f32,
    pub weapon_kinetic: f32,
    pub weapon_missiles: f32,
    pub point_defense: f32,
    pub engine_power: f32,
    pub is_destroyed: bool,
}

impl ShipClass {
    pub fn corvette(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: "Corvette".to_string(),
            max_shields: 30.0,
            shields: 30.0,
            max_armor: 20.0,
            armor: 20.0,
            max_hull: 40.0,
            hull: 40.0,
            weapon_energy: 15.0,
            weapon_kinetic: 10.0,
            weapon_missiles: 12.0,
            point_defense: 8.0,
            engine_power: 80.0,
            is_destroyed: false,
        }
    }

    pub fn frigate(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: "Frigate".to_string(),
            max_shields: 80.0,
            shields: 80.0,
            max_armor: 60.0,
            armor: 60.0,
            max_hull: 100.0,
            hull: 100.0,
            weapon_energy: 35.0,
            weapon_kinetic: 30.0,
            weapon_missiles: 25.0,
            point_defense: 25.0,
            engine_power: 60.0,
            is_destroyed: false,
        }
    }

    pub fn cruiser(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: "Cruiser".to_string(),
            max_shields: 220.0,
            shields: 220.0,
            max_armor: 180.0,
            armor: 180.0,
            max_hull: 280.0,
            hull: 280.0,
            weapon_energy: 85.0,
            weapon_kinetic: 75.0,
            weapon_missiles: 60.0,
            point_defense: 50.0,
            engine_power: 45.0,
            is_destroyed: false,
        }
    }

    pub fn battleship(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: "Battleship".to_string(),
            max_shields: 550.0,
            shields: 550.0,
            max_armor: 480.0,
            armor: 480.0,
            max_hull: 650.0,
            hull: 650.0,
            weapon_energy: 200.0,
            weapon_kinetic: 180.0,
            weapon_missiles: 140.0,
            point_defense: 110.0,
            engine_power: 30.0,
            is_destroyed: false,
        }
    }

    pub fn total_health(&self) -> f32 {
        self.shields + self.armor + self.hull
    }

    pub fn total_max_health(&self) -> f32 {
        self.max_shields + self.max_armor + self.max_hull
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalFleet {
    pub faction_id: String,
    pub doctrine: CombatDoctrine,
    pub target_system: TargetSystem,
    pub ships: Vec<ShipClass>,
    pub supplies: f32, // 0.0 to 100.0
}

impl TacticalFleet {
    pub fn new(faction_id: &str, doctrine: CombatDoctrine) -> Self {
        Self {
            faction_id: faction_id.to_string(),
            doctrine,
            target_system: TargetSystem::GeneralHull,
            ships: Vec::new(),
            supplies: 100.0,
        }
    }

    pub fn active_ships(&self) -> impl Iterator<Item = &ShipClass> {
        self.ships.iter().filter(|s| !s.is_destroyed)
    }

    pub fn active_ships_mut(&mut self) -> impl Iterator<Item = &mut ShipClass> {
        self.ships.iter_mut().filter(|s| !s.is_destroyed)
    }

    pub fn total_health(&self) -> f32 {
        self.active_ships().map(|s| s.total_health()).sum()
    }

    pub fn total_max_health(&self) -> f32 {
        self.ships.iter().map(|s| s.total_max_health()).sum()
    }

    pub fn average_engine_power(&self) -> f32 {
        let active: Vec<_> = self.active_ships().collect();
        if active.is_empty() {
            return 0.0;
        }
        active.iter().map(|s| s.engine_power).sum::<f32>() / active.len() as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatReport {
    pub attacker_losses: usize,
    pub defender_losses: usize,
    pub attacker_damage_dealt: f32,
    pub defender_damage_dealt: f32,
    pub attacker_retreated: bool,
    pub defender_retreated: bool,
    pub rounds_fought: usize,
    pub summary: String,
}

pub struct TacticalCombatResolver;

impl TacticalCombatResolver {
    /// Simulates a phased tactical combat engagement between two tactical fleets
    pub fn resolve_battle<R: Rng>(
        attacker: &mut TacticalFleet,
        defender: &mut TacticalFleet,
        rng: &mut R,
    ) -> CombatReport {
        let init_atk_count = attacker.active_ships().count();
        let init_def_count = defender.active_ships().count();

        let mut atk_dmg_total = 0.0f32;
        let mut def_dmg_total = 0.0f32;
        let mut atk_retreated = false;
        let mut def_retreated = false;
        let mut rounds_fought = 0;

        for round in 1..=5 {
            rounds_fought = round;

            if attacker.active_ships().count() == 0 || defender.active_ships().count() == 0 {
                break;
            }

            // Supply penalty check
            let atk_supply_mod = if attacker.supplies < 30.0 { 0.75 } else { 1.0 };
            let def_supply_mod = if defender.supplies < 30.0 { 0.75 } else { 1.0 };

            // 1. Phase 1: Missile Barrage & Point Defense Interception
            let atk_missiles = attacker.active_ships().map(|s| s.weapon_missiles).sum::<f32>()
                * attacker.doctrine.fire_rate_multiplier()
                * atk_supply_mod;
            let def_pd = defender.active_ships().map(|s| s.point_defense).sum::<f32>();

            // Point defense shoots down torpedoes
            let def_intercept_ratio = (def_pd / (atk_missiles.max(1.0) * 1.2)).clamp(0.0, 0.85);
            let atk_missile_hits = atk_missiles * (1.0 - def_intercept_ratio);

            let def_missiles = defender.active_ships().map(|s| s.weapon_missiles).sum::<f32>()
                * defender.doctrine.fire_rate_multiplier()
                * def_supply_mod;
            let atk_pd = attacker.active_ships().map(|s| s.point_defense).sum::<f32>();
            let atk_intercept_ratio = (atk_pd / (def_missiles.max(1.0) * 1.2)).clamp(0.0, 0.85);
            let def_missile_hits = def_missiles * (1.0 - atk_intercept_ratio);

            // 2. Phase 2: Directed Energy & Kinetic Salvo
            let atk_energy = attacker.active_ships().map(|s| s.weapon_energy).sum::<f32>()
                * attacker.doctrine.fire_rate_multiplier()
                * atk_supply_mod;
            let atk_kinetic = attacker.active_ships().map(|s| s.weapon_kinetic).sum::<f32>()
                * attacker.doctrine.fire_rate_multiplier()
                * atk_supply_mod;

            let def_energy = defender.active_ships().map(|s| s.weapon_energy).sum::<f32>()
                * defender.doctrine.fire_rate_multiplier()
                * def_supply_mod;
            let def_kinetic = defender.active_ships().map(|s| s.weapon_kinetic).sum::<f32>()
                * defender.doctrine.fire_rate_multiplier()
                * def_supply_mod;

            // 3. Apply Damage to Defender
            let dmg_to_def = Self::apply_salvo(
                defender,
                atk_energy,
                atk_kinetic,
                atk_missile_hits,
                attacker.doctrine.accuracy_multiplier(),
                attacker.target_system,
                rng,
            );
            atk_dmg_total += dmg_to_def;

            // 4. Apply Damage to Attacker
            let dmg_to_atk = Self::apply_salvo(
                attacker,
                def_energy,
                def_kinetic,
                def_missile_hits,
                defender.doctrine.accuracy_multiplier(),
                defender.target_system,
                rng,
            );
            def_dmg_total += dmg_to_atk;

            // 5. Phase 3: Morale & Tactical Emergency FTL Retreat Check
            let atk_health_ratio = attacker.total_health() / attacker.total_max_health().max(1.0);
            if atk_health_ratio <= attacker.doctrine.retreat_threshold() {
                let jump_chance = (0.50 + attacker.doctrine.disengage_bonus()
                    + (attacker.average_engine_power() / 100.0) * 0.25)
                    .clamp(0.10, 0.95);
                if rng.gen_bool(jump_chance as f64) {
                    atk_retreated = true;
                    break;
                }
            }

            let def_health_ratio = defender.total_health() / defender.total_max_health().max(1.0);
            if def_health_ratio <= defender.doctrine.retreat_threshold() {
                let jump_chance = (0.50 + defender.doctrine.disengage_bonus()
                    + (defender.average_engine_power() / 100.0) * 0.25)
                    .clamp(0.10, 0.95);
                if rng.gen_bool(jump_chance as f64) {
                    def_retreated = true;
                    break;
                }
            }
        }

        let final_atk_count = attacker.active_ships().count();
        let final_def_count = defender.active_ships().count();

        let attacker_losses = init_atk_count.saturating_sub(final_atk_count);
        let defender_losses = init_def_count.saturating_sub(final_def_count);

        let summary = if atk_retreated {
            format!("Attacker initiated Emergency FTL Warp! Defending fleet held the perimeter.")
        } else if def_retreated {
            format!("Defender initiated Emergency FTL Warp! Attacking fleet secured orbital dominance.")
        } else if final_def_count == 0 && final_atk_count > 0 {
            format!("Defender fleet was completely eliminated by attacker strike group.")
        } else if final_atk_count == 0 && final_def_count > 0 {
            format!("Attacker armada was completely repelled and wiped out.")
        } else {
            format!("Tactical stalemate after {} rounds of engagement.", rounds_fought)
        };

        CombatReport {
            attacker_losses,
            defender_losses,
            attacker_damage_dealt: atk_dmg_total,
            defender_damage_dealt: def_dmg_total,
            attacker_retreated: atk_retreated,
            defender_retreated: def_retreated,
            rounds_fought,
            summary,
        }
    }

    /// Distributes weapon salvo across ships in a fleet according to defense layers & subsystems
    fn apply_salvo<R: Rng>(
        target_fleet: &mut TacticalFleet,
        energy_dps: f32,
        kinetic_dps: f32,
        missile_dps: f32,
        accuracy: f32,
        target_system: TargetSystem,
        rng: &mut R,
    ) -> f32 {
        let active_count = target_fleet.active_ships().count();
        if active_count == 0 {
            return 0.0;
        }

        let mut total_dmg_dealt = 0.0;
        let per_ship_energy = (energy_dps * accuracy) / active_count as f32;
        let per_ship_kinetic = (kinetic_dps * accuracy) / active_count as f32;
        let per_ship_missile = (missile_dps * accuracy) / active_count as f32;

        for ship in target_fleet.active_ships_mut() {
            // 1. Energy vs Shields (1.5x damage to shields)
            let mut shield_dmg = per_ship_energy * 1.5 + per_ship_kinetic * 0.7 + per_ship_missile * 0.85;
            let absorbed_shields = shield_dmg.min(ship.shields);
            ship.shields -= absorbed_shields;
            shield_dmg -= absorbed_shields;
            total_dmg_dealt += absorbed_shields;

            // 2. Kinetic vs Armor (1.5x damage to armor)
            let mut armor_dmg = shield_dmg * 0.6 + per_ship_kinetic * 1.5 + per_ship_missile * 1.1;
            let absorbed_armor = armor_dmg.min(ship.armor);
            ship.armor -= absorbed_armor;
            armor_dmg -= absorbed_armor;
            total_dmg_dealt += absorbed_armor;

            // 3. Torpedoes & Residual vs Hull (1.75x damage to hull)
            let hull_dmg = armor_dmg * 0.8 + per_ship_missile * 1.75;
            let absorbed_hull = hull_dmg.min(ship.hull);
            ship.hull -= absorbed_hull;
            total_dmg_dealt += absorbed_hull;

            // 4. Targeted Subsystem Damage
            if absorbed_hull > 5.0 {
                match target_system {
                    TargetSystem::Engines => {
                        ship.engine_power = (ship.engine_power - absorbed_hull * 0.35).max(10.0);
                    }
                    TargetSystem::Weapons => {
                        ship.weapon_energy = (ship.weapon_energy - absorbed_hull * 0.2).max(0.0);
                        ship.weapon_kinetic = (ship.weapon_kinetic - absorbed_hull * 0.2).max(0.0);
                        ship.weapon_missiles = (ship.weapon_missiles - absorbed_hull * 0.2).max(0.0);
                    }
                    TargetSystem::Reactor => {
                        // High risk critical hit chance!
                        if rng.gen_bool(0.12) {
                            ship.hull = 0.0;
                        }
                    }
                    TargetSystem::GeneralHull => {}
                }
            }

            if ship.hull <= 0.0 {
                ship.is_destroyed = true;
            }
        }

        total_dmg_dealt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_doctrine_methods() {
        assert_eq!(CombatDoctrine::default(), CombatDoctrine::ScreenEscort);
        assert_eq!(CombatDoctrine::ScreenEscort.name(), "Screening Escort");
        assert_eq!(CombatDoctrine::BrawlingAssault.retreat_threshold(), 0.20);
        assert_eq!(CombatDoctrine::KitingSniper.retreat_threshold(), 0.65);
    }
}
