# Project Star-Core Architecture Specification

## Overview
`star_core` is a high-performance 2D space simulation framework and game engine written in Rust. It powers a dynamic living galaxy with 100 procedural star systems, autonomous trading AI, faction diplomacy, modular tile-based ship physics, and custom rendering pipeline.

---

## System Architecture Map

```
                     ┌──────────────────────────┐
                     │         main.rs          │
                     └────────────┬─────────────┘
                                  │
    ┌─────────────────────────────┼─────────────────────────────┐
    │                             │                             │
┌───▼──────────────┐   ┌──────────▼──────────┐       ┌──────────▼──────────┐
│  Simulation Core │   │  AI & Decision Systems │   │ Rendering & VFX     │
│  (src/sim/)      │   │  (src/ai/)          │   │ (src/render/)       │
│                  │   │                     │   │                     │
│  - SpatialIndex  │   │  - Utility AI Trader│   │ - GalaxyRenderer    │
│  - EventBus      │   │  - Strategic AI     │   │ - ParticleSystem    │
│  - GalacticWorld │   │  - Personalities    │   │ - SciFiUi           │
│  - Combat Engine │   │                     │   │ - Camera System     │
└──────────────────┘   └─────────────────────┘   └─────────────────────┘
```

---

## Module Breakdown

### 1. Simulation & Spatial Index (`src/sim/`)
- **`SpatialIndex` (`src/sim/spatial.rs`)**:
  - Provides $O(1)$ pairwise distance lookups between star systems.
  - Maintains pre-sorted neighbor arrays for instantaneous $O(\log N)$ radius queries.
- **`GalacticWorld` (`src/sim/world.rs`)**:
  - Central orchestrator driving market dynamic pricing, AI trader logistics, fleet movement, and diplomacy turns.
- **`EventBus` (`src/sim/events.rs`)**:
  - Lock-free ring buffer log capturing galactic events (wars, conquests, raids, trade executions).
- **`TacticalCombatResolver` (`src/sim/combat.rs`)**:
  - Multi-phase tactical resolver handling directed energy, kinetic salvo, missile interception, shield/armor layer penetration, and morale FTL disengagement.

### 2. Autonomous Utility AI (`src/ai/`)
- **`TraderAi` (`src/ai/trader_ai.rs`)**:
  - Evaluates multi-factor utility candidates across neighboring star systems considering risk tolerances, archetype behaviors (Merchant, Smuggler, Bulk Hauler), and anti-herd market flooding checks.
- **`StrategicAi` (`src/ai/strategic_ai.rs`)**:
  - Determines expansion targets for unaligned worlds and invasion targets against hostile factions.

### 3. Modular Physics & Ship Mechanics (`src/ship.rs`)
- **`ShipGrid`**:
  - Tile-based modular ship layout with mass recalculation, center of mass tracking, and moment of inertia $I = \sum m_i r_i^2$.
  - Integrated linear and rotational physics (`update_physics`).
  - Breadth-first search (BFS) structural integrity check for splitting detached debris chunks upon hull rupture.

### 4. Custom Rendering & VFX Pipeline (`src/render/`)
- **Frustum Culling**:
  - All particle systems, starfield objects, nebulae, trade routes, and planets perform spatial viewport culling against screen boundaries.
- **`ParticleSystem` (`src/render/particles.rs`)**:
  - Zero-allocation pre-allocated ring buffer particle pool emitting engine trails, laser tracers, explosion sparks, and EVA gas puffs.

---

## Build & Release Profiles
The engine uses maximum optimization settings in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```
