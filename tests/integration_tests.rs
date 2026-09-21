use glam::Vec2;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::Path;

use star_core::ai::trader_ai::{TraderAi, TraderArchetype};
use star_core::factions::FactionManager;
use star_core::galaxy::Galaxy;
use star_core::scripting::ScriptEngine;
use star_core::ship::ShipGrid;
use star_core::sim::spatial::SpatialIndex;
use star_core::sim::world::GalacticWorld;
use star_core::traders::TradeFleet;

#[test]
fn test_integration_full_world_simulation() {
    let script_engine = ScriptEngine::new();
    let items = script_engine
        .load_items(Path::new("scripts/items.lua"))
        .expect("Failed to load scripts/items.lua");
    let personalities = script_engine
        .load_personalities(Path::new("scripts/ai_personalities.lua"))
        .unwrap_or_default();

    let mut rng = StdRng::seed_from_u64(12345);
    let mut galaxy = Galaxy::generate_procedural(&mut rng, &items);

    let mut faction_mgr = FactionManager::new();
    faction_mgr.generate_procedural(&mut rng, 4, &mut galaxy);

    let mut trade_fleet = TradeFleet::new();
    trade_fleet.spawn_trader(1, "Merchant Prime", 0, 20000.0, 100.0, 30.0);

    let mut world = GalacticWorld::new(
        galaxy,
        faction_mgr,
        personalities,
        trade_fleet,
        items,
    );

    assert_eq!(world.cycle_count, 1);

    // Simulate 10 full ticks
    for _ in 0..10 {
        world.tick(&mut rng);
    }

    assert_eq!(world.cycle_count, 11);
    assert!(world.galaxy.planets.len() == 100);
}

#[test]
fn test_integration_spatial_index_and_trader_ai() {
    let script_engine = ScriptEngine::new();
    let items = script_engine
        .load_items(Path::new("scripts/items.lua"))
        .expect("Failed to load items.lua");

    let mut rng = StdRng::seed_from_u64(99);
    let galaxy = Galaxy::generate_procedural(&mut rng, &items);

    let positions: Vec<Vec2> = galaxy.planets.iter().map(|p| p.position).collect();
    let spatial = SpatialIndex::build(&positions);

    let trader = star_core::traders::TraderShip {
        id: 1,
        name: "Test Merchant".to_string(),
        current_planet: 0,
        destination_planet: None,
        travel_progress: 0.0,
        speed: 35.0,
        cargo: std::collections::HashMap::new(),
        cargo_capacity: 100.0,
        credits: 50000.0,
        total_profit_earned: 0.0,
    };

    let in_flight = std::collections::HashMap::new();

    let route = TraderAi::select_best_route(
        &trader,
        TraderArchetype::StandardMerchant,
        &galaxy,
        &spatial,
        &items,
        600.0,
        &in_flight,
    );

    if let Some(candidate) = route {
        assert!(candidate.quantity > 0.0);
        assert!(candidate.sell_price > candidate.buy_price);
    }
}

#[test]
fn test_integration_ship_grid_construction() {
    let script_engine = ScriptEngine::new();
    let blocks = script_engine
        .load_blocks(Path::new("scripts/blocks.lua"))
        .expect("Failed to load blocks.lua");

    let mut ship = ShipGrid::new("Flagship", Vec2::new(0.0, 0.0));
    let cockpit = blocks.get("cockpit").unwrap();
    let reactor = blocks.get("fusion_reactor").unwrap();

    ship.place_block(0, 0, cockpit);
    ship.place_block(0, 1, reactor);

    assert_eq!(ship.blocks.len(), 2);
    assert!(ship.total_mass > 0.0);
}
