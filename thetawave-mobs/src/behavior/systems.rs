use avian2d::prelude::{AngularVelocity, LinearVelocity, RevoluteJoint};
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res},
    },
    math::Vec2,
    time::{Time, Timer},
    transform::components::Transform,
};
use bevy_behave::prelude::BehaveCtx;
use thetawave_particles::ActivateParticleEvent;
use thetawave_player::PlayerStats;
use thetawave_projectiles::SpawnProjectileEvent;

use crate::{
    MobType, SpawnMobEvent,
    attributes::{
        JointsComponent, MobAttributesComponent, MobSpawnerComponent, ProjectileSpawnerComponent,
    },
    behavior::{
        BehaviorReceiverComponent, MobBehaviorComponent, MobBehaviorType,
        data::{TargetComponent, TransmitBehaviorEvent},
    },
};

/// MobBehaviorType::TransmitMobBehavior
/// Sends TransmitBehaviorEvents
pub(super) fn transmit_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut transmit_event_writer: EventWriter<TransmitBehaviorEvent>,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        for behavior in mob_behavior.behaviors.iter() {
            if let MobBehaviorType::TransmitMobBehavior {
                mob_type,
                behaviors,
            } = behavior
            {
                transmit_event_writer.write(TransmitBehaviorEvent {
                    source_entity: ctx.target_entity(),
                    mob_type: mob_type.clone(),
                    behaviors: behaviors.clone(),
                });
            }
        }
    }
}

