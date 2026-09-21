use crate::ai::personality::AiPersonality;
use crate::factions::{DiplomaticState, Faction, FactionManager, FleetMission};
use crate::galaxy::{Galaxy, PlanetType};
use crate::sim::events::GameEvent;
use crate::sim::spatial::SpatialIndex;

pub struct StrategicAi;

impl StrategicAi {
    /// Evaluates all unaligned planets and selects the most strategic candidate for colonization
    pub fn pick_colonization_target(
        faction: &Faction,
        personality: &AiPersonality,
        brain: Option<&crate::ai::brain::AiBrain>,
        galaxy: &Galaxy,
        spatial: &SpatialIndex,
    ) -> Option<(usize, f32)> {
        if faction.controlled_planets.is_empty() {
            return None;
        }

        let mut best_target = None;
        let mut best_score = 0.0f32;

        let risk_tolerance = brain.map_or(personality.risk_tolerance, |b| b.effective_risk_tolerance(personality));

        for planet in &galaxy.planets {
            if planet.owner_faction.is_some() {
                continue;
            }

            // Find distance to closest friendly planet
            let min_dist = faction
                .controlled_planets
                .iter()
                .map(|&p_id| spatial.distance(p_id, planet.id))
                .fold(f32::MAX, f32::min);

            if min_dist > 650.0 {
                continue;
            }

            // Resource strategic value
            let type_value = match planet.planet_type {
                PlanetType::MiningColony => 1.8 * personality.economic_greed,
                PlanetType::IndustrialForge => 1.5,
                PlanetType::HighTechHub => 1.4,
                PlanetType::AgriWorld => 1.3,
                PlanetType::FrontierOutpost => 1.1,
            };

            let pop_value = (planet.population_millions as f32).clamp(10.0, 500.0) / 100.0;
            let danger = brain.map_or(0.0, |b| b.get_danger(planet.id));

            // Utility: High value, low danger, close proximity, personality expansionism
            let danger_penalty = 1.0 + (danger / (risk_tolerance + 0.1));
            let score = (((type_value * pop_value) / (min_dist + 40.0)) * personality.expansionism) / danger_penalty;

            if score > best_score {
                best_score = score;
                best_target = Some((planet.id, score));
            }
        }

        best_target
    }

    /// Evaluates enemy worlds and picks the highest-value conquest target
    pub fn pick_invasion_target(
        faction: &Faction,
        personality: &AiPersonality,
        brain: Option<&crate::ai::brain::AiBrain>,
        galaxy: &Galaxy,
        spatial: &SpatialIndex,
        factions: &FactionManager,
    ) -> Option<(usize, f32)> {
        if faction.controlled_planets.is_empty() {
            return None;
        }

        let mut best_target = None;
        let mut best_score = 0.0f32;

        for (other_id, other_faction) in &factions.factions {
            if other_id == &faction.id {
                continue;
            }

            // Must be at War
            let at_war = factions
                .diplomacy
                .get_relation(&faction.id, other_id)
                .map(|r| r.state == DiplomaticState::War)
                .unwrap_or(false);

            if !at_war {
                continue;
            }

            let eff_aggression = brain.map_or(personality.aggression, |b| b.effective_aggression(personality, Some(other_id)));

            for &enemy_planet_id in &other_faction.controlled_planets {
                let enemy_planet = match galaxy.get_planet(enemy_planet_id) {
                    Some(p) => p,
                    None => continue,
                };

                let min_dist = faction
                    .controlled_planets
                    .iter()
                    .map(|&p_id| spatial.distance(p_id, enemy_planet_id))
                    .fold(f32::MAX, f32::min);

                if min_dist > 700.0 {
                    continue;
                }

                // Target vulnerability: lower defense = easier capture
                let vulnerability = 300.0 / (enemy_planet.planetary_defense + 30.0);
                let strategic_value = (enemy_planet.population_millions as f32).clamp(20.0, 1000.0) / 100.0;

                let score = ((strategic_value * vulnerability) / (min_dist + 50.0)) * eff_aggression;

                if score > best_score {
                    best_score = score;
                    best_target = Some((enemy_planet_id, score));
                }
            }
        }

        best_target
    }

    /// Checks for incoming enemy armadas targeting faction territory and attempts to intercept them!
    pub fn check_interception_opportunities(
        faction_id: &str,
        faction: &mut Faction,
        personality: &AiPersonality,
        mut brain: Option<&mut crate::ai::brain::AiBrain>,
        active_fleets: &mut Vec<crate::factions::FactionFleet>,
        galaxy: &Galaxy,
        cycle: usize,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();

        if faction.treasury < 6000.0 || faction.military_power < 100.0 {
            return events;
        }

        let mut intercepted_indices = Vec::new();

        for (idx, fleet) in active_fleets.iter().enumerate() {
            if fleet.owner_faction == faction_id {
                continue;
            }

            if let FleetMission::MilitaryInvasion { firepower } = fleet.mission {
                // Is this armada targeting one of our planets?
                if faction.controlled_planets.contains(&fleet.target_planet) {
                    // Decide whether to launch an interception sortie
                    let chance = brain.as_ref().map_or(personality.interception_chance, |b| {
                        (personality.interception_chance * b.effective_defense_bias(personality)).clamp(0.1, 0.95)
                    });
                    let roll: f32 = rand::random();
                    if roll <= chance {
                        let intercept_cost = 5000.0;
                        let def_bias = brain.as_ref().map_or(personality.defense_bias, |b| b.effective_defense_bias(personality));
                        let patrol_power = 120.0 * def_bias;

                        faction.treasury -= intercept_cost;
                        faction.military_power -= 30.0;

                        let target_name = galaxy.get_planet(fleet.target_planet).map(|p| p.name.clone()).unwrap_or_default();
                        let enemy_name = fleet.owner_faction.clone();

                        let defender_won = patrol_power >= firepower * 0.75;
                        if defender_won {
                            intercepted_indices.push(idx);
                            if let Some(ref mut b) = brain {
                                b.record_combat_victory(fleet.target_planet, &enemy_name, cycle);
                            }
                        } else {
                            if let Some(ref mut b) = brain {
                                b.record_combat_loss(fleet.target_planet, 2, &enemy_name, cycle);
                            }
                        }

                        events.push(GameEvent::FleetIntercepted {
                            defender: faction.name.clone(),
                            attacker: enemy_name,
                            target_planet: target_name,
                            defender_won,
                        });
                        break;
                    }
                }
            }
        }

        // Remove destroyed fleets in reverse
        for &idx in intercepted_indices.iter().rev() {
            active_fleets.remove(idx);
        }

        events
    }
}
