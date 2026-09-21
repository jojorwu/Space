-- Star-Core Procedural Factions, Heraldry, and Diplomacy Config
return {
    name_generator = {
        prefixes = {
            "Solar", "Iron", "Crimson", "Cyber", "Astral", "Free", "Sovereign",
            "Stellar", "Obsidian", "Vanguard", "Titan", "Zenith", "Phoenix",
            "Nova", "Aegis", "Eclipse", "Celestial", "Apex", "Nexus", "Valiant"
        },
        suffixes = {
            "Empire", "Syndicate", "Federation", "Hegemony", "Commonwealth",
            "Dominion", "Directorate", "Enclave", "Ascendancy", "Republic",
            "Conglomerate", "Coalition", "Alliance", "Order", "Marauders"
        }
    },
    flag_symbols = {
        { symbol = "[*]", name = "Imperial Star" },
        { symbol = "[X]", name = "Crossed Blades" },
        { symbol = "[$]", name = "Gilded Merchant Scales" },
        { symbol = "[#]", name = "Atom Core" },
        { symbol = "[^]", name = "Crown of Sovereign" },
        { symbol = "[o]", name = "Aegis Shield" },
        { symbol = "[~]", name = "Solar Falcon" },
        { symbol = "[%]", name = "Industrial Gear" }
    },
    color_palettes = {
        { name = "Solar Gold", primary = {1.0, 0.84, 0.0}, secondary = {0.2, 0.2, 0.2} },
        { name = "Crimson Blood", primary = {0.86, 0.08, 0.24}, secondary = {0.1, 0.1, 0.1} },
        { name = "Deep Cobalt", primary = {0.0, 0.28, 0.67}, secondary = {0.9, 0.9, 0.9} },
        { name = "Emerald Jade", primary = {0.0, 0.75, 0.35}, secondary = {0.1, 0.2, 0.1} },
        { name = "Void Violet", primary = {0.55, 0.0, 0.7}, secondary = {0.8, 0.7, 1.0} },
        { name = "Cyber Cyan", primary = {0.0, 0.85, 0.9}, secondary = {0.05, 0.1, 0.15} },
        { name = "Obsidian Dark", primary = {0.15, 0.15, 0.15}, secondary = {0.9, 0.2, 0.2} }
    },
    ideologies = {
        {
            id = "Militarist",
            name = "Militarist Expansionists",
            description = "Seeks galactic dominance, heavily builds warfleets, easily declares war on rivals.",
            war_threshold = -25.0,
            expansion_rate = 1.3,
            defense_bonus = 1.2
        },
        {
            id = "Mercantile",
            name = "Trade Consortium",
            description = "Prioritizes economic wealth, establishes trade agreements, avoids expensive wars.",
            war_threshold = -60.0,
            expansion_rate = 1.0,
            defense_bonus = 0.9
        },
        {
            id = "Technocrat",
            name = "Technocratic Enclave",
            description = "Focuses on planetary defense, science, and high-tech industry.",
            war_threshold = -45.0,
            expansion_rate = 0.8,
            defense_bonus = 1.5
        },
        {
            id = "Pirate",
            name = "Lawless Corsairs",
            description = "Frontier raiders preying on trade routes and weak independent planets.",
            war_threshold = -15.0,
            expansion_rate = 1.5,
            defense_bonus = 0.8
        }
    }
}
