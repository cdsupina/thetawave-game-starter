mod data;
mod plugin;

pub(crate) use data::{
    DespawnAfterAnimationComponent, ProjectileAttributesResource, ProjectileRangeComponent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;

pub use data::{ProjectileSpawner, ProjectileSpread, ProjectileType, SpawnProjectileEvent};