/// Reads TransmitBehaviorEvents and runs the sent behaviors for the designated mobs with matching BehaviorRecieverComponents
pub(super) fn receieve_system(
    mut transmit_event_reader: EventReader<TransmitBehaviorEvent>,
    mut mob_q: Query<(
        &MobType,
        &BehaviorReceiverComponent,
        &MobAttributesComponent,
        &mut LinearVelocity,
    )>,
) {
    for event in transmit_event_reader.read() {
        for (mob_type, behavior_recv, mob_attr, mut lin_vel) in mob_q.iter_mut() {
            if *mob_type == event.mob_type && behavior_recv.0 == event.source_entity {
                for behavior in event.behaviors.iter() {
                    match behavior {
                        MobBehaviorType::MoveDown => apply_move_down(&mut lin_vel, mob_attr),
                        MobBehaviorType::MoveUp => apply_move_up(&mut lin_vel, mob_attr),
                        MobBehaviorType::MoveLeft => apply_move_left(&mut lin_vel, mob_attr),
                        MobBehaviorType::MoveRight => apply_move_right(&mut lin_vel, mob_attr),
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Helper function to apply downward movement
#[inline]
fn apply_move_down(lin_vel: &mut LinearVelocity, attributes: &MobAttributesComponent) {
    if lin_vel.y > -attributes.max_linear_speed.y {
        lin_vel.y -= attributes.linear_acceleration.y;
    }
}

/// Helper function to apply upward movement
#[inline]
fn apply_move_up(lin_vel: &mut LinearVelocity, attributes: &MobAttributesComponent) {
    if lin_vel.y < attributes.max_linear_speed.y {
        lin_vel.y += attributes.linear_acceleration.y;
    }
}

/// Helper function to apply leftward movement
#[inline]
fn apply_move_left(lin_vel: &mut LinearVelocity, attributes: &MobAttributesComponent) {
    if lin_vel.x > -attributes.max_linear_speed.x {
        lin_vel.x -= attributes.linear_acceleration.x;
    }
}

/// Helper function to apply rightward movement
#[inline]
fn apply_move_right(lin_vel: &mut LinearVelocity, attributes: &MobAttributesComponent) {
    if lin_vel.x < attributes.max_linear_speed.x {
        lin_vel.x += attributes.linear_acceleration.x;
    }
}

/// Unified directional movement system
/// Handles MoveDown, MoveUp, MoveLeft, and MoveRight behaviors in a single system
/// This eliminates the need for separate systems for each direction
pub(super) fn directional_movement_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &MobAttributesComponent)>,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mut lin_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        // Check all directional behaviors and apply them
        for behavior in &mob_behavior.behaviors {
            match behavior {
                MobBehaviorType::MoveDown => apply_move_down(&mut lin_vel, attributes),
                MobBehaviorType::MoveUp => apply_move_up(&mut lin_vel, attributes),
                MobBehaviorType::MoveLeft => apply_move_left(&mut lin_vel, attributes),
                MobBehaviorType::MoveRight => apply_move_right(&mut lin_vel, attributes),
                _ => {} // Ignore non-directional movement behaviors
            }
        }
    }
}

/// MobBehaviorType::BrakeHorizontal
/// Modifies LinearVelocity to approach 0 velocity in the x axis
pub(super) fn brake_horizontal_system(
    move_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        if move_behavior
            .behaviors
            .contains(&MobBehaviorType::BrakeHorizontal)
        {
            brake_horizontal(attributes, &mut lin_vel);
        }
    }
}

fn brake_horizontal(attributes: &MobAttributesComponent, lin_vel: &mut LinearVelocity) {
    if lin_vel.x > 0.0 {
        lin_vel.x = (lin_vel.x - attributes.linear_deceleration.x).max(0.0);
    } else if lin_vel.x < 0.0 {
        lin_vel.x = (lin_vel.x + attributes.linear_deceleration.x).min(0.0);
    }
}

/// MobBehaviorType::MoveTo
/// Moves the mob to a designated coordinate
pub(super) fn move_to_system(
    move_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &Transform, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, transform, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        for behavior in move_behavior.behaviors.iter() {
            if let MobBehaviorType::MoveTo(target_pos) = behavior {
                move_to(attributes, &mut lin_vel, transform, target_pos)
            }
        }
    }
}

fn move_to(
    attributes: &MobAttributesComponent,
    lin_vel: &mut LinearVelocity,
    transform: &Transform,
    target_pos: &Vec2,
) {
    let current_pos = transform.translation.truncate();
    let direction = (*target_pos - current_pos).normalize_or_zero();

    let target_velocity = direction * attributes.max_linear_speed;
    let velocity_diff = target_velocity - lin_vel.0;

    // Apply acceleration towards target velocity, clamped by acceleration limits
    lin_vel.x += (velocity_diff.x * attributes.linear_acceleration.x).clamp(
        -attributes.linear_acceleration.x,
        attributes.linear_acceleration.x,
    );
    lin_vel.y += (velocity_diff.y * attributes.linear_acceleration.y).clamp(
        -attributes.linear_acceleration.y,
        attributes.linear_acceleration.y,
    );
}

/// MobBehaviorType::FindPlayerTarget
/// Finds the closest player entity within the mob's targeting range and creates a TargetComponent
pub(super) fn find_player_target_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mob_q: Query<(Entity, &Transform, &MobAttributesComponent), With<MobAttributesComponent>>,
    player_q: Query<(Entity, &Transform), With<PlayerStats>>,
    mut cmds: Commands,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mob_entity, mob_transform, attributes)) = mob_q.get(ctx.target_entity()) else {
            continue;
        };

        if mob_behavior
            .behaviors
            .contains(&MobBehaviorType::FindPlayerTarget)
        {
            find_player_target(
                mob_entity,
                mob_transform,
                attributes,
                &player_q,
                &mut cmds,
                ctx,
            );
        }
    }
}

