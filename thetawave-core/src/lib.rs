//! Core shared components for the Thetawave game.

use bevy::{app::Plugin, ecs::component::Component, prelude::App, reflect::Reflect};
use serde::Deserialize;

/// Entity faction for determining combat interactions.
#[derive(Debug, Clone, Reflect, Deserialize)]
pub enum Faction {
    Ally,
    Enemy,
}

/// Component for tracking entity health, damage, and healing.
#[derive(Component, Reflect)]
pub struct HealthComponent {
    pub max_health: u32,
    pub current_health: u32,
}

impl HealthComponent {
    /// Creates a new HealthComponent with the given max health.
    pub fn new(value: u32) -> Self {
        HealthComponent {
            max_health: value,
            current_health: value,
        }
    }

    /// Applies damage and returns true if health reaches 0.
    pub fn take_damage(&mut self, damage: u32) -> bool {
        self.current_health = self.current_health.saturating_sub(damage);
        self.current_health == 0
    }

    /// Heals the entity, capped at max health.
    pub fn heal(&mut self, amount: u32) {
        self.current_health = (self.current_health + amount).min(self.max_health);
    }
}

/// Plugin that registers core Thetawave components for reflection.
pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();
    }
}
