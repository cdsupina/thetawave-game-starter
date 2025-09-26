use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        system::{Commands, In, Query, Res},
    },
    math::Vec2,
    time::{Time, Timer},
    transform::components::Transform,
};
use thetawave_core::Faction;
use thetawave_projectiles::{ProjectileType, SpawnProjectileEvent};

use crate::{ExecutePlayerAbilityEvent, PlayerAbility, PlayerStats, ability::AbilityRegistry};
use leafwing_abilities::prelude::CooldownState;

#[derive(Component)]
pub struct ChargeAbility {
    pub timer: Timer,
    pub original_max_speed: f32,
    pub original_acceleration: f32,
    pub ability_type: PlayerAbility,
}

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
    mut commands: Commands,
    mut player_q: Query<&mut PlayerStats>,
    charge_q: Query<&ChargeAbility>,
) {
    if let Ok(mut player_stats) = player_q.get_mut(player_entity) {
        // Only activate charge if not already active
        if charge_q.get(player_entity).is_err() {
            let original_max_speed = player_stats.max_speed;
            let original_acceleration = player_stats.acceleration;

            // Apply the charge boost
            player_stats.max_speed *= 3.0;
            player_stats.acceleration *= 3.0;

            // Add the charge component with a 3 second timer
            commands.entity(player_entity).insert(ChargeAbility {
                timer: Timer::from_seconds(1.5, bevy::time::TimerMode::Once),
                original_max_speed,
                original_acceleration,
                ability_type: PlayerAbility::SecondaryAttack,
            });
        }
    }
}

pub(crate) fn charge_ability_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut charge_q: Query<(
        Entity,
        &mut ChargeAbility,
        &mut PlayerStats,
        &mut CooldownState<PlayerAbility>,
    )>,
) {
    for (entity, mut charge_ability, mut player_stats, mut cooldown_state) in charge_q.iter_mut() {
        charge_ability.timer.tick(time.delta());

        if charge_ability.timer.finished() {
            // Revert to original stats
            player_stats.max_speed = charge_ability.original_max_speed;
            player_stats.acceleration = charge_ability.original_acceleration;

            // Now trigger the cooldown (start the actual cooldown period)
            // Ability triggers at the end of the ability as well as the beginning
            let _ = cooldown_state.trigger(&charge_ability.ability_type);

            // Remove the charge component
            commands.entity(entity).remove::<ChargeAbility>();
        }
    }
}
