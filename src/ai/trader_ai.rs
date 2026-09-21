use std::collections::HashMap;
use crate::galaxy::{Galaxy, ZoneType};
use crate::items::ItemDef;
use crate::sim::spatial::SpatialIndex;
use crate::traders::TraderShip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraderArchetype {
    StandardMerchant,
    Smuggler,
    BulkHauler,
}

pub struct RouteCandidate {
    pub target_planet_id: usize,
    pub item_id: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub quantity: f64,
    pub utility_score: f32,
}

pub struct TraderAi;

impl TraderAi {
    /// Evaluates all feasible trade routes from `current_planet` within `jump_range`
    /// using the O(1) SpatialIndex, cognitive brain threat awareness, and multi-factor Utility scoring.
    pub fn select_best_route(
        trader: &TraderShip,
        archetype: TraderArchetype,
        galaxy: &Galaxy,
        spatial: &SpatialIndex,
        items: &HashMap<String, ItemDef>,
        jump_range: f32,
        in_flight: &HashMap<(usize, String), f64>,
        brain: Option<&crate::ai::brain::AiBrain>,
    ) -> Option<RouteCandidate> {
        let cur_id = trader.current_planet;
        let cur_planet = galaxy.get_planet(cur_id)?;

        // Instant neighbor query using spatial index!
        let neighbors = spatial.neighbors_within(cur_id, jump_range);
        let mut best_candidate: Option<RouteCandidate> = None;
        let mut highest_score = 0.0f32;

        for &(dest_id, dist) in neighbors {
            if dest_id == cur_id {
                continue;
            }
            let dest_planet = match galaxy.get_planet(dest_id) {
                Some(p) => p,
                None => continue,
            };

            // Calculate zone danger & war risk + brain system danger perception
            let base_risk: f32 = match dest_planet.zone {
                ZoneType::Core => 0.05,
                ZoneType::Neutral => 0.20,
                ZoneType::Frontier => 0.50,
            };
            let danger_memory = brain.map_or(0.0, |b| b.get_danger(dest_id));
            let war_risk: f32 = if dest_planet.planetary_defense < dest_planet.max_defense { 0.35 } else { 0.0 };
            let total_risk: f32 = base_risk + war_risk + (danger_memory * 0.2);

            // Risk penalty according to archetype
            let risk_multiplier: f32 = match archetype {
                TraderArchetype::StandardMerchant => (1.0f32 - total_risk).max(0.1f32),
                TraderArchetype::BulkHauler => (1.0f32 - total_risk * 1.2f32).max(0.05f32),
                TraderArchetype::Smuggler => {
                    // Smugglers embrace risk if price margins are high!
                    1.0f32 + total_risk * 0.3f32
                }
            };

            for (item_id, item_def) in items {
                // Bulk haulers favor heavy industrial/raw materials
                if archetype == TraderArchetype::BulkHauler
                    && item_def.category != crate::items::ItemCategory::RawMaterial
                    && item_def.category != crate::items::ItemCategory::Industrial
                {
                    continue;
                }

                let buy_price = cur_planet.market.get_price(item_id, item_def);

                // Anti-herd behavior: account for cargo already en route to this destination!
                let incoming = in_flight.get(&(dest_id, item_id.clone())).copied().unwrap_or(0.0);
                let cur_stock = dest_planet.market.inventory.get(item_id).map(|i| i.current_stock).unwrap_or(0.0);
                let target_stock = dest_planet.market.inventory.get(item_id).map(|i| i.target_stock).unwrap_or(10.0);
                let effective_stock = cur_stock + incoming;
                let ratio = (target_stock - effective_stock) / (target_stock + 10.0);
                let mult = (1.0 + ratio * 1.5).clamp(0.25, 4.0);
                let sell_price = (item_def.base_price * mult * 100.0).round() / 100.0;

                let min_margin_ratio = match archetype {
                    TraderArchetype::Smuggler => 1.30,
                    _ => 1.15,
                };

                if sell_price <= buy_price * min_margin_ratio {
                    continue;
                }

                let available_stock = cur_planet
                    .market
                    .inventory
                    .get(item_id)
                    .map(|i| i.current_stock)
                    .unwrap_or(0.0);

                if available_stock < 3.0 {
                    continue;
                }

                let max_units_by_weight = trader.cargo_capacity / item_def.unit_mass.max(0.1);
                let max_units_by_credits = (trader.credits * 0.9) / buy_price.max(1.0);
                let quantity = available_stock.min(max_units_by_weight).min(max_units_by_credits);

                if quantity < 1.0 {
                    continue;
                }

                let net_profit = (quantity * (sell_price - buy_price)) as f32;
                // Utility: Profit per distance unit adjusted by risk
                let utility = (net_profit / (dist + 25.0)) * risk_multiplier;

                if utility > highest_score {
                    highest_score = utility;
                    best_candidate = Some(RouteCandidate {
                        target_planet_id: dest_id,
                        item_id: item_id.clone(),
                        buy_price,
                        sell_price,
                        quantity,
                        utility_score: utility,
                    });
                }
            }
        }

        best_candidate
    }
}
