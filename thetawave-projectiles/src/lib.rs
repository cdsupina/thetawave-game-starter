//! Projectile system for spawning and managing projectiles.

mod attributes;
mod plugin;
mod spawn;
mod systems;

pub use attributes::{ProjectileSpawner, ProjectileType, SpawnProjectileEvent};
pub use plugin::ThetawaveProjectilesPlugin;
pub use spawn::FactionExt;
