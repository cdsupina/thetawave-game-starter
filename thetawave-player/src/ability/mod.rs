use bevy::{ecs::resource::Resource, platform::collections::HashMap};
use serde::Deserialize;
use thetawave_projectiles::ProjectileType;

/// Resource containing all available abilities mapped by string keys
#[derive(Resource, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AbilitiesResource {
    pub abilities: HashMap<String, AbilityData>,
}

/// Defines the behavior and parameters of an ability
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AbilityData {
    /// Fires a projectile with specified parameters
    Projectile { projectile_type: ProjectileType },
}
