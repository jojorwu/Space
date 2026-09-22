use std::collections::HashMap;
use std::path::Path;
use mlua::{Lua, Result, Table};

use crate::items::{ItemCategory, ItemDef};

#[derive(Debug, Clone)]
pub struct ZoneConfig {
    pub id: String,
    pub name: String,
    pub police_response_time: f64,
    pub tariff_rate: f64,
    pub piracy_risk: f64,
    pub min_planets: usize,
}

#[derive(Debug, Clone)]
pub struct FactionDef {
    pub id: String,
    pub name: String,
    pub home_zone: String,
    pub color: [f32; 3],
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct BlockDef {
    pub id: String,
    pub name: String,
    pub category: String,
    pub max_hp: f32,
    pub mass: f32,
    pub walkable: bool,
    pub sealed: bool,
    pub power_draw: f32,
    pub thrust: f32,
    pub cargo_capacity: f32,
    pub is_eva_exit: bool,
    pub description: String,
}

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    pub fn load_items<P: AsRef<Path>>(&self, path: P) -> Result<HashMap<String, ItemDef>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read items lua file: {}", e)))?;
        let clean_content = content.trim_start_matches('\u{feff}');
        let table: Table = self.lua.load(clean_content).eval()?;
        let items_table: Table = table.get("items")?;

        let mut items = HashMap::new();
        for pair in items_table.pairs::<mlua::Integer, Table>() {
            let (_, item_tbl) = pair?;
            let id: String = item_tbl.get("id")?;
            let name: String = item_tbl.get("name")?;
            let cat_str: String = item_tbl.get("category")?;
            let base_price: f64 = item_tbl.get("base_price")?;
            let unit_mass: f64 = item_tbl.get("unit_mass")?;
            let description: String = item_tbl.get("description")?;

            items.insert(
                id.clone(),
                ItemDef {
                    id,
                    name,
                    category: ItemCategory::from(cat_str.as_str()),
                    base_price,
                    unit_mass,
                    description,
                },
            );
        }
        Ok(items)
    }

    pub fn load_factions_and_zones<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(HashMap<String, ZoneConfig>, Vec<FactionDef>)> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read factions lua file: {}", e)))?;
        let clean_content = content.trim_start_matches('\u{feff}');
        let table: Table = self.lua.load(clean_content).eval()?;

        let zones_tbl: Table = table.get("zones")?;
        let mut zones = HashMap::new();
        for pair in zones_tbl.pairs::<String, Table>() {
            let (key, z_tbl) = pair?;
            let id: String = z_tbl.get("id")?;
            let name: String = z_tbl.get("name")?;
            let police_response_time: f64 = z_tbl.get("police_response_time")?;
            let tariff_rate: f64 = z_tbl.get("tariff_rate")?;
            let piracy_risk: f64 = z_tbl.get("piracy_risk")?;
            let min_planets: usize = z_tbl.get("min_planets")?;

            zones.insert(
                key,
                ZoneConfig {
                    id,
                    name,
                    police_response_time,
                    tariff_rate,
                    piracy_risk,
                    min_planets,
                },
            );
        }

        let factions_tbl: Table = table.get("factions")?;
        let mut factions = Vec::new();
        for pair in factions_tbl.pairs::<mlua::Integer, Table>() {
            let (_, f_tbl) = pair?;
            let id: String = f_tbl.get("id")?;
            let name: String = f_tbl.get("name")?;
            let home_zone: String = f_tbl.get("home_zone")?;
            let col_tbl: Table = f_tbl.get("color")?;
            let r: f32 = col_tbl.get(1)?;
            let g: f32 = col_tbl.get(2)?;
            let b: f32 = col_tbl.get(3)?;
            let description: String = f_tbl.get("description")?;

            factions.push(FactionDef {
                id,
                name,
                home_zone,
                color: [r, g, b],
                description,
            });
        }

        Ok((zones, factions))
    }

    pub fn load_blocks<P: AsRef<Path>>(&self, path: P) -> Result<HashMap<String, BlockDef>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read blocks lua file: {}", e)))?;
        let clean_content = content.trim_start_matches('\u{feff}');
        let table: Table = self.lua.load(clean_content).eval()?;
        let blocks_tbl: Table = table.get("blocks")?;

        let mut blocks = HashMap::new();
        for pair in blocks_tbl.pairs::<mlua::Integer, Table>() {
            let (_, b_tbl) = pair?;
            let id: String = b_tbl.get("id")?;
            let name: String = b_tbl.get("name")?;
            let category: String = b_tbl.get("category")?;
            let max_hp: f32 = b_tbl.get("max_hp")?;
            let mass: f32 = b_tbl.get("mass")?;
            let walkable: bool = b_tbl.get("walkable")?;
            let sealed: bool = b_tbl.get("sealed")?;
            let power_draw: f32 = b_tbl.get("power_draw")?;
            let thrust: f32 = b_tbl.get("thrust").unwrap_or(0.0);
            let cargo_capacity: f32 = b_tbl.get("cargo_capacity").unwrap_or(0.0);
            let is_eva_exit: bool = b_tbl.get("is_eva_exit").unwrap_or(false);
            let description: String = b_tbl.get("description")?;

            blocks.insert(
                id.clone(),
                BlockDef {
                    id,
                    name,
                    category,
                    max_hp,
                    mass,
                    walkable,
                    sealed,
                    power_draw,
                    thrust,
                    cargo_capacity,
                    is_eva_exit,
                    description,
                },
            );
        }
        Ok(blocks)
    }

    pub fn load_personalities<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<HashMap<String, crate::ai::personality::AiPersonality>> {
        crate::ai::personality::PersonalityLoader::load_from_lua(&self.lua, path)
    }

    /// Triggers Lua garbage collection cycle to reclaim unused script memory
    pub fn gc(&self) -> Result<()> {
        self.lua.gc_collect()
    }

    /// Returns the total memory allocated by the Lua engine state in bytes
    pub fn used_memory(&self) -> usize {
        self.lua.used_memory()
    }

    /// Modding Engine API: Dynamically loads and merges all `.lua` mod files inside a specified `mods/` directory
    pub fn load_mods_from_dir<P: AsRef<Path>>(&self, mods_dir: P) -> Result<usize> {
        let path = mods_dir.as_ref();
        if !path.exists() || !path.is_dir() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read mods directory: {}", e)))?;

        let mut loaded_count = 0;
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("lua") {
                let content = std::fs::read_to_string(&file_path)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read mod file {:?}: {}", file_path, e)))?;
                let clean_content = content.trim_start_matches('\u{feff}');
                self.lua.load(clean_content).exec()?;
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }
}

