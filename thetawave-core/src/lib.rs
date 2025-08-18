//! # Thetawave Core
//! 
//! Core shared components and systems for the Thetawave game.
//! This crate provides fundamental game mechanics and data structures
//! that are used across multiple game modules.
//! 
//! ## Components
//! 
//! - [`HealthComponent`] - Manages entity health, damage, and healing
//! - [`Faction`] - Designates entity allegiance (Ally/Enemy)

use bevy::{app::Plugin, ecs::component::Component, prelude::App, reflect::Reflect};
use serde::Deserialize;

/// Designates factions for entities like projectiles and mobs.
/// 
/// This enum is used to determine combat interactions and targeting behavior.
/// Entities of different factions will typically be hostile to each other.
#[derive(Debug, Clone, Reflect, Deserialize)]
pub enum Faction {
    Ally,
    Enemy,
}

/// Component for tracking the health of game entities.
/// 
/// Provides functionality for damage, healing, and death detection.
/// Health values are represented as unsigned integers to prevent negative health.
#[derive(Component, Reflect)]
pub struct HealthComponent {
    pub max_health: u32,
    pub current_health: u32,
}

impl HealthComponent {
    /// Creates a new HealthComponent with the specified maximum health.
    /// 
    /// Both current and maximum health are set to the same value.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The maximum health value for this component
    pub fn new(value: u32) -> Self {
        HealthComponent {
            max_health: value,
            current_health: value,
        }
    }

    /// Applies damage to this health component.
    /// 
    /// Uses saturating subtraction to prevent underflow.
    /// 
    /// # Arguments
    /// 
    /// * `damage` - The amount of damage to apply
    /// 
    /// # Returns
    /// 
    /// `true` if the entity should be considered dead (health reached 0), `false` otherwise
    pub fn take_damage(&mut self, damage: u32) -> bool {
        self.current_health = self.current_health.saturating_sub(damage);
        self.current_health == 0
    }

    /// Heals this health component by the specified amount.
    /// 
    /// Healing is capped at the maximum health value.
    /// 
    /// # Arguments
    /// 
    /// * `amount` - The amount of health to restore
    pub fn heal(&mut self, amount: u32) {
        self.current_health = (self.current_health + amount).min(self.max_health);
    }
}

/// Plugin that registers core Thetawave components and systems.
/// 
/// This plugin should be added early in the application setup as many
/// other Thetawave plugins depend on the components defined here.
pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();
    }
}
