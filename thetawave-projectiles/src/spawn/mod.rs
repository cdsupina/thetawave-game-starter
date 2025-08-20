mod effect;
mod projectile;

use bevy::color::Color;
use thetawave_core::Faction;
pub(crate) use {effect::spawn_effect_system, projectile::spawn_projectile_system};

use crate::ProjectileType;

trait FactionExt {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color;
}

impl FactionExt for Faction {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color {
        match projectile_type {
            ProjectileType::Bullet => match self {
                Faction::Ally => Color::srgba(0.0, 0.0, 5.0, 1.0),
                Faction::Enemy => Color::srgba(5.0, 0.0, 0.0, 1.0),
            },
            ProjectileType::Blast => match self {
                Faction::Ally => Color::srgba(5.0, 5.0, 0.0, 0.25),
                Faction::Enemy => Color::srgba(5.0, 0.0, 0.0, 0.25),
            },
        }
    }
}
