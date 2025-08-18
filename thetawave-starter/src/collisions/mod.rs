use avian2d::prelude::CollisionStarted;
use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::EventReader,
        query::{With, Without},
        system::Query,
    },
    log::info,
};
use thetawave_core::{CollisionDamage, HealthComponent};
use thetawave_mobs::MobType;
use thetawave_player::PlayerStats;
use thetawave_projectiles::ProjectileType;

/// A plugin for managing the Thetawave game's camera systems
pub(crate) struct ThetawaveCollisionsPlugin;

impl Plugin for ThetawaveCollisionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, detect_collisions_system);
    }
}

pub fn detect_collisions_system(
    mut started: EventReader<CollisionStarted>,
    player_q: Query<&mut HealthComponent, (With<PlayerStats>, Without<MobType>)>,
    mob_q: Query<&mut HealthComponent, (With<MobType>, Without<PlayerStats>)>,
    projectile_q: Query<&CollisionDamage, With<ProjectileType>>,
) {
    for event in started.read() {
        // Get the two entities involved in the collision
        let entity1 = event.0;
        let entity2 = event.1;

        // Check if one of the entities is a projectile
        if let Ok(projectile_damage) = projectile_q.get(entity1) {
            // entity1 is a projectile, check if entity2 is a player or mob
            if player_q.contains(entity2) {
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Player (Entity {:?}) for {} damage",
                    entity1, entity2, projectile_damage.0
                );
            } else if mob_q.contains(entity2) {
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Mob (Entity {:?}) for {} damage",
                    entity1, entity2, projectile_damage.0
                );
            }
        } else if let Ok(projectile_damage) = projectile_q.get(entity2) {
            // entity2 is a projectile, check if entity1 is a player or mob
            if player_q.contains(entity1) {
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Player (Entity {:?}) for {} damage",
                    entity2, entity1, projectile_damage.0
                );
            } else if mob_q.contains(entity1) {
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Mob (Entity {:?}) for {} damage",
                    entity2, entity1, projectile_damage.0
                );
            }
        }
    }
}
