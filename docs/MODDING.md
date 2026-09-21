# Project Star-Core Modding Engine & Tooling Specification

## Overview
Project Star-Core features a dynamic Lua-driven modding architecture. Game content—including commodities, modular ship blocks, AI personalities, factions, and galaxy zone parameters—can be created, customized, or overridden without recompiling Rust binaries.

---

## Directory Structure
Place custom Lua mod files into the root `mods/` directory:
```
star_core/
├── mods/
│   ├── custom_items.lua
│   ├── custom_blocks.lua
│   └── custom_personalities.lua
├── scripts/
│   ├── items.lua
│   ├── blocks.lua
│   ├── factions.lua
│   └── ai_personalities.lua
```

---

## Modding Data Formats

### 1. Custom Items (`mods/custom_items.lua`)
```lua
return {
    items = {
        {
            id = "antimatter_cells",
            name = "Antimatter Containment Cell",
            category = "HighTech", -- RawMaterial, Industrial, HighTech, Medical, ConsumerGoods
            base_price = 850.0,
            unit_mass = 0.5,
            description = "Stabilized antimatter core for high-yield hyperdrives."
        }
    }
}
```

### 2. Custom Ship Blocks (`mods/custom_blocks.lua`)
```lua
return {
    blocks = {
        {
            id = "quantum_shield_generator",
            name = "Quantum Shield Matrix",
            category = "Defense",
            max_hp = 450.0,
            mass = 120.0,
            walkable = true,
            sealed = true,
            power_draw = 80.0,
            thrust = 0.0,
            cargo_capacity = 0.0,
            is_eva_exit = false,
            description = "High-energy shield projector emitter."
        }
    }
}
```

### 3. Custom AI Personalities (`mods/custom_personalities.lua`)
```lua
return {
    personalities = {
        zealot = {
            aggression = 1.8,
            expansionism = 1.6,
            economic_greed = 0.4,
            defense_bias = 1.2,
            interception_chance = 0.85
        }
    }
}
```

---

## Modding API Tools (`src/scripting.rs`)

The `ScriptEngine` provides the following Rust entrypoint for discovering and executing mod files dynamically:

```rust
// Discovers and executes all .lua scripts inside the mods/ directory
let mods_loaded = engine.load_mods_from_dir(Path::new("mods")).unwrap();
println!("Loaded {} custom Lua mods successfully.", mods_loaded);
```
