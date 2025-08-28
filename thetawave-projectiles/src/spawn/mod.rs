mod effect;
mod projectile;

use bevy::color::Color;
use bevy::prelude::Alpha;
use thetawave_core::Faction;
pub(crate) use {effect::spawn_effect_system, projectile::spawn_projectile_system};

use crate::ProjectileType;

pub trait FactionExt {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color;
}

impl FactionExt for Faction {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color {
        match projectile_type {
            ProjectileType::Bullet => self.get_base_color(),
            ProjectileType::Blast => self.get_base_color().with_alpha(0.25),
        }
    }
}
