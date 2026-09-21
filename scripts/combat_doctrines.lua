-- scripts/combat_doctrines.lua
-- Star-Core Tactical Combat Doctrines & Damage Matrix

return {
    doctrines = {
        kiting_sniper = {
            id = "kiting_sniper",
            name = "Kiting & Long-Range Artillery",
            preferred_range = 450.0,
            retreat_health_threshold = 0.65, -- Retreats early if shields/armor broken
            target_priority = "engines",     -- Slow down enemy to keep distance
            fire_rate_modifier = 0.90,
            accuracy_modifier = 1.25,
            disengage_bonus = 0.35,          -- Higher chance of successful FTL jump
            description = "Maintains standoff distance, firing particle lances and torpedoes while avoiding close brawls."
        },
        brawling_assault = {
            id = "brawling_assault",
            name = "Brawling & Point-Blank Rush",
            preferred_range = 100.0,
            retreat_health_threshold = 0.20, -- Fights almost to the death
            target_priority = "reactor",     -- High-alpha critical strikes
            fire_rate_modifier = 1.30,
            accuracy_modifier = 0.85,
            disengage_bonus = -0.20,
            description = "Aggressive close-quarters assault. Maximizes damage output with plasma batteries and heavy armor."
        },
        screen_escort = {
            id = "screen_escort",
            name = "Screening & Fleet Escort",
            preferred_range = 250.0,
            retreat_health_threshold = 0.40,
            target_priority = "weapons",     -- Neutralizes enemy offense
            fire_rate_modifier = 1.05,
            accuracy_modifier = 1.10,
            disengage_bonus = 0.10,
            description = "Forms a defensive perimeter, using point defense flak to intercept incoming ordnance."
        },
        hit_and_run = {
            id = "hit_and_run",
            name = "Guerrilla Hit & Run",
            preferred_range = 300.0,
            retreat_health_threshold = 0.75, -- Retreats immediately if taking significant fire
            target_priority = "engines",
            fire_rate_modifier = 1.15,
            accuracy_modifier = 0.95,
            disengage_bonus = 0.50,          -- Rapid tactical escape
            description = "Strikes fast, inflicts disruption, and immediately initiates tactical emergency warp."
        }
    },

    -- Damage Matrix (Attacker Weapon Type -> Target Defense Layer)
    damage_matrix = {
        energy = {
            vs_shields = 1.50, -- Energy destroys electromagnetic shields
            vs_armor   = 0.75,
            vs_hull    = 1.00
        },
        kinetic = {
            vs_shields = 0.70,
            vs_armor   = 1.50, -- Dense kinetics shatter composite armor
            vs_hull    = 1.10
        },
        missile = {
            vs_shields = 0.85,
            vs_armor   = 1.10,
            vs_hull    = 1.75  -- High explosive warheads devastate internal hull
        }
    }
}
