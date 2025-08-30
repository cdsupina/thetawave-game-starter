mod data;
mod plugin;

pub(crate) use data::{
    DespawnAfterAnimationComponent, ProjectileAttributesResource, ProjectileEffectType,
    ProjectileRangeComponent, SpawnProjectileEffectEvent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;

pub use data::{ProjectileSpawner, ProjectileType, SpawnProjectileEvent};
