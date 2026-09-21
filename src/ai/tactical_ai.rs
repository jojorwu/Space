use crate::ai::personality::AiPersonality;
use crate::sim::combat::{CombatDoctrine, ShipClass, TacticalFleet, TargetSystem};

pub struct TacticalAi;

impl TacticalAi {
    /// Selects the most optimal combat doctrine based on faction personality and tactical role
    pub fn choose_combat_doctrine(personality: &AiPersonality, fleet_role: &str) -> CombatDoctrine {
        if fleet_role == "Raider" {
            return CombatDoctrine::HitAndRun;
        }

        if fleet_role == "Escort" || personality.defense_bias >= 1.35 {
            return CombatDoctrine::ScreenEscort;
        }

        if personality.aggression >= 1.45 && personality.risk_tolerance >= 1.20 {
            CombatDoctrine::BrawlingAssault
        } else if personality.risk_tolerance <= 0.85 {
            CombatDoctrine::KitingSniper
        } else {
            CombatDoctrine::ScreenEscort
        }
    }

    /// Chooses subsystem target priority based on enemy doctrine and current battlefield conditions
    pub fn choose_target_subsystem(
        _my_doctrine: CombatDoctrine,
        enemy_doctrine: CombatDoctrine,
        enemy_health_ratio: f32,
    ) -> TargetSystem {
        // If enemy is prone to fleeing, shoot out their thrusters first!
        if enemy_doctrine == CombatDoctrine::HitAndRun || enemy_doctrine == CombatDoctrine::KitingSniper {
            return TargetSystem::Engines;
        }

        // If enemy has breached armor and hull, aim for reactor detonation
        if enemy_health_ratio <= 0.35 {
            return TargetSystem::Reactor;
        }

        // If facing heavy brawler, disable their heavy weapon batteries
        if enemy_doctrine == CombatDoctrine::BrawlingAssault {
            return TargetSystem::Weapons;
        }

        TargetSystem::GeneralHull
    }

    /// Assembles an optimized task force composition based on role and firepower rating
    pub fn assemble_task_force(
        faction_id: &str,
        role: &str,
        firepower: f32,
        doctrine: CombatDoctrine,
    ) -> TacticalFleet {
        let mut fleet = TacticalFleet::new(faction_id, doctrine);
        let prefix = match faction_id {
            s if s.len() >= 3 => &s[..3],
            _ => "FLT",
        };

        match role {
            "SiegeArmada" => {
                // Heavy line of battleships, cruisers, and frigate screens
                let bb_count = (firepower / 350.0).floor().max(1.0) as usize;
                let ca_count = (firepower / 180.0).floor().max(1.0) as usize;
                let ff_count = (firepower / 90.0).floor().max(2.0) as usize;

                for i in 1..=bb_count {
                    fleet.ships.push(ShipClass::battleship(&format!("{}-Dreadnought {:02}", prefix, i)));
                }
                for i in 1..=ca_count {
                    fleet.ships.push(ShipClass::cruiser(&format!("{}-HeavyCruiser {:02}", prefix, i)));
                }
                for i in 1..=ff_count {
                    fleet.ships.push(ShipClass::frigate(&format!("{}-ScreenFrigate {:02}", prefix, i)));
                }
            }
            "HunterKillerPatrol" => {
                // Fast interceptor task force with heavy screening frigates and corvettes
                let ca_count = (firepower / 220.0).floor().max(1.0) as usize;
                let ff_count = (firepower / 100.0).floor().max(2.0) as usize;
                let dd_count = (firepower / 50.0).floor().max(3.0) as usize;

                for i in 1..=ca_count {
                    fleet.ships.push(ShipClass::cruiser(&format!("{}-VanguardCruiser {:02}", prefix, i)));
                }
                for i in 1..=ff_count {
                    fleet.ships.push(ShipClass::frigate(&format!("{}-HunterFrigate {:02}", prefix, i)));
                }
                for i in 1..=dd_count {
                    fleet.ships.push(ShipClass::corvette(&format!("{}-Interceptor {:02}", prefix, i)));
                }
            }
            "Raider" => {
                // Fast hit and run corvettes and missile frigates
                let ff_count = (firepower / 120.0).floor().max(1.0) as usize;
                let dd_count = (firepower / 40.0).floor().max(4.0) as usize;

                for i in 1..=ff_count {
                    fleet.ships.push(ShipClass::frigate(&format!("{}-Corsair {:02}", prefix, i)));
                }
                for i in 1..=dd_count {
                    fleet.ships.push(ShipClass::corvette(&format!("{}-Skiff {:02}", prefix, i)));
                }
            }
            _ => {
                // Default balanced patrol
                let ff_count = (firepower / 100.0).floor().max(2.0) as usize;
                let dd_count = (firepower / 50.0).floor().max(2.0) as usize;

                for i in 1..=ff_count {
                    fleet.ships.push(ShipClass::frigate(&format!("{}-Frigate {:02}", prefix, i)));
                }
                for i in 1..=dd_count {
                    fleet.ships.push(ShipClass::corvette(&format!("{}-PatrolCraft {:02}", prefix, i)));
                }
            }
        }

        fleet
    }
}