fn find_player_target(
    mob_entity: Entity,
    mob_transform: &Transform,
    attributes: &MobAttributesComponent,
    player_q: &Query<(Entity, &Transform), With<PlayerStats>>,
    cmds: &mut Commands,
    ctx: &BehaveCtx,
) {
    let mob_pos = mob_transform.translation.truncate();

    let closest_player = player_q
        .iter()
        .filter_map(|(entity, player_transform)| {
            let player_pos = player_transform.translation.truncate();
            let distance_squared = mob_pos.distance_squared(player_pos);

            // If targeting_range is Some, only consider players within that range
            // If targeting_range is None, consider all players (infinite range)
            if let Some(range) = attributes.targeting_range
                && distance_squared > range * range
            {
                return None; // Player is outside targeting range
            }

            Some((entity, distance_squared))
        })
        .min_by_key(|(_, distance_squared)| {
            (*distance_squared * 1000.0) as u32 // multiplied by 1000 to avoid floating point precision issues
        })
        .map(|(entity, _)| entity);

    if let Some(closest_player_entity) = closest_player {
        cmds.entity(mob_entity)
            .insert(TargetComponent(closest_player_entity));
        cmds.trigger(ctx.success());
    }
}

/// MobBehaviorType::LoseTarget
/// Remove the TargetComponent when the targetted entity is out of range
pub(super) fn lose_target_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<
        (
            Entity,
            &TargetComponent,
            &Transform,
            &MobAttributesComponent,
        ),
        With<MobAttributesComponent>,
    >,
    target_q: Query<&Transform>,
    mut cmds: Commands,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mob_entity, target, mob_transform, attributes)) =
            mob_q.get_mut(ctx.target_entity())
        else {
            continue;
        };

        if mob_behavior
            .behaviors
            .contains(&MobBehaviorType::LoseTarget)
        {
            lose_target(
                mob_entity,
                target,
                mob_transform,
                attributes,
                &target_q,
                &mut cmds,
                ctx,
            );
        }
    }
}

fn lose_target(
    mob_entity: Entity,
    target: &TargetComponent,
    mob_transform: &Transform,
    attributes: &MobAttributesComponent,
    target_q: &Query<&Transform>,
    cmds: &mut Commands,
    ctx: &BehaveCtx,
) {
    let Ok(target_transform) = target_q.get(target.0) else {
        // Target entity doesn't exist anymore, remove target
        cmds.entity(mob_entity).remove::<TargetComponent>();
        cmds.trigger(ctx.success());
        return;
    };

    let mob_pos = mob_transform.translation.truncate();
    let target_pos = target_transform.translation.truncate();
    let distance_squared = mob_pos.distance_squared(target_pos);

    // Check if target is out of range (if range is specified)
    if let Some(range) = attributes.targeting_range
        && distance_squared > range * range
    {
        cmds.entity(mob_entity).remove::<TargetComponent>();
        cmds.trigger(ctx.success());
    }
}

/// MobBehaviorType::MoveToTarget
/// Move mob to entity specified in Target component
pub(super) fn move_to_target_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(
        Entity,
        &TargetComponent,
        &mut LinearVelocity,
        &Transform,
        &MobAttributesComponent,
    )>,
    target_q: Query<&Transform>,
    mut cmds: Commands,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mob_entity, target, mut lin_vel, transform, attributes)) =
            mob_q.get_mut(ctx.target_entity())
        else {
            continue;
        };

        if mob_behavior
            .behaviors
            .contains(&MobBehaviorType::MoveToTarget)
        {
            move_to_target(
                mob_entity,
                target,
                &mut lin_vel,
                transform,
                attributes,
                &target_q,
                &mut cmds,
                ctx,
            );
        }
    }
}

fn move_to_target(
    mob_entity: Entity,
    target: &TargetComponent,
    lin_vel: &mut LinearVelocity,
    transform: &Transform,
    attributes: &MobAttributesComponent,
    target_q: &Query<&Transform>,
    cmds: &mut Commands,
    ctx: &BehaveCtx,
) {
    let Ok(target_transform) = target_q.get(target.0) else {
        cmds.entity(mob_entity).remove::<TargetComponent>();
        cmds.trigger(ctx.success());
        return;
    };

    let current_pos = transform.translation.truncate();
    let target_pos = target_transform.translation.truncate();
    let direction = (target_pos - current_pos).normalize_or_zero();

    let target_velocity = direction * attributes.max_linear_speed;
    let velocity_diff = target_velocity - lin_vel.0;

    // Apply acceleration towards target velocity, clamped by acceleration limits
    lin_vel.x += (velocity_diff.x * attributes.linear_acceleration.x).clamp(
        -attributes.linear_acceleration.x,
        attributes.linear_acceleration.x,
    );
    lin_vel.y += (velocity_diff.y * attributes.linear_acceleration.y).clamp(
        -attributes.linear_acceleration.y,
        attributes.linear_acceleration.y,
    );
}

