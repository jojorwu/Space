use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::ai::personality::AiPersonality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEvent {
    CombatLoss {
        planet_id: usize,
        lost_ships: usize,
        enemy_faction: String,
        timestamp: usize,
    },
    CombatVictory {
        planet_id: usize,
        defeated_faction: String,
        timestamp: usize,
    },
    TradeProfit {
        planet_id: usize,
        item_id: String,
        profit: f64,
        timestamp: usize,
    },
    Betrayal {
        offending_faction: String,
        severity: f32,
        timestamp: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBrain {
    pub faction_id: String,
    pub memories: Vec<MemoryEvent>,
    pub grudges: HashMap<String, f32>,
    pub system_danger: HashMap<usize, f32>,
    pub dynamic_risk_modifier: f32,
    pub dynamic_aggression_modifier: f32,
    pub dynamic_defense_modifier: f32,
    pub memory_capacity: usize,
}

impl AiBrain {
    pub fn new(faction_id: &str) -> Self {
        Self {
            faction_id: faction_id.to_string(),
            memories: Vec::new(),
            grudges: HashMap::new(),
            system_danger: HashMap::new(),
            dynamic_risk_modifier: 1.0,
            dynamic_aggression_modifier: 1.0,
            dynamic_defense_modifier: 1.0,
            memory_capacity: 50,
        }
    }

    /// Records a combat defeat, increasing threat awareness, defensive bias, and grudge against enemy
    pub fn record_combat_loss(
        &mut self,
        planet_id: usize,
        lost_ships: usize,
        enemy_faction: &str,
        cycle: usize,
    ) {
        self.memories.push(MemoryEvent::CombatLoss {
            planet_id,
            lost_ships,
            enemy_faction: enemy_faction.to_string(),
            timestamp: cycle,
        });

        // Increase danger score for this planet
        let current_danger = self.system_danger.entry(planet_id).or_insert(0.0);
        *current_danger = (*current_danger + 0.35 + (lost_ships as f32 * 0.1)).min(2.5);

        // Increase grudge against enemy faction
        let grudge = self.grudges.entry(enemy_faction.to_string()).or_insert(0.0);
        *grudge = (*grudge + 15.0 + (lost_ships as f32 * 3.0)).min(100.0);

        // Shift emotional posture towards caution and defensive bias
        self.dynamic_risk_modifier = (self.dynamic_risk_modifier * 0.88).max(0.4);
        self.dynamic_defense_modifier = (self.dynamic_defense_modifier * 1.15).min(2.0);

        self.trim_memory();
    }

    /// Records a victory, boosting confidence and aggressive posture
    pub fn record_combat_victory(
        &mut self,
        planet_id: usize,
        defeated_faction: &str,
        cycle: usize,
    ) {
        self.memories.push(MemoryEvent::CombatVictory {
            planet_id,
            defeated_faction: defeated_faction.to_string(),
            timestamp: cycle,
        });

        // System is now safer / controlled
        if let Some(danger) = self.system_danger.get_mut(&planet_id) {
            *danger = (*danger * 0.5).max(0.0);
        }

        // Slight satisfaction reduces grudge
        if let Some(grudge) = self.grudges.get_mut(defeated_faction) {
            *grudge = (*grudge - 10.0).max(0.0);
        }

        // Boost confidence and aggression
        self.dynamic_risk_modifier = (self.dynamic_risk_modifier * 1.08).min(1.8);
        self.dynamic_aggression_modifier = (self.dynamic_aggression_modifier * 1.10).min(2.0);

        self.trim_memory();
    }

    /// Records a lucrative trade transaction
    pub fn record_trade_profit(&mut self, planet_id: usize, item_id: &str, profit: f64, cycle: usize) {
        self.memories.push(MemoryEvent::TradeProfit {
            planet_id,
            item_id: item_id.to_string(),
            profit,
            timestamp: cycle,
        });
        self.trim_memory();
    }

    /// Records diplomatic betrayal or war declaration
    pub fn record_betrayal(&mut self, offending_faction: &str, severity: f32, cycle: usize) {
        self.memories.push(MemoryEvent::Betrayal {
            offending_faction: offending_faction.to_string(),
            severity,
            timestamp: cycle,
        });

        let grudge = self.grudges.entry(offending_faction.to_string()).or_insert(0.0);
        *grudge = (*grudge + severity * 25.0).min(100.0);

        self.dynamic_aggression_modifier = (self.dynamic_aggression_modifier * 1.25).min(2.5);

        self.trim_memory();
    }

    /// Decays memories and normalizes emotional modifiers over time
    pub fn decay(&mut self, current_cycle: usize) {
        // Decay grudges slowly
        for grudge in self.grudges.values_mut() {
            *grudge = (*grudge * 0.96) - 0.2;
            if *grudge < 0.0 {
                *grudge = 0.0;
            }
        }

        // Decay danger levels
        for danger in self.system_danger.values_mut() {
            *danger *= 0.95;
        }

        // Gradually decay dynamic modifiers back towards 1.0 baseline
        self.dynamic_risk_modifier += (1.0 - self.dynamic_risk_modifier) * 0.05;
        self.dynamic_aggression_modifier += (1.0 - self.dynamic_aggression_modifier) * 0.05;
        self.dynamic_defense_modifier += (1.0 - self.dynamic_defense_modifier) * 0.05;

        // Retain memories only within last 100 cycles
        self.memories.retain(|m| {
            let ts = match m {
                MemoryEvent::CombatLoss { timestamp, .. } => *timestamp,
                MemoryEvent::CombatVictory { timestamp, .. } => *timestamp,
                MemoryEvent::TradeProfit { timestamp, .. } => *timestamp,
                MemoryEvent::Betrayal { timestamp, .. } => *timestamp,
            };
            current_cycle.saturating_sub(ts) <= 100
        });
    }

    fn trim_memory(&mut self) {
        if self.memories.len() > self.memory_capacity {
            let drain_count = self.memories.len() - self.memory_capacity;
            self.memories.drain(0..drain_count);
        }
    }

    /// Calculates effective aggression blending baseline personality with brain modifiers and grudges against target
    pub fn effective_aggression(&self, base: &AiPersonality, target_faction: Option<&str>) -> f32 {
        let grudge_bonus = if let Some(target) = target_faction {
            self.get_grudge(target) / 50.0 // Grudge adds up to +2.0 aggression
        } else {
            0.0
        };

        (base.aggression * self.dynamic_aggression_modifier + grudge_bonus).clamp(0.1, 3.5)
    }

    /// Calculates effective risk tolerance
    pub fn effective_risk_tolerance(&self, base: &AiPersonality) -> f32 {
        (base.risk_tolerance * self.dynamic_risk_modifier).clamp(0.1, 2.5)
    }

    /// Calculates effective defense bias
    pub fn effective_defense_bias(&self, base: &AiPersonality) -> f32 {
        (base.defense_bias * self.dynamic_defense_modifier).clamp(0.1, 3.0)
    }

    pub fn get_grudge(&self, faction_id: &str) -> f32 {
        self.grudges.get(faction_id).copied().unwrap_or(0.0)
    }

    pub fn get_danger(&self, planet_id: usize) -> f32 {
        self.system_danger.get(&planet_id).copied().unwrap_or(0.0)
    }
}
