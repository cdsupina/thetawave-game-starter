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

// mega_blast ability
const MEGA_BLAST_SCALE: f32 = 4.0;
const MEGA_BLAST_VELOCITY_MULTIPLIER: f32 = 1.5;
const MEGA_BLAST_DAMAGE_MULTIPLIER: u32 = 5;

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
            projectile_spread: player_stats.projectile_spread.clone(),
            count: player_stats.projectile_count,
            faction: Faction::Ally,
            position: transform.translation.truncate() + player_stats.projectile_spawner_position,
            scale: 1.0,
            velocity: Vec2::new(0.0, player_stats.projectile_speed)
                + (player_stats.inherited_velocity_multiplier * lin_vel.0),
            damage: player_stats.projectile_damage,
            range_seconds: player_stats.projectile_range_seconds,
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
            projectile_spread: player_stats.projectile_spread.clone(),
            count: player_stats.projectile_count,
            faction: Faction::Ally,
            position: transform.translation.truncate() + player_stats.projectile_spawner_position,
            scale: 1.0,
            velocity: Vec2::new(0.0, player_stats.projectile_speed)
                + (player_stats.inherited_velocity_multiplier * lin_vel.0),
            damage: player_stats.projectile_damage,
            range_seconds: player_stats.projectile_range_seconds,
        });
    }
}

pub(crate) fn mega_blast_ability(
    In(player_entity): In<Entity>,
    player_q: Query<(&Transform, &PlayerStats, &LinearVelocity)>,
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
) {
    if let Ok((transform, player_stats, lin_vel)) = player_q.get(player_entity) {
        spawn_projectile_event_writer.write(SpawnProjectileEvent {
            projectile_type: ProjectileType::Blast,
            projectile_spread: player_stats.projectile_spread.clone(),
            count: (player_stats.projectile_count / 2).max(1),
            faction: Faction::Ally,
            position: transform.translation.truncate() + player_stats.projectile_spawner_position,
            scale: MEGA_BLAST_SCALE,
            velocity: Vec2::new(
                0.0,
                player_stats.projectile_speed * MEGA_BLAST_VELOCITY_MULTIPLIER,
            ) + (player_stats.inherited_velocity_multiplier * lin_vel.0),
            damage: player_stats.projectile_damage * MEGA_BLAST_DAMAGE_MULTIPLIER,
            range_seconds: player_stats.projectile_range_seconds,
        });
    }
}

pub(crate) fn charge_ability(
    In(player_entity): In<Entity>,
    mut player_q: Query<(&mut PlayerStats, &mut LinearVelocity)>,
) {
    if let Ok((mut player_stats, mut lin_vel)) = player_q.get_mut(player_entity) {
        // Apply a forward velocity boost that can exceed max_speed
        player_stats.max_speed *= 2.0;
        player_stats.acceleration *= 4.0;
    }
}
