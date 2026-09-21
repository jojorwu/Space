use std::collections::HashMap;
use std::path::Path;
use mlua::{Lua, Result, Table};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPersonality {
    pub id: String,
    pub name: String,
    pub expansionism: f32,
    pub aggression: f32,
    pub economic_greed: f32,
    pub defense_bias: f32,
    pub risk_tolerance: f32,
    pub interception_chance: f32,
    pub description: String,
}

impl Default for AiPersonality {
    fn default() -> Self {
        Self {
            id: "balanced".to_string(),
            name: "Balanced Strategist".to_string(),
            expansionism: 1.0,
            aggression: 1.0,
            economic_greed: 1.0,
            defense_bias: 1.0,
            risk_tolerance: 1.0,
            interception_chance: 0.5,
            description: "Standard balanced galactic actor.".to_string(),
        }
    }
}

pub struct PersonalityLoader;

impl PersonalityLoader {
    pub fn load_from_lua<P: AsRef<Path>>(lua: &Lua, path: P) -> Result<HashMap<String, AiPersonality>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read personalities lua: {}", e)))?;
        let clean = content.trim_start_matches('\u{feff}');
        let root: Table = lua.load(clean).eval()?;
        let table: Table = root.get("personalities")?;

        let mut result = HashMap::new();
        for pair in table.pairs::<String, Table>() {
            let (key, tbl) = pair?;
            let id: String = tbl.get("id")?;
            let name: String = tbl.get("name")?;
            let expansionism: f32 = tbl.get("expansionism").unwrap_or(1.0);
            let aggression: f32 = tbl.get("aggression").unwrap_or(1.0);
            let economic_greed: f32 = tbl.get("economic_greed").unwrap_or(1.0);
            let defense_bias: f32 = tbl.get("defense_bias").unwrap_or(1.0);
            let risk_tolerance: f32 = tbl.get("risk_tolerance").unwrap_or(1.0);
            let interception_chance: f32 = tbl.get("interception_chance").unwrap_or(0.5);
            let description: String = tbl.get("description").unwrap_or_default();

            result.insert(
                key.to_lowercase(),
                AiPersonality {
                    id,
                    name,
                    expansionism,
                    aggression,
                    economic_greed,
                    defense_bias,
                    risk_tolerance,
                    interception_chance,
                    description,
                },
            );
        }
        Ok(result)
    }
}
