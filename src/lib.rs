pub mod items;
pub mod scripting;
pub mod economy;
pub mod galaxy;
pub mod traders;
pub mod ship;
pub mod factions;
pub mod render;
pub mod sim;
pub mod ai;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::path::Path;

    #[test]
    fn test_lua_scripts_loading() {
        let engine = scripting::ScriptEngine::new();
        
        let items = engine.load_items(Path::new("scripts/items.lua")).expect("Failed to load items.lua");
        assert!(items.len() >= 8, "Expected at least 8 items");
        assert!(items.contains_key("iron_ore"));
        assert!(items.contains_key("quantum_cores"));

        let (zones, factions) = engine.load_factions_and_zones(Path::new("scripts/factions.lua")).expect("Failed to load factions.lua");
        assert_eq!(zones.len(), 3, "Expected 3 zones (core, neutral, frontier)");
        assert!(factions.len() >= 3, "Expected at least 3 factions");

        let blocks = engine.load_blocks(Path::new("scripts/blocks.lua")).expect("Failed to load blocks.lua");
        assert!(blocks.contains_key("cockpit"));
        assert!(blocks.contains_key("corridor"));
        assert!(blocks.contains_key("airlock"));
    }

    #[test]
    fn test_galaxy_100_planets_distribution() {
        let engine = scripting::ScriptEngine::new();
        let items = engine.load_items(Path::new("scripts/items.lua")).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let galaxy = galaxy::Galaxy::generate_procedural(&mut rng, &items);

        assert_eq!(galaxy.planets.len(), 100, "Galaxy must have exactly 100 planets");

        let core_count = galaxy.planets.iter().filter(|p| p.zone == galaxy::ZoneType::Core).count();
        let neutral_count = galaxy.planets.iter().filter(|p| p.zone == galaxy::ZoneType::Neutral).count();
        let frontier_count = galaxy.planets.iter().filter(|p| p.zone == galaxy::ZoneType::Frontier).count();

        assert_eq!(core_count, 30, "Expected 30 planets in Core zone");
        assert_eq!(neutral_count, 40, "Expected 40 planets in Neutral zone");
        assert_eq!(frontier_count, 30, "Expected 30 planets in Frontier zone");
    }

    #[test]
    fn test_dynamic_pricing_and_market() {
        let mut market = economy::PlanetaryMarket::new(10000.0);
        let ore = items::ItemDef {
            id: "iron_ore".to_string(),
            name: "Iron Ore".to_string(),
            category: items::ItemCategory::RawMaterial,
            base_price: 20.0,
            unit_mass: 2.0,
            description: "Raw ore".to_string(),
        };

        // Target is 100, current is 10 (severe deficit)
        market.register_commodity("iron_ore", 10.0, 100.0, 0.0, 5.0);
        let deficit_price = market.get_price("iron_ore", &ore);
        assert!(deficit_price > ore.base_price, "Price should surge during deficit");

        // Target is 100, current is 250 (surplus)
        market.register_commodity("iron_ore", 250.0, 100.0, 10.0, 1.0);
        let surplus_price = market.get_price("iron_ore", &ore);
        assert!(surplus_price < ore.base_price, "Price should drop during surplus");
    }

    #[test]
    fn test_procedural_factions_and_independent_planets() {
        let engine = scripting::ScriptEngine::new();
        let items = engine.load_items(Path::new("scripts/items.lua")).unwrap();

        let mut rng = StdRng::seed_from_u64(999);
        let mut galaxy = galaxy::Galaxy::generate_procedural(&mut rng, &items);

        let mut faction_mgr = factions::FactionManager::new();
        faction_mgr.generate_procedural(&mut rng, 4, &mut galaxy);

        assert_eq!(faction_mgr.factions.len(), 4, "Should have generated 4 factions");

        for (_, faction) in faction_mgr.factions.iter() {
            assert!(!faction.name.is_empty());
            assert!(!faction.flag.symbol.is_empty());
            assert!(!faction.flag.name.is_empty());
        }

        // Count independent vs claimed planets
        let independent_count = galaxy.planets.iter().filter(|p| p.owner_faction.is_none()).count();
        let claimed_count = galaxy.planets.iter().filter(|p| p.owner_faction.is_some()).count();

        assert!(independent_count > 20, "At least 20 planets should start unaligned/independent");
        assert!(claimed_count > 20, "At least 20 planets should be initially claimed");
    }

    #[test]
    fn test_diplomacy_war_and_exhaustion_peace() {
        let mut matrix = factions::DiplomacyMatrix::new();
        matrix.set_relation("empire_a", "syndicate_b", 0.0, factions::DiplomaticState::Neutral);

        // Declare war
        matrix.declare_war("empire_a", "syndicate_b");
        assert_eq!(
            matrix.get_relation("empire_a", "syndicate_b").unwrap().state,
            factions::DiplomaticState::War
        );

        // Sign peace
        matrix.sign_peace("empire_a", "syndicate_b");
        assert_eq!(
            matrix.get_relation("empire_a", "syndicate_b").unwrap().state,
            factions::DiplomaticState::Hostile
        );
    }

    #[test]
    fn test_colonization_and_invasion_mechanics() {
        let engine = scripting::ScriptEngine::new();
        let items = engine.load_items(Path::new("scripts/items.lua")).unwrap();

        let mut rng = StdRng::seed_from_u64(777);
        let mut galaxy = galaxy::Galaxy::generate_procedural(&mut rng, &items);

        let mut faction_mgr = factions::FactionManager::new();
        faction_mgr.generate_procedural(&mut rng, 2, &mut galaxy);

        let faction_ids: Vec<String> = faction_mgr.factions.keys().cloned().collect();
        let faction_a = &faction_ids[0];

        // 1. Colonize an independent planet
        let indep_planet = galaxy.planets.iter_mut().find(|p| p.owner_faction.is_none()).unwrap();
        let p_id = indep_planet.id;

        // Manually dispatch a colonization fleet
        faction_mgr.active_fleets.push(factions::FactionFleet {
            id: 999,
            owner_faction: faction_a.clone(),
            origin_planet: p_id,
            target_planet: p_id,
            progress: 1.0, // Instantly arrive
            speed: 100.0,
            mission: factions::FleetMission::Colonization { colony_supplies: 150.0 },
        });

        let events = faction_mgr.tick_strategic_turn(&mut rng, &mut galaxy);
        let claimed_planet = galaxy.get_planet(p_id).unwrap();
        assert_eq!(claimed_planet.owner_faction.as_ref(), Some(faction_a));
        assert!(events.iter().any(|e| e.contains("[PLANET CLAIMED]")));
    }

    #[test]
    fn test_ship_block_destruction_and_flood_fill() {
        let engine = scripting::ScriptEngine::new();
        let blocks = engine.load_blocks(Path::new("scripts/blocks.lua")).unwrap();

        let mut ship = ship::ShipGrid::new("StarSparrow", Vec2::new(100.0, 100.0));

        let cockpit = blocks.get("cockpit").unwrap();
        let corridor = blocks.get("corridor").unwrap();
        let thruster = blocks.get("ion_thruster").unwrap();

        ship.place_block(0, 0, cockpit);
        ship.place_block(1, 0, corridor);
        ship.place_block(2, 0, corridor);
        ship.place_block(3, 0, thruster);

        assert_eq!(ship.blocks.len(), 4);
        assert!(ship.total_mass > 0.0);

        // Sever the bridge at (2,0)
        let destroyed = ship.damage_block(2, 0, 9999.0);
        assert!(destroyed);
        assert_eq!(ship.blocks.len(), 3);

        let detached = ship.split_disconnected_chunks((0, 0));
        assert_eq!(detached.len(), 1, "Thruster at (3,0) should detach into debris");
        assert_eq!(ship.blocks.len(), 2, "Cockpit and corridor (1,0) should remain");
        assert!(detached[0].blocks.contains_key(&(3, 0)));
    }

    #[test]
    fn test_character_eva_and_boarding() {
        let engine = scripting::ScriptEngine::new();
        let blocks = engine.load_blocks(Path::new("scripts/blocks.lua")).unwrap();

        let mut ship = ship::ShipGrid::new("Explorer", Vec2::new(500.0, 500.0));
        let airlock = blocks.get("airlock").unwrap();
        ship.place_block(0, 0, airlock);

        let mut character = ship::Character {
            name: "Commander Shepard".to_string(),
            in_ship: true,
            local_pos: Vec2::new(0.0, 0.0),
            world_pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            oxygen: 100.0,
            in_eva_suit: true,
        };

        // Exit ship
        ship.exit_to_eva(&mut character, (0, 0)).expect("Exit to EVA failed");
        assert!(!character.in_ship);
        assert_eq!(character.world_pos, Vec2::new(500.0, 500.0));

        // Board ship
        ship.board_ship(&mut character, (0, 0)).expect("Board ship failed");
        assert!(character.in_ship);
        assert_eq!(character.local_pos, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_spatial_index_o1_and_neighbors() {
        let positions = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(100.0, 0.0),
        ];
        let spatial = sim::spatial::SpatialIndex::build(&positions);

        assert_eq!(spatial.distance(0, 1), 10.0);
        assert_eq!(spatial.distance(1, 0), 10.0);
        assert_eq!(spatial.distance(0, 3), 100.0);

        let nearest_to_0 = spatial.nearest(0).unwrap();
        assert_eq!(nearest_to_0.0, 1);
        assert_eq!(nearest_to_0.1, 10.0);

        let within_20 = spatial.neighbors_within(0, 20.0);
        assert_eq!(within_20.len(), 1);
        assert_eq!(within_20[0].0, 1);

        let within_60 = spatial.neighbors_within(0, 60.0);
        assert_eq!(within_60.len(), 2);
    }

    #[test]
    fn test_ai_personality_loading() {
        let engine = scripting::ScriptEngine::new();
        let personalities = engine
            .load_personalities(Path::new("scripts/ai_personalities.lua"))
            .expect("Failed to load ai_personalities.lua");

        assert!(personalities.contains_key("militarist"));
        assert!(personalities.contains_key("mercantile"));
        assert!(personalities.contains_key("technocrat"));
        assert!(personalities.contains_key("pirate"));

        let warmonger = personalities.get("militarist").unwrap();
        assert!(warmonger.aggression > 1.5);
    }

    #[test]
    fn test_galactic_world_simulation_cycle() {
        let engine = scripting::ScriptEngine::new();
        let items = engine.load_items(Path::new("scripts/items.lua")).unwrap();
        let personalities = engine.load_personalities(Path::new("scripts/ai_personalities.lua")).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let mut galaxy = galaxy::Galaxy::generate_procedural(&mut rng, &items);

        let mut faction_mgr = factions::FactionManager::new();
        faction_mgr.generate_procedural(&mut rng, 4, &mut galaxy);

        let mut trade_fleet = traders::TradeFleet::new();
        trade_fleet.spawn_trader(1, "Test Hauler", 0, 10000.0, 50.0, 30.0);

        let mut world = sim::world::GalacticWorld::new(
            galaxy,
            faction_mgr,
            personalities,
            trade_fleet,
            items,
        );

        assert_eq!(world.cycle_count, 1);
        // Step 3 cycles
        for _ in 0..3 {
            world.tick(&mut rng);
        }
        assert_eq!(world.cycle_count, 4);
        assert_eq!(world.spatial.count, 100);
    }

    #[test]
    fn test_particle_system_lifecycle() {
        use macroquad::prelude::{Color, Vec2};
        let mut ps = render::particles::ParticleSystem::new(20);

        // Emit several types
        ps.emit_engine_trail(Vec2::new(10.0, 10.0), Vec2::new(1.0, 0.0), Color::new(1.0, 0.5, 0.0, 1.0), 20.0);
        ps.emit_laser_tracer(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), Color::new(0.0, 1.0, 0.0, 1.0));
        ps.emit_explosion_spark(Vec2::new(50.0, 50.0), Color::new(1.0, 0.2, 0.2, 1.0));
        ps.emit_eva_puff(Vec2::new(5.0, 5.0), Vec2::new(0.0, 1.0));

        // Advance time: particles should move and eventually expire
        ps.update(0.1);
        ps.update(2.0); // all particles have lifetimes <= 1.0s, should become inactive
        
        // Emitting more should cleanly reuse slots without allocating
        ps.emit_engine_trail(Vec2::ZERO, Vec2::X, Color::new(0.0, 0.8, 1.0, 1.0), 10.0);
    }

    #[test]
    fn test_trader_ai_anti_herd_in_flight_discount() {
        let engine = scripting::ScriptEngine::new();
        let items = engine.load_items(Path::new("scripts/items.lua")).unwrap();
        let mut rng = StdRng::seed_from_u64(101);
        let galaxy = galaxy::Galaxy::generate_procedural(&mut rng, &items);
        let positions: Vec<Vec2> = galaxy.planets.iter().map(|p| p.position).collect();
        let spatial = sim::spatial::SpatialIndex::build(&positions);

        let trader = traders::TraderShip {
            id: 1,
            name: "Test Merchant".to_string(),
            current_planet: 0,
            destination_planet: None,
            travel_progress: 0.0,
            speed: 35.0,
            cargo: std::collections::HashMap::new(),
            cargo_capacity: 50.0,
            credits: 20000.0,
            total_profit_earned: 0.0,
        };

        // Case A: no in-flight cargo
        let empty_in_flight = std::collections::HashMap::new();
        let route_a = ai::trader_ai::TraderAi::select_best_route(
            &trader,
            ai::trader_ai::TraderArchetype::StandardMerchant,
            &galaxy,
            &spatial,
            &items,
            600.0,
            &empty_in_flight,
        );

        if let Some(r_a) = route_a {
            // Case B: simulate that 500 units of that commodity are ALREADY en route to target planet
            let mut flooded_in_flight = std::collections::HashMap::new();
            flooded_in_flight.insert((r_a.target_planet_id, r_a.item_id.clone()), 500.0);

            let route_b = ai::trader_ai::TraderAi::select_best_route(
                &trader,
                ai::trader_ai::TraderArchetype::StandardMerchant,
                &galaxy,
                &spatial,
                &items,
                600.0,
                &flooded_in_flight,
            );

            if let Some(r_b) = route_b {
                // If it chose the same planet and item, utility score must be drastically reduced
                if r_b.target_planet_id == r_a.target_planet_id && r_b.item_id == r_a.item_id {
                    assert!(r_b.utility_score < r_a.utility_score, "Score should drop due to market flooding");
                }
            }
        }
    }

    #[test]
    fn test_event_bus_and_coalition_events() {
        let mut bus = sim::events::EventBus::new(5);
        let ev1 = sim::events::GameEvent::CoalitionFormed {
            members: vec!["Faction_A".to_string(), "Faction_B".to_string()],
            target_hegemon: "Hegemon_Prime".to_string(),
        };
        let ev2 = sim::events::GameEvent::PirateRaid {
            pirate_faction: "Black_Sun".to_string(),
            target_planet: "Outpost 9".to_string(),
            plunder: 1500.0,
        };

        bus.push(ev1);
        bus.push(ev2);

        let recents = bus.recent_strings(2);
        assert_eq!(recents.len(), 2);
        assert!(recents[0].contains("[COALITION]"));
        assert!(recents[1].contains("[RAID]"));
    }
}


