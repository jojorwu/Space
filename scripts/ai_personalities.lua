-- Star-Core AI Personalities and Behavioral Weights
return {
    personalities = {
        militarist = {
            id = "militarist",
            name = "Warmonger Strategist",
            expansionism = 1.3,
            aggression = 1.8,
            economic_greed = 0.7,
            defense_bias = 0.9,
            risk_tolerance = 1.4,
            interception_chance = 0.75,
            description = "Relentlessly attacks weak neighbors and aggressively contests unaligned worlds."
        },
        mercantile = {
            id = "mercantile",
            name = "Pragmatic Merchant",
            expansionism = 1.1,
            aggression = 0.3,
            economic_greed = 2.0,
            defense_bias = 1.2,
            risk_tolerance = 0.4,
            interception_chance = 0.30,
            description = "Prioritizes high-yield trade routes, establishes peaceful buffer zones, avoids expensive wars."
        },
        technocrat = {
            id = "technocrat",
            name = "Fortified Technocrat",
            expansionism = 0.6,
            aggression = 0.5,
            economic_greed = 1.2,
            defense_bias = 2.2,
            risk_tolerance = 0.5,
            interception_chance = 0.85,
            description = "Focuses on impenetrable planetary garrisons and rapid interception of hostile armadas."
        },
        pirate = {
            id = "pirate",
            name = "Void Corsair",
            expansionism = 1.4,
            aggression = 1.9,
            economic_greed = 1.6,
            defense_bias = 0.4,
            risk_tolerance = 2.0,
            interception_chance = 0.40,
            description = "Reckless raiders preying on undefended worlds, taking extreme risks for massive loot."
        }
    }
}
