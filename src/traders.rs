use std::collections::HashMap;
use crate::galaxy::Galaxy;
use crate::items::ItemDef;

#[derive(Debug, Clone)]
pub struct TraderShip {
    pub id: u64,
    pub name: String,
    pub current_planet: usize,
    pub destination_planet: Option<usize>,
    pub travel_progress: f32, // 0.0 to 1.0
    pub speed: f32,
    pub cargo: HashMap<String, f64>,
    pub cargo_capacity: f64,
    pub credits: f64,
    pub total_profit_earned: f64,
}

pub struct TradeFleet {
    pub ships: Vec<TraderShip>,
}

impl TradeFleet {
    pub fn new() -> Self {
        Self { ships: Vec::new() }
    }

    pub fn spawn_trader(
        &mut self,
        id: u64,
        name: &str,
        start_planet: usize,
        credits: f64,
        cargo_capacity: f64,
        speed: f32,
    ) {
        self.ships.push(TraderShip {
            id,
            name: name.to_string(),
            current_planet: start_planet,
            destination_planet: None,
            travel_progress: 0.0,
            speed,
            cargo: HashMap::new(),
            cargo_capacity,
            credits,
            total_profit_earned: 0.0,
        });
    }

    /// Update all traders for one tick
    pub fn update(
        &mut self,
        galaxy: &mut Galaxy,
        items: &HashMap<String, ItemDef>,
    ) -> Vec<String> {
        let mut log_events = Vec::new();

        for ship in self.ships.iter_mut() {
            if let Some(dest_id) = ship.destination_planet {
                let dist = galaxy.distance(ship.current_planet, dest_id).max(1.0);
                let step = ship.speed / dist;
                ship.travel_progress += step;

                if ship.travel_progress >= 1.0 {
                    // Arrived at destination!
                    ship.travel_progress = 0.0;
                    ship.current_planet = dest_id;
                    ship.destination_planet = None;

                    // Sell all cargo to this planet's market
                    let mut earnings = 0.0;
                    if let Some(planet) = galaxy.get_planet_mut(dest_id) {
                        let cargo_items: Vec<(String, f64)> = ship
                            .cargo
                            .drain()
                            .filter(|(_, amt)| *amt > 0.0)
                            .collect();

                        for (item_id, amount) in cargo_items {
                            if let Some(item_def) = items.get(&item_id) {
                                let sell_price = planet.market.get_price(&item_id, item_def);
                                if let Ok(sold) = planet.market.sell(&item_id, amount, sell_price) {
                                    let revenue = sold * sell_price;
                                    ship.credits += revenue;
                                    earnings += revenue;
                                    log_events.push(format!(
                                        "[{}] Arrived at {} and sold {:.1}x {} for {:.0} credits (price {:.1})",
                                        ship.name, planet.name, sold, item_def.name, revenue, sell_price
                                    ));
                                }
                            }
                        }
                    }
                    ship.total_profit_earned += earnings;
                }
            } else {
                // Ship is docked at `ship.current_planet`. Find best trade route.
                let cur_id = ship.current_planet;
                let cur_planet = match galaxy.get_planet(cur_id) {
                    Some(p) => p,
                    None => continue,
                };

                // Find most lucrative opportunity:
                // Scan planets within jump range (up to 500 units)
                let mut best_target: Option<(usize, String, f64, f64)> = None; // (dest_id, item_id, profit_per_unit, buy_price)
                let mut highest_total_profit = 0.0;

                for candidate in galaxy.planets.iter() {
                    if candidate.id == cur_id {
                        continue;
                    }
                    let dist = galaxy.distance(cur_id, candidate.id);
                    if dist > 600.0 {
                        continue;
                    }

                    for (item_id, item_def) in items.iter() {
                        let buy_price = cur_planet.market.get_price(item_id, item_def);
                        let sell_price = candidate.market.get_price(item_id, item_def);

                        if sell_price > buy_price * 1.15 {
                            // Check available stock at current planet
                            let available_stock = cur_planet
                                .market
                                .inventory
                                .get(item_id)
                                .map(|i| i.current_stock)
                                .unwrap_or(0.0);

                            if available_stock < 5.0 {
                                continue;
                            }

                            let max_units_by_weight = ship.cargo_capacity / item_def.unit_mass.max(0.1);
                            let max_units_by_credits = ship.credits / buy_price.max(1.0);
                            let units = available_stock
                                .min(max_units_by_weight)
                                .min(max_units_by_credits);

                            if units >= 2.0 {
                                let total_profit = units * (sell_price - buy_price);
                                if total_profit > highest_total_profit {
                                    highest_total_profit = total_profit;
                                    best_target = Some((candidate.id, item_id.clone(), sell_price - buy_price, buy_price));
                                }
                            }
                        }
                    }
                }

                if let Some((dest_id, item_id, _margin, buy_price)) = best_target {
                    let dest_name = galaxy.get_planet(dest_id).map(|p| p.name.clone()).unwrap_or_default();
                    let cur_name = cur_planet.name.clone();

                    // Buy goods
                    let item_def = items.get(&item_id).unwrap();
                    let max_units_by_weight = ship.cargo_capacity / item_def.unit_mass.max(0.1);
                    let max_units_by_credits = (ship.credits * 0.9) / buy_price.max(1.0); // Keep 10% reserve
                    let target_amount = max_units_by_weight.min(max_units_by_credits);

                    if let Some(cur_mut) = galaxy.get_planet_mut(cur_id) {
                        if let Ok(bought) = cur_mut.market.buy(&item_id, target_amount, buy_price) {
                            let cost = bought * buy_price;
                            ship.credits -= cost;
                            *ship.cargo.entry(item_id.clone()).or_insert(0.0) += bought;
                            ship.destination_planet = Some(dest_id);
                            ship.travel_progress = 0.0;

                            log_events.push(format!(
                                "[{}] Bought {:.1}x {} at {} for {:.0} cr -> Heading to {}",
                                ship.name, bought, item_def.name, cur_name, cost, dest_name
                            ));
                        }
                    }
                }
            }
        }

        log_events
    }
}
