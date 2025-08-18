use bevy::{app::Plugin, ecs::component::Component, prelude::App, reflect::Reflect};
use serde::Deserialize;

/// Used for designating factions for projectiles
#[derive(Debug, Clone, Reflect, Deserialize)]
pub enum Faction {
    Ally,
    Enemy,
}

/// Component for tracking the health of players and mobs
#[derive(Component, Reflect)]
pub struct HealthComponent {
    pub max_health: u32,
    pub current_health: u32,
}

impl HealthComponent {
    pub fn new(value: u32) -> Self {
        HealthComponent {
            max_health: value,
            current_health: value,
        }
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        self.current_health = self.current_health.saturating_sub(damage);
        self.current_health == 0
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_health = (self.current_health + amount).min(self.max_health);
    }
}

pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();
    }
}