/// MobBehaviorType::RotateToTarget
/// Rotates mob to face the entity specified in the Target component
pub(super) fn rotate_to_target_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(
        Entity,
        &TargetComponent,
        &mut AngularVelocity,
        &Transform,
        &MobAttributesComponent,
    )>,
    target_q: Query<&Transform>,
    mut cmds: Commands,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mob_entity, target, mut ang_vel, transform, attributes)) =
            mob_q.get_mut(ctx.target_entity())
        else {
            continue;
        };

        if mob_behavior
            .behaviors
            .contains(&MobBehaviorType::RotateToTarget)
        {
            rotate_to_target(
                mob_entity,
                target,
                &mut ang_vel,
                transform,
                attributes,
                &target_q,
                &mut cmds,
                ctx,
            );
        }
    }
}

fn rotate_to_target(
    mob_entity: Entity,
    target: &TargetComponent,
    ang_vel: &mut AngularVelocity,
    transform: &Transform,
    attributes: &MobAttributesComponent,
    target_q: &Query<&Transform>,
    cmds: &mut Commands,
    ctx: &BehaveCtx,
) {
    let Ok(target_transform) = target_q.get(target.0) else {
        cmds.entity(mob_entity).remove::<TargetComponent>();
        cmds.trigger(ctx.success());
        return;
    };

    let current_pos = transform.translation.truncate();
    let target_pos = target_transform.translation.truncate();
    let direction = target_pos - current_pos;

    if direction.length_squared() > 0.001 {
        let target_angle = direction.y.atan2(direction.x) + std::f32::consts::FRAC_PI_2;
        let current_angle = transform.rotation.to_euler(bevy::math::EulerRot::ZYX).0;

        let mut angle_diff = target_angle - current_angle;

        // Normalize angle difference to [-π, π]
        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        let target_angular_velocity = angle_diff.signum() * attributes.max_angular_speed;
        let velocity_diff = target_angular_velocity - ang_vel.0;

        // Apply acceleration towards target angular velocity, clamped by acceleration limits
        ang_vel.0 += (velocity_diff * attributes.angular_acceleration).clamp(
            -attributes.angular_acceleration,
            attributes.angular_acceleration,
        );
    }
}

/// MobBehaviorType::BrakeAngular
/// Applies angular deceleration to the mob until it stops rotating
pub(super) fn brake_angular_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut AngularVelocity, &MobAttributesComponent)>,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mut ang_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        if mob_behavior
            .behaviors
            .contains(&MobBehaviorType::BrakeAngular)
        {
            brake_angular(&mut ang_vel, attributes);
        }
    }
}

fn brake_angular(ang_vel: &mut AngularVelocity, attributes: &MobAttributesComponent) {
    let target_angular_velocity = 0.0;
    let velocity_diff = target_angular_velocity - ang_vel.0;

    // Apply deceleration towards target angular velocity, clamped by deceleration limits
    ang_vel.0 += (velocity_diff * attributes.angular_deceleration).clamp(
        -attributes.angular_deceleration,
        attributes.angular_deceleration,
    );
}

/// MobBehaviorType::MoveForward
/// Applies linear acceleration to the mob in the direction that it is facing
pub(super) fn move_forward_system(
    move_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &Transform, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, transform, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        if move_behavior
            .behaviors
            .contains(&MobBehaviorType::MoveForward)
        {
            move_forward(&mut lin_vel, transform, attributes);
        }
    }
}

