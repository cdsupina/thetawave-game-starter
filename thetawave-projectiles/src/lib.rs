//! # Thetawave Projectiles
//! 
//! Projectile system for the Thetawave game, handling projectile attributes,
//! spawning, and lifecycle management.
//! 
//! This crate provides:
//! 
//! ## Core Components
//! - [`ProjectileType`] - Different types of projectiles (bullets, missiles, etc.)
//! - [`ProjectileSpawner`] - Configuration for spawning projectiles
//! - [`SpawnProjectileEvent`] - Event for requesting projectile spawning
//! 
//! ## Plugin
//! - [`ThetawaveProjectilesPlugin`] - Main plugin that registers all projectile systems
//! 
//! ## Usage
//! 
//! Add the [`ThetawaveProjectilesPlugin`] to your Bevy app to enable projectile functionality:
//! 
//! ```rust,no_run
//! use bevy::prelude::*;
//! use thetawave_projectiles::ThetawaveProjectilesPlugin;
//! 
//! App::new()
//!     .add_plugins(ThetawaveProjectilesPlugin)
//!     .run();
//! ```

mod attributes;
mod plugin;
mod spawn;

pub use attributes::{ProjectileSpawner, ProjectileType, SpawnProjectileEvent};
pub use plugin::ThetawaveProjectilesPlugin;
