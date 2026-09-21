use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    WarDeclared {
        attacker: String,
        defender: String,
    },
    PeaceSigned {
        faction_a: String,
        faction_b: String,
        reason: String,
    },
    PlanetColonized {
        faction: String,
        planet_name: String,
        banner: String,
    },
    PlanetConquered {
        conqueror: String,
        defender: String,
        planet_name: String,
        banner: String,
    },
    SiegeRepelled {
        defender_planet: String,
        attacker: String,
    },
    FleetIntercepted {
        defender: String,
        attacker: String,
        target_planet: String,
        defender_won: bool,
    },
    TradeExecuted {
        trader_name: String,
        item_name: String,
        amount: f32,
        revenue: f64,
        planet_name: String,
    },
    CoalitionFormed {
        members: Vec<String>,
        target_hegemon: String,
    },
    PirateRaid {
        pirate_faction: String,
        target_planet: String,
        plunder: f64,
    },
    TacticalBattleResolved {
        attacker_faction: String,
        defender_faction: String,
        location: String,
        attacker_losses: usize,
        defender_losses: usize,
        retreated: Option<String>,
    },
    FortificationConstructed {
        faction: String,
        planet_name: String,
        building_type: String,
    },
    SupplyStarvation {
        faction: String,
        fleet_id: usize,
        location: String,
    },
}

impl GameEvent {
    pub fn to_display_string(&self) -> String {
        match self {
            GameEvent::WarDeclared { attacker, defender } => {
                format!("[WAR DECLARED] {} launched war against {}!", attacker, defender)
            }
            GameEvent::PeaceSigned { faction_a, faction_b, reason } => {
                format!("[PEACE] {} and {} signed ceasefire ({})", faction_a, faction_b, reason)
            }
            GameEvent::PlanetColonized { faction, planet_name, banner } => {
                format!("[COLONIZED] {} settled on '{}' ({})", faction, planet_name, banner)
            }
            GameEvent::PlanetConquered { conqueror, defender, planet_name, banner } => {
                format!("[CONQUEST] {} conquered '{}' from {}! Banner: {}", conqueror, planet_name, defender, banner)
            }
            GameEvent::SiegeRepelled { defender_planet, attacker } => {
                format!("[SIEGE REPELLED] '{}' defenses crushed {}'s assault!", defender_planet, attacker)
            }
            GameEvent::FleetIntercepted { defender, attacker, target_planet, defender_won } => {
                if *defender_won {
                    format!("[INTERCEPTION] {}'s patrol destroyed {}'s armada en route to '{}'!", defender, attacker, target_planet)
                } else {
                    format!("[PATROL BROKEN] {}'s armada broke through {}'s patrol heading to '{}'!", attacker, defender, target_planet)
                }
            }
            GameEvent::TradeExecuted { trader_name, item_name, amount, revenue, planet_name } => {
                format!("[TRADE] {} sold {:.0}x {} at '{}' (+{:.0} cr)", trader_name, amount, item_name, planet_name, revenue)
            }
            GameEvent::CoalitionFormed { members, target_hegemon } => {
                format!("[COALITION] Alliance formed by [{}] against hegemon {}!", members.join(", "), target_hegemon)
            }
            GameEvent::PirateRaid { pirate_faction, target_planet, plunder } => {
                format!("[RAID] {} raided '{}' and plundered {:.0} cr!", pirate_faction, target_planet, plunder)
            }
            GameEvent::TacticalBattleResolved { attacker_faction, defender_faction, location, attacker_losses, defender_losses, retreated } => {
                if let Some(ref ret) = retreated {
                    format!("[TACTICAL BATTLE] {} vs {} at '{}'! Losses: {} vs {}. ({} FTL retreated!)", attacker_faction, defender_faction, location, attacker_losses, defender_losses, ret)
                } else {
                    format!("[TACTICAL BATTLE] {} vs {} at '{}'! Losses: {} vs {}.", attacker_faction, defender_faction, location, attacker_losses, defender_losses)
                }
            }
            GameEvent::FortificationConstructed { faction, planet_name, building_type } => {
                format!("[FORTIFY] {} constructed {} on '{}'!", faction, building_type, planet_name)
            }
            GameEvent::SupplyStarvation { faction, fleet_id, location } => {
                format!("[ATTRITION] {}'s Fleet #{} ran out of supplies near '{}' (-25% combat readiness)!", faction, fleet_id, location)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventBus {
    events: Vec<GameEvent>,
    max_history: usize,
}

impl EventBus {
    pub fn new(max_history: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn push(&mut self, event: GameEvent) {
        if self.events.len() >= self.max_history {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn recent(&self, count: usize) -> &[GameEvent] {
        let start = self.events.len().saturating_sub(count);
        &self.events[start..]
    }

    pub fn recent_strings(&self, count: usize) -> Vec<String> {
        self.recent(count).iter().map(|e| e.to_display_string()).collect()
    }
}
