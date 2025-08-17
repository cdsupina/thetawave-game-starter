use bevy::reflect::Reflect;
use serde::Deserialize;

#[derive(Debug, Clone, Reflect, Deserialize)]
pub enum Faction {
    Ally,
    Enemy,
}
