-- Star-Core Modular Ship Block Definitions
return {
    blocks = {
        {
            id = "hull_armor",
            name = "Reinforced Armor Hull",
            category = "Structure",
            max_hp = 500.0,
            mass = 50.0,
            walkable = false,
            sealed = true,
            power_draw = 0.0,
            description = "Heavy composite plating to deflect micrometeorites and laser fire."
        },
        {
            id = "corridor",
            name = "Pressurized Corridor",
            category = "Interior",
            max_hp = 120.0,
            mass = 10.0,
            walkable = true,
            sealed = true,
            power_draw = 1.0,
            description = "Walkway inside ship with artificial gravity and oxygen supply."
        },
        {
            id = "cockpit",
            name = "Command Bridge",
            category = "Control",
            max_hp = 250.0,
            mass = 35.0,
            walkable = true,
            sealed = true,
            power_draw = 15.0,
            description = "Helm control console where the pilot maneuvers the vessel."
        },
        {
            id = "fusion_reactor",
            name = "Compact Fusion Core",
            category = "Power",
            max_hp = 300.0,
            mass = 70.0,
            walkable = false,
            sealed = true,
            power_draw = -150.0, -- Generates 150 kW power
            description = "Provides main power to thrusters, life support, and shields."
        },
        {
            id = "ion_thruster",
            name = "Ion Vector Thruster",
            category = "Propulsion",
            max_hp = 180.0,
            mass = 40.0,
            walkable = false,
            sealed = true,
            power_draw = 40.0,
            thrust = 2500.0,
            description = "High efficiency maneuvering thruster."
        },
        {
            id = "cargo_bay",
            name = "Pressurized Cargo Bay",
            category = "Storage",
            max_hp = 200.0,
            mass = 25.0,
            walkable = true,
            sealed = true,
            power_draw = 5.0,
            cargo_capacity = 50.0, -- Metric tons
            description = "Secure magnetic bay for shipping trade goods and raw minerals."
        },
        {
            id = "airlock",
            name = "Decompression Airlock",
            category = "Access",
            max_hp = 220.0,
            mass = 20.0,
            walkable = true,
            sealed = true,
            power_draw = 2.0,
            is_eva_exit = true,
            description = "Airlock allowing the crew to exit the ship in an EVA suit into open space."
        }
    }
}