fn move_forward(
    lin_vel: &mut LinearVelocity,
    transform: &Transform,
    attributes: &MobAttributesComponent,
) {
    // Get the forward direction based on the mob's rotation
    // In Bevy, the default forward direction is negative Y, but we need to account for rotation
    let rotation = transform.rotation;
    let forward_direction = rotation * bevy::math::Vec3::NEG_Y;
    let forward_2d = forward_direction.truncate().normalize_or_zero();

    // Calculate target velocity in the forward direction
    let target_velocity = forward_2d * attributes.max_linear_speed;
    let velocity_diff = target_velocity - lin_vel.0;

    // Apply acceleration towards target velocity, clamped by acceleration limits
    lin_vel.x += (velocity_diff.x * attributes.linear_acceleration.x).clamp(
        -attributes.linear_acceleration.x,
        attributes.linear_acceleration.x,
    );
    lin_vel.y += (velocity_diff.y * attributes.linear_acceleration.y).clamp(
        -attributes.linear_acceleration.y,
        attributes.linear_acceleration.y,
    );
}

/// MobBehaviorType::SpawnMob
/// Spawns mobs using the MobSpawnerComponent, using the given spawner keys
pub(super) fn spawn_mob_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(&mut MobSpawnerComponent, &Transform)>,
    mut spawn_mob_event_writer: EventWriter<SpawnMobEvent>,
    time: Res<Time>,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mut mob_spawner, transform)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        for behavior in mob_behavior.behaviors.iter() {
            if let MobBehaviorType::SpawnMob(Some(spawner_keys)) = behavior {
                spawn_mob(
                    spawner_keys,
                    &mut mob_spawner,
                    transform,
                    &mut spawn_mob_event_writer,
                    &time,
                );
            }
        }
    }
}

fn spawn_mob(
    spawner_keys: &[String],
    mob_spawner: &mut MobSpawnerComponent,
    transform: &Transform,
    spawn_mob_event_writer: &mut EventWriter<SpawnMobEvent>,
    time: &Res<Time>,
) {
    for key in spawner_keys.iter() {
        if let Some(spawner) = mob_spawner.spawners.get_mut(key)
            && spawner.timer.tick(time.delta()).just_finished()
        {
            spawn_mob_event_writer.write(SpawnMobEvent {
                mob_type: spawner.mob_type.clone(),
                rotation: spawner.rotation,
                position: transform.translation.truncate() + spawner.position,
            });
        }
    }
}

/// MobBehaviorType::SpawnProjectile
/// Spawns projectiles using the ProjectileSpawnerComponent, using the given spawner keys
pub(super) fn spawn_projectile_system(
    mob_behavior_q: Query<(&MobBehaviorComponent, &BehaveCtx)>,
    mut mob_q: Query<(
        &mut ProjectileSpawnerComponent,
        &Transform,
        &MobAttributesComponent,
    )>,
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
    mut activate_particle_event_writer: EventWriter<ActivateParticleEvent>,
    time: Res<Time>,
) {
    for (mob_behavior, ctx) in mob_behavior_q.iter() {
        let Ok((mut projectile_spawner, transform, attributes)) =
            mob_q.get_mut(ctx.target_entity())
        else {
            continue;
        };

        for behavior in mob_behavior.behaviors.iter() {
            if let MobBehaviorType::SpawnProjectile(Some(spawner_keys)) = behavior {
                spawn_projectile(
                    spawner_keys,
                    &mut projectile_spawner,
                    transform,
                    attributes,
                    &mut spawn_projectile_event_writer,
                    &mut activate_particle_event_writer,
                    &time,
                );
            }
        }
    }
}

