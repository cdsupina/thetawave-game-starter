use avian2d::prelude::CollisionStarted;
use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::{EventReader, EventWriter},
        query::{With, Without},
        schedule::{common_conditions, IntoScheduleConfigs},
        system::Query,
    },
    log::info,
};
use thetawave_core::{CollisionDamage, HealthComponent};
use thetawave_mobs::{MobDeathEvent, MobMarker};
use thetawave_player::{PlayerDeathEvent, PlayerStats};
use thetawave_projectiles::{ProjectileSystemSet, ProjectileType};

/// A plugin for managing the Thetawave game's camera systems
pub(crate) struct ThetawaveCollisionsPlugin;

impl Plugin for ThetawaveCollisionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            detect_collisions_system
                .run_if(common_conditions::on_event::<CollisionStarted>)
                .before(ProjectileSystemSet::Despawn),
        );
    }
}

pub fn detect_collisions_system(
    mut started: EventReader<CollisionStarted>,
    mut player_q: Query<&mut HealthComponent, (With<PlayerStats>, Without<MobMarker>)>,
    mut mob_q: Query<&mut HealthComponent, (With<MobMarker>, Without<PlayerStats>)>,
    projectile_q: Query<&CollisionDamage, With<ProjectileType>>,
    mut player_death_event_writer: EventWriter<PlayerDeathEvent>,
    mut mob_death_event_writer: EventWriter<MobDeathEvent>,
) {
    for event in started.read() {
        // Get the two entities involved in the collision
        let entity1 = event.0;
        let entity2 = event.1;

        // Check if one of the entities is a projectile
        if let Ok(projectile_damage) = projectile_q.get(entity1) {
            // entity1 is a projectile, check if entity2 is a player or mob
            if let Ok(mut player_health) = player_q.get_mut(entity2) {
                if player_health.take_damage(projectile_damage.0) {
                    player_death_event_writer.write(PlayerDeathEvent {
                        player_entity: entity2,
                    });
                }
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Player (Entity {:?}) for {} damage. Player health: {}",
                    entity1, entity2, projectile_damage.0, player_health.current_health
                );
            } else if let Ok(mut mob_health) = mob_q.get_mut(entity2) {
                if mob_health.take_damage(projectile_damage.0) {
                    mob_death_event_writer.write(MobDeathEvent {
                        mob_entity: entity2,
                    });
                }
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Mob (Entity {:?}) for {} damage. Mob health: {}",
                    entity1, entity2, projectile_damage.0, mob_health.current_health
                );
            }
        } else if let Ok(projectile_damage) = projectile_q.get(entity2) {
            // entity2 is a projectile, check if entity1 is a player or mob
            if let Ok(mut player_health) = player_q.get_mut(entity1) {
                if player_health.take_damage(projectile_damage.0) {
                    player_death_event_writer.write(PlayerDeathEvent {
                        player_entity: entity1,
                    });
                }
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Player (Entity {:?}) for {} damage. Player health: {}",
                    entity2, entity1, projectile_damage.0, player_health.current_health
                );
            } else if let Ok(mut mob_health) = mob_q.get_mut(entity1) {
                if mob_health.take_damage(projectile_damage.0) {
                    mob_death_event_writer.write(MobDeathEvent {
                        mob_entity: entity1,
                    });
                }
                info!(
                    "Projectile collision detected: Projectile (Entity {:?}) hit Mob (Entity {:?}) for {} damage. Mob health: {}",
                    entity2, entity1, projectile_damage.0, mob_health.current_health
                );
            }
        }
    }
}
