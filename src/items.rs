use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemCategory {
    RawMaterial,
    ConsumerGood,
    Industrial,
    HighTech,
    Energy,
    Other,
}

impl From<&str> for ItemCategory {
    fn from(s: &str) -> Self {
        match s {
            "RawMaterial" => ItemCategory::RawMaterial,
            "ConsumerGood" => ItemCategory::ConsumerGood,
            "Industrial" => ItemCategory::Industrial,
            "HighTech" => ItemCategory::HighTech,
            "Energy" => ItemCategory::Energy,
            _ => ItemCategory::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub category: ItemCategory,
    pub base_price: f64,
    pub unit_mass: f64,
    pub description: String,
}
