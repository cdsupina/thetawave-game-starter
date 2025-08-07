use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::{
    ecs::{
        entity::Entity,
        event::EventWriter,
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_behave::prelude::BehaveCtx;
use thetawave_player::PlayerStats;

use crate::{
    SpawnMobEvent,
    attributes::{MobAttributesComponent, MobSpawnerComponent},
    behavior::{MobBehavior, MobBehaviorType, data::Target},
};

pub(super) fn move_down_system(
    move_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        #[allow(clippy::collapsible_if)]
        if move_behavior.behaviors.contains(&MobBehaviorType::MoveDown) {
            if lin_vel.y > -attributes.max_linear_speed.y {
                lin_vel.y -= attributes.linear_acceleration.y;
            }
        }
    }
}

pub(super) fn brake_horizontal_system(
    move_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        #[allow(clippy::collapsible_if)]
        if move_behavior
            .behaviors
            .contains(&MobBehaviorType::BrakeHorizontal)
        {
            if lin_vel.x > 0.0 {
                lin_vel.x = (lin_vel.x - attributes.linear_deceleration.x).max(0.0);
            } else if lin_vel.x < 0.0 {
                lin_vel.x = (lin_vel.x + attributes.linear_deceleration.x).min(0.0);
            }
        }
    }
}

pub(super) fn move_to_system(
    move_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &Transform, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, transform, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        for behavior in move_behavior.behaviors.iter() {
            if let MobBehaviorType::MoveTo(target_pos) = behavior {
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
        }
    }
}

pub(super) fn find_player_target_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
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
                    .insert(Target(closest_player_entity));
                cmds.trigger(ctx.success());
            }
        }
    }
}

pub(super) fn lose_target_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<
        (Entity, &Target, &Transform, &MobAttributesComponent),
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
            let Ok(target_transform) = target_q.get(target.0) else {
                // Target entity doesn't exist anymore, remove target
                cmds.entity(mob_entity).remove::<Target>();
                cmds.trigger(ctx.success());
                continue;
            };

            let mob_pos = mob_transform.translation.truncate();
            let target_pos = target_transform.translation.truncate();
            let distance_squared = mob_pos.distance_squared(target_pos);

            // Check if target is out of range (if range is specified)
            if let Some(range) = attributes.targeting_range
                && distance_squared > range * range
            {
                cmds.entity(mob_entity).remove::<Target>();
                cmds.trigger(ctx.success());
            }
        }
    }
}

pub(super) fn move_to_target_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<(
        Entity,
        &Target,
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
            let Ok(target_transform) = target_q.get(target.0) else {
                cmds.entity(mob_entity).remove::<Target>();
                cmds.trigger(ctx.success());
                continue;
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
    }
}

pub(super) fn rotate_to_target_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
    mut mob_q: Query<(
        Entity,
        &Target,
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
            let Ok(target_transform) = target_q.get(target.0) else {
                cmds.entity(mob_entity).remove::<Target>();
                cmds.trigger(ctx.success());
                continue;
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
    }
}

pub(super) fn brake_angular_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
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
            let target_angular_velocity = 0.0;
            let velocity_diff = target_angular_velocity - ang_vel.0;

            // Apply deceleration towards target angular velocity, clamped by deceleration limits
            ang_vel.0 += (velocity_diff * attributes.angular_deceleration).clamp(
                -attributes.angular_deceleration,
                attributes.angular_deceleration,
            );
        }
    }
}

pub(super) fn move_forward_system(
    move_q: Query<(&MobBehavior, &BehaveCtx)>,
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
    }
}

pub(super) fn spawn_mob_system(
    mob_behavior_q: Query<(&MobBehavior, &BehaveCtx)>,
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
        }
    }
}

pub(super) fn do_for_time_system(
    mut mob_behavior_q: Query<(&mut MobBehavior, &BehaveCtx)>,
    mut cmds: Commands,
    time: Res<Time>,
) {
    for (mut mob_behavior, ctx) in mob_behavior_q.iter_mut() {
        for behavior in mob_behavior.behaviors.iter_mut() {
            if let MobBehaviorType::DoForTime(timer) = behavior {
                if timer.tick(time.delta()).just_finished() {
                    // Perform the action when the timer finishes
                    cmds.trigger(ctx.success());
                }
            }
        }
    }
}
