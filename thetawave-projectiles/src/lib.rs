//! Projectile system for spawning and managing projectiles.

mod attributes;
mod plugin;
mod spawn;
mod systems;

use bevy::ecs::schedule::SystemSet;

/// SystemSet for projectile-related systems that other plugins can use for ordering
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProjectileSystemSet {
    /// Systems that handle projectile cleanup/despawning after collisions
    Despawn,
}

pub use attributes::{ProjectileSpawner, ProjectileSpread, ProjectileType, SpawnProjectileEvent};
pub use plugin::ThetawaveProjectilesPlugin;
pub use spawn::FactionExt;
