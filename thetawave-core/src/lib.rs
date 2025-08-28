use bevy::{app::Plugin, color::Color, ecs::component::Component, prelude::App, reflect::Reflect};
use serde::Deserialize;

/// Used for designating factions for projectiles
#[derive(Debug, Clone, Reflect, Deserialize, Component)]
pub enum Faction {
    Ally,
    Enemy,
}

impl Faction {
    /// Get basic faction color for simple use cases
    pub fn get_base_color(&self) -> Color {
        match self {
            Faction::Ally => Color::srgba(0.0, 0.0, 5.0, 1.0), // Blue with bloom
            Faction::Enemy => Color::srgba(5.0, 0.0, 0.0, 1.0), // Red with bloom
        }
    }
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

/// Component for transferring collision damage between players, projectiles, and mobs
#[derive(Component, Reflect)]
pub struct CollisionDamage(pub u32);

pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();
    }
}
