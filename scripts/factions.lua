-- Star-Core Faction and Zone Definitions
return {
    zones = {
        core = {
            id = "core",
            name = "Imperial Core",
            police_response_time = 1.5,
            tariff_rate = 0.15,
            piracy_risk = 0.02,
            min_planets = 30
        },
        neutral = {
            id = "neutral",
            name = "Independent Trade Expanse",
            police_response_time = 5.0,
            tariff_rate = 0.05,
            piracy_risk = 0.15,
            min_planets = 40
        },
        frontier = {
            id = "frontier",
            name = "Outer Rim & Wild Frontier",
            police_response_time = 999.0,
            tariff_rate = 0.0,
            piracy_risk = 0.50,
            min_planets = 30
        }
    },
    factions = {
        {
            id = "solaris_directorate",
            name = "Solaris Directorate",
            home_zone = "core",
            color = {0.2, 0.6, 1.0},
            description = "Technocratic empire ruling the core worlds with strict laws and dense industrial infrastructure."
        },
        {
            id = "trade_league",
            name = "Free Trade League",
            home_zone = "neutral",
            color = {0.2, 0.9, 0.4},
            description = "A powerful coalition of merchants, orbital stations, and independent refineries."
        },
        {
            id = "void_marauders",
            name = "Crimson Marauders",
            home_zone = "frontier",
            color = {0.9, 0.2, 0.2},
            description = "Outlaws and pirate syndicates operating from asteroid bases and lawless frontier planets."
        }
    }
}
