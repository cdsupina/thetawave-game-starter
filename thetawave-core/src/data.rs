use bevy::{color::Color, ecs::component::Component, reflect::Reflect};
use serde::Deserialize;

// Faction color constants
const ALLY_BASE_COLOR: Color = Color::srgba(3.0, 3.0, 0.0, 1.0); // Yellow with bloom
const ENEMY_BASE_COLOR: Color = Color::srgba(3.0, 0.0, 0.0, 1.0); // Red with bloom

/// Used for designating factions for projectiles
#[derive(Debug, Clone, Reflect, Deserialize, Component)]
pub enum Faction {
    Ally,
    Enemy,
}

impl Faction {
    /// Get faction color
    pub fn get_color(&self) -> Color {
        match self {
            Faction::Ally => ALLY_BASE_COLOR,
            Faction::Enemy => ENEMY_BASE_COLOR,
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
