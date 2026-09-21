use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::items::ItemDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketItem {
    pub current_stock: f64,
    pub target_stock: f64,
    pub production_rate: f64,   // units produced per economic tick
    pub consumption_rate: f64,  // units consumed per economic tick
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryMarket {
    pub credits: f64,
    pub inventory: HashMap<String, MarketItem>,
}

impl PlanetaryMarket {
    pub fn new(initial_credits: f64) -> Self {
        Self {
            credits: initial_credits,
            inventory: HashMap::new(),
        }
    }

    pub fn register_commodity(
        &mut self,
        item_id: &str,
        stock: f64,
        target: f64,
        prod: f64,
        cons: f64,
    ) {
        self.inventory.insert(
            item_id.to_string(),
            MarketItem {
                current_stock: stock,
                target_stock: target,
                production_rate: prod,
                consumption_rate: cons,
            },
        );
    }

    /// Calculate dynamic price based on supply and demand equilibrium
    pub fn get_price(&self, item_id: &str, item_def: &ItemDef) -> f64 {
        if let Some(entry) = self.inventory.get(item_id) {
            let target = entry.target_stock.max(1.0);
            let current = entry.current_stock.max(0.0);
            
            // Deficit -> Price rises up to 4x base price
            // Surplus -> Price drops down to 0.3x base price
            let ratio = (target - current) / (target + 10.0);
            let multiplier = (1.0 + ratio * 1.5).clamp(0.25, 4.0);
            
            (item_def.base_price * multiplier * 100.0).round() / 100.0
        } else {
            item_def.base_price
        }
    }

    /// Economic cycle tick: produce and consume goods
    pub fn tick(&mut self) {
        for (_item_id, item) in self.inventory.iter_mut() {
            item.current_stock += item.production_rate;
            item.current_stock -= item.consumption_rate;
            if item.current_stock < 0.0 {
                item.current_stock = 0.0;
            }
        }
    }

    /// Trader or player buys goods from this planet
    pub fn buy(&mut self, item_id: &str, amount: f64, price: f64) -> Result<f64, String> {
        let entry = self
            .inventory
            .get_mut(item_id)
            .ok_or_else(|| format!("Item {} not traded on this planet", item_id))?;

        let actual_amount = amount.min(entry.current_stock);
        if actual_amount <= 0.0 {
            return Err("Not enough stock available".to_string());
        }

        entry.current_stock -= actual_amount;
        let total_cost = actual_amount * price;
        self.credits += total_cost;
        Ok(actual_amount)
    }

    /// Trader or player sells goods to this planet
    pub fn sell(&mut self, item_id: &str, amount: f64, price: f64) -> Result<f64, String> {
        let total_cost = amount * price;
        if self.credits < total_cost {
            let affordable = (self.credits / price).floor();
            if affordable <= 0.0 {
                return Err("Planet market lacks credits to purchase this cargo".to_string());
            }
            return self.sell(item_id, affordable, price);
        }

        let entry = self
            .inventory
            .get_mut(item_id)
            .ok_or_else(|| format!("Item {} not accepted on this planet", item_id))?;

        entry.current_stock += amount;
        self.credits -= total_cost;
        Ok(amount)
    }
}
