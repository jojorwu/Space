use std::collections::HashMap;
use glam::Vec2;
use rand::Rng;

use crate::ai::personality::AiPersonality;
use crate::ai::strategic_ai::StrategicAi;
use crate::ai::trader_ai::{TraderAi, TraderArchetype};
use crate::factions::{FactionFleet, FactionManager, FleetMission};
use crate::galaxy::Galaxy;
use crate::items::ItemDef;
use crate::sim::events::{EventBus, GameEvent};
use crate::sim::spatial::SpatialIndex;
use crate::traders::TradeFleet;

pub struct GalacticWorld {
    pub galaxy: Galaxy,
    pub spatial: SpatialIndex,
    pub factions: FactionManager,
    pub personalities: HashMap<String, AiPersonality>,
    pub traders: TradeFleet,
    pub in_flight_cargo: HashMap<(usize, String), f64>,
    pub event_bus: EventBus,
    pub items: HashMap<String, ItemDef>,
    pub cycle_count: usize,
}

impl GalacticWorld {
    pub fn new(
        galaxy: Galaxy,
        factions: FactionManager,
        personalities: HashMap<String, AiPersonality>,
        traders: TradeFleet,
        items: HashMap<String, ItemDef>,
    ) -> Self {
        // Build flat O(1) spatial index from planet coordinates
        let positions: Vec<Vec2> = galaxy.planets.iter().map(|p| p.position).collect();
        let spatial = SpatialIndex::build(&positions);
        let event_bus = EventBus::new(30);

        Self {
            galaxy,
            spatial,
            factions,
            personalities,
            traders,
            in_flight_cargo: HashMap::new(),
            event_bus,
            items,
            cycle_count: 1,
        }
    }

    /// Primary simulation cycle: updates economy, AI agents, diplomacy, fleets, and wars
    pub fn tick<R: Rng>(&mut self, rng: &mut R) {
        self.cycle_count += 1;

        // 1. Economic Production & Consumption on all 100 planets
        self.galaxy.tick_economy();

        // 2. Autonomous Utility-based Trader Logistics
        self.update_traders();

        // 3. Faction Defense & Fleet Interception Patrols
        self.check_fleet_interceptions();

        // 4. Strategic AI & Diplomatic Turn
        self.update_factions_and_armadas(rng);
    }