fn spawn_projectile(
    spawner_keys: &[String],
    projectile_spawner: &mut ProjectileSpawnerComponent,
    transform: &Transform,
    attributes: &MobAttributesComponent,
    spawn_projectile_event_writer: &mut EventWriter<SpawnProjectileEvent>,
    activate_particle_event_writer: &mut EventWriter<ActivateParticleEvent>,
    time: &Res<Time>,
) {
    for key in spawner_keys.iter() {
        if let Some(spawner) = projectile_spawner.spawners.get_mut(key) {
            let timer_just_finished = spawner.timer.tick(time.delta()).just_finished();

            // start the spawn animation particle effect
            if spawner.timer.remaining_secs() <= spawner.pre_spawn_animation_start_time
                && let Some(particle_effect_entity) = spawner.spawn_effect_entity
            {
                activate_particle_event_writer.write(ActivateParticleEvent {
                    entity: particle_effect_entity,
                    active: true,
                });
            }

            // stop the spawn animation particle effect
            if spawner.timer.remaining_secs() <= spawner.pre_spawn_animation_end_time
                && let Some(particle_effect_entity) = spawner.spawn_effect_entity
            {
                activate_particle_event_writer.write(ActivateParticleEvent {
                    entity: particle_effect_entity,
                    active: false,
                });
            }

            if timer_just_finished {
                spawn_projectile_event_writer.write(SpawnProjectileEvent {
                    projectile_type: spawner.projectile_type.clone(),
                    rotation: spawner.rotation,
                    position: transform.translation.truncate() + spawner.position,
                    faction: spawner.faction.clone(),
                    speed: spawner.speed_multiplier * attributes.projectile_speed,
                    damage: (spawner.damage_multiplier * attributes.projectile_damage as f32)
                        as u32,
                    range_seconds: spawner.range_seconds_multiplier
                        * attributes.projectile_range_seconds,
                });
            }
        }
    }
}

/// MobBehaviorType::DoForTime
/// Triggers success when the timer is finsihed to progress the behavior tree
pub(super) fn do_for_time_system(
    mut mob_behavior_q: Query<(&mut MobBehaviorComponent, &BehaveCtx)>,
    mut cmds: Commands,
    time: Res<Time>,
) {
    for (mut mob_behavior, ctx) in mob_behavior_q.iter_mut() {
        for behavior in mob_behavior.behaviors.iter_mut() {
            if let MobBehaviorType::DoForTime(timer) = behavior {
                do_for_time(timer, &time, &mut cmds, ctx);
            }
        }
    }
}

fn do_for_time(timer: &mut Timer, time: &Res<Time>, cmds: &mut Commands, ctx: &BehaveCtx) {
    if timer.tick(time.delta()).just_finished() {
        // Perform the action when the timer finishes
        cmds.trigger(ctx.success());
    }
}

/// MobBehaviorType::RotateJointsClockwise
/// Uses joint motor to rotate joint
/// Waiting on motor implementation for Avian physics
#[allow(dead_code)]
pub(super) fn rotate_clockwise_system(
    mut mob_behavior_q: Query<(&mut MobBehaviorComponent, &BehaveCtx)>,
    mob_q: Query<&JointsComponent>,
    mut joints_q: Query<&mut RevoluteJoint>,
) {
    for (mut mob_behavior, ctx) in mob_behavior_q.iter_mut() {
        let Ok(joints) = mob_q.get(ctx.target_entity()) else {
            continue;
        };

        for behavior in mob_behavior.behaviors.iter_mut() {
            if let MobBehaviorType::RotateJointsClockwise(keys) = behavior {
                rotate_joints_clockwise(keys, joints, &mut joints_q);
            }
        }
    }
}

#[allow(dead_code)]
fn rotate_joints_clockwise(
    keys: &Vec<String>,
    joints: &JointsComponent,
    joints_q: &mut Query<&mut RevoluteJoint>,
) {
    for joint_key in keys {
        if let Some(joint_entity) = joints.joints.get(joint_key)
            && let Ok(_revolute_joint) = joints_q.get_mut(*joint_entity)
        {
            // rotate the revolute joint, will require joint motors
        }
    }
}
