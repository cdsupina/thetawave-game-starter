mod data;
mod plugin;

pub(crate) use data::{ProjectileAttributesResource, ProjectileRangeComponent};
pub(crate) use plugin::ThetawaveAttributesPlugin;

pub use data::{ProjectileSpawner, ProjectileType, SpawnProjectileEvent};
