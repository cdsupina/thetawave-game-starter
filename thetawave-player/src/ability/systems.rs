use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        system::{Commands, In, Query, Res},
    },
    math::Vec2,
    transform::components::Transform,
};
use thetawave_core::Faction;
use thetawave_projectiles::{ProjectileType, SpawnProjectileEvent};

use crate::{ExecutePlayerAbilityEvent, PlayerStats, ability::AbilityRegistry};

pub(crate) fn ability_dispatcher_system(
    mut cmds: Commands,
    ability_reg: Res<AbilityRegistry>,
    mut player_ability_event_reader: EventReader<ExecutePlayerAbilityEvent>,
) {
    for ExecutePlayerAbilityEvent {
        ability_type,
        player_entity,
    } in player_ability_event_reader.read()
    {
        if let Some(&ability_system) = ability_reg.abilities.get(ability_type) {
            cmds.run_system_with(ability_system, *player_entity);
        }
    }
}

pub(crate) fn fire_blast_ability(
    In(player_entity): In<Entity>,
    player_q: Query<(&Transform, &PlayerStats, &LinearVelocity)>,
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
) {
    if let Ok((transform, player_stats, lin_vel)) = player_q.get(player_entity) {
        spawn_projectile_event_writer.write(SpawnProjectileEvent {
            projectile_type: ProjectileType::Blast,
            faction: Faction::Ally,
            position: transform.translation.truncate() + Vec2::new(0.0, 10.0),
            velocity: Vec2::new(0.0, player_stats.projectile_speed + 100.0) + lin_vel.0,
            damage: player_stats.projectile_damage,
            range_seconds: 1.0,
        });
    }
}
pub(crate) fn fire_bullet_ability(
    In(player_entity): In<Entity>,
    player_q: Query<(&Transform, &PlayerStats, &LinearVelocity)>,
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
) {
    if let Ok((transform, player_stats, lin_vel)) = player_q.get(player_entity) {
        spawn_projectile_event_writer.write(SpawnProjectileEvent {
            projectile_type: ProjectileType::Bullet,
            faction: Faction::Ally,
            position: transform.translation.truncate() + Vec2::new(0.0, 15.0),
            velocity: Vec2::new(0.0, player_stats.projectile_speed) + lin_vel.0,
            damage: player_stats.projectile_damage,
            range_seconds: 1.0,
        });
    }
}
