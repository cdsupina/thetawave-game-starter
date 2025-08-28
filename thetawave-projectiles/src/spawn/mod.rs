mod effect;
mod projectile;

use bevy::color::Color;
use bevy::prelude::Alpha;
use thetawave_core::Faction;
pub(crate) use {effect::spawn_effect_system, projectile::spawn_projectile_system};

// Projectile-specific color constants
const ALLY_BULLET_COLOR: Color = Color::srgba(0.0, 0.0, 4.0, 1.0); // Blue with bloom for ally bullets
const ENEMY_BULLET_COLOR: Color = Color::srgba(4.0, 0.0, 0.0, 1.0); // Red for enemy bullets
const BLAST_ALPHA: f32 = 0.25; // Transparency for blast projectiles

use crate::ProjectileType;

pub trait FactionExt {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color;
}

impl FactionExt for Faction {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color {
        match projectile_type {
            ProjectileType::Bullet => match self {
                Faction::Ally => ALLY_BULLET_COLOR,
                Faction::Enemy => ENEMY_BULLET_COLOR,
            },
            ProjectileType::Blast => self.get_base_color().with_alpha(BLAST_ALPHA),
        }
    }
}
