//! Projectile system for spawning and managing projectiles.

mod attributes;
mod plugin;
mod spawn;

pub use attributes::{ProjectileSpawner, ProjectileType, SpawnProjectileEvent};
pub use plugin::ThetawaveProjectilesPlugin;