    fn update_traders(&mut self) {
        let archetypes = [
            TraderArchetype::StandardMerchant,
            TraderArchetype::Smuggler,
            TraderArchetype::BulkHauler,
            TraderArchetype::StandardMerchant,
        ];

        for (idx, ship) in self.traders.ships.iter_mut().enumerate() {
            let archetype = archetypes[idx % archetypes.len()];

            if let Some(dest_id) = ship.destination_planet {
                let dist = self.spatial.distance(ship.current_planet, dest_id).max(1.0);
                let step = ship.speed / dist;
                ship.travel_progress += step;

                if ship.travel_progress >= 1.0 {
                    // Arrived! Sell cargo
                    ship.travel_progress = 0.0;
                    ship.current_planet = dest_id;
                    ship.destination_planet = None;

                    if let Some(planet) = self.galaxy.get_planet_mut(dest_id) {
                        let cargo_items: Vec<(String, f64)> = ship.cargo.drain().collect();
                        for (item_id, amount) in cargo_items {
                            if amount <= 0.0 {
                                continue;
                            }
                            if let Some(entry) = self.in_flight_cargo.get_mut(&(dest_id, item_id.clone())) {
                                *entry = (*entry - amount).max(0.0);
                            }
                            if let Some(item_def) = self.items.get(&item_id) {
                                let sell_price = planet.market.get_price(&item_id, item_def);
                                if let Ok(sold) = planet.market.sell(&item_id, amount, sell_price) {
                                    let revenue = sold * sell_price;
                                    ship.credits += revenue;
                                    ship.total_profit_earned += revenue;

                                    self.event_bus.push(GameEvent::TradeExecuted {
                                        trader_name: ship.name.clone(),
                                        item_name: item_def.name.clone(),
                                        amount: sold as f32,
                                        revenue,
                                        planet_name: planet.name.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            } else {
                // Docked: evaluate routes using Utility AI and Spatial Index!
                let jump_range = 550.0;
                if let Some(route) = TraderAi::select_best_route(
                    ship,
                    archetype,
                    &self.galaxy,
                    &self.spatial,
                    &self.items,
                    jump_range,
                    &self.in_flight_cargo,
                ) {
                    let cur_id = ship.current_planet;
                    if let Some(cur_planet) = self.galaxy.get_planet_mut(cur_id) {
                        if let Ok(bought) = cur_planet.market.buy(&route.item_id, route.quantity, route.buy_price) {
                            let cost = bought * route.buy_price;
                            ship.credits -= cost;
                            *ship.cargo.entry(route.item_id.clone()).or_insert(0.0) += bought;
                            *self.in_flight_cargo.entry((route.target_planet_id, route.item_id)).or_insert(0.0) += bought;
                            ship.destination_planet = Some(route.target_planet_id);
                            ship.travel_progress = 0.0;
                        }
                    }
                }
            }
        }
    }

    fn check_fleet_interceptions(&mut self) {
        let faction_ids: Vec<String> = self.factions.factions.keys().cloned().collect();
        for f_id in &faction_ids {
            let personality = self.get_personality(f_id);
            if let Some(faction) = self.factions.factions.get_mut(f_id) {
                let events = StrategicAi::check_interception_opportunities(
                    f_id,
                    faction,
                    &personality,
                    &mut self.factions.active_fleets,
                    &self.galaxy,
                );
                for ev in events {
                    self.event_bus.push(ev);
                }
            }
        }
    }

    fn update_factions_and_armadas<R: Rng>(&mut self, rng: &mut R) {
        let faction_ids: Vec<String> = self.factions.factions.keys().cloned().collect();

        // 1. Diplomatic turn and fleet movement
        let old_events = self.factions.tick_strategic_turn(rng, &mut self.galaxy);
        for ev_str in old_events {
            // Forward relevant events
            if ev_str.contains("WAR DECLARED") {
                // Already formatted, keep in log
            }
        }

        // 2. High-level strategic targeting via StrategicAi
        for f_id in &faction_ids {
            let personality = self.get_personality(f_id);
            let faction = match self.factions.factions.get(f_id) {
                Some(f) if f.treasury >= 12000.0 && !f.controlled_planets.is_empty() => f.clone(),
                _ => continue,
            };

            let origin_planet = faction.controlled_planets[0];

            // Consider Colonization
            if rng.gen_bool((0.35 * personality.expansionism as f64).clamp(0.1, 0.8)) {
                if let Some((target_id, _score)) = StrategicAi::pick_colonization_target(
                    &faction,
                    &personality,
                    &self.galaxy,
                    &self.spatial,
                ) {
                    let cost = 8000.0;
                    if let Some(f_mut) = self.factions.factions.get_mut(f_id) {
                        f_mut.treasury -= cost;
                    }
                    let fleet = FactionFleet::new(
                        self.factions.next_fleet_id,
                        f_id.clone(),
                        origin_planet,
                        target_id,
                        38.0,
                        FleetMission::Colonization {
                            colony_supplies: 120.0,
                        },
                    );
                    self.factions.next_fleet_id += 1;
                    self.factions.active_fleets.push(fleet);
                    continue;
                }
            }

            // Consider Invasion
            if faction.military_power >= 200.0 && rng.gen_bool((0.40 * personality.aggression as f64).clamp(0.1, 0.8)) {
                if let Some((target_id, _score)) = StrategicAi::pick_invasion_target(
                    &faction,
                    &personality,
                    &self.galaxy,
                    &self.spatial,
                    &self.factions,
                ) {
                    let cost = 14000.0;
                    let firepower = 190.0 * personality.aggression;
                    if let Some(f_mut) = self.factions.factions.get_mut(f_id) {
                        f_mut.treasury -= cost;
                        f_mut.military_power -= 40.0;
                    }
                    let fleet = FactionFleet::new(
                        self.factions.next_fleet_id,
                        f_id.clone(),
                        origin_planet,
                        target_id,
                        32.0,
                        FleetMission::MilitaryInvasion { firepower },
                    );
                    self.factions.next_fleet_id += 1;
                    self.factions.active_fleets.push(fleet);
                }
            }
        }

        // 3. Anti-Hegemony Defensive Coalition check
        let hegemon = faction_ids.iter().find(|id| {
            self.factions.factions.get(*id).map_or(false, |f| f.controlled_planets.len() >= 25)
        }).cloned();

        if let Some(hegemon_id) = hegemon {
            let coalition_candidates: Vec<String> = faction_ids.iter()
                .filter(|id| *id != &hegemon_id)
                .filter(|id| self.factions.factions.get(*id).map_or(false, |f| f.controlled_planets.len() < 20))
                .cloned()
                .collect();

            if coalition_candidates.len() >= 2 && rng.gen_bool(0.25) {
                for i in 0..coalition_candidates.len() {
                    for j in (i+1)..coalition_candidates.len() {
                        let f_a = &coalition_candidates[i];
                        let f_b = &coalition_candidates[j];
                        self.factions.diplomacy.set_relation(f_a, f_b, 80.0, crate::factions::DiplomaticState::Allied);
                    }
                }
                self.event_bus.push(GameEvent::CoalitionFormed {
                    members: coalition_candidates,
                    target_hegemon: hegemon_id,
                });
            }
        }

        // 4. Frontier Piracy Raids
        for f_id in &faction_ids {
            let personality = self.get_personality(f_id);
            let is_pirate = self.factions.factions.get(f_id).map_or(false, |f| {
                f.ideology == crate::factions::Ideology::Pirate || personality.aggression > 0.8
            });
            if is_pirate && rng.gen_bool(0.18) {
                let candidate = self.galaxy.planets.iter_mut().find(|p| {
                    p.zone == crate::galaxy::ZoneType::Frontier
                        && p.owner_faction.as_ref() != Some(f_id)
                        && p.planetary_defense < 100.0
                });
                if let Some(target) = candidate {
                    let loot = (target.planetary_defense * 15.0).clamp(500.0, 3000.0) as f64;
                    target.planetary_defense = (target.planetary_defense - 25.0).max(10.0);
                    if let Some(f_mut) = self.factions.factions.get_mut(f_id) {
                        f_mut.treasury += loot;
                    }
                    self.event_bus.push(GameEvent::PirateRaid {
                        pirate_faction: f_id.clone(),
                        target_planet: target.name.clone(),
                        plunder: loot,
                    });
                }
            }
        }
    }

    pub fn get_personality(&self, faction_id: &str) -> AiPersonality {
        if let Some(faction) = self.factions.factions.get(faction_id) {
            let key = match faction.ideology {
                crate::factions::Ideology::Militarist => "militarist",
                crate::factions::Ideology::Mercantile => "mercantile",
                crate::factions::Ideology::Technocrat => "technocrat",
                crate::factions::Ideology::Pirate => "pirate",
            };
            self.personalities.get(key).cloned().unwrap_or_default()
        } else {
            AiPersonality::default()
        }
    }
}
