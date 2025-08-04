use avian2d::prelude::LinearVelocity;
use bevy::{ecs::system::Query, transform::components::Transform};
use bevy_behave::prelude::BehaveCtx;

use crate::{
    attributes::MobAttributesComponent,
    behavior::{MoveBehavior, MoveBehaviorType},
};

pub(super) fn move_system(
    move_q: Query<(&MoveBehavior, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &Transform, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, transform, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
            continue;
        };

        for behavior in move_behavior.behaviors.iter() {
            match behavior {
                MoveBehaviorType::MoveDown => {
                    if lin_vel.y < -attributes.max_linear_speed.y {
                        lin_vel.y += attributes.linear_deceleration.y;
                    } else {
                        lin_vel.y -= attributes.linear_acceleration.y;
                    }
                }
                MoveBehaviorType::BrakeHorizontal => {
                    if lin_vel.x > 0.0 {
                        lin_vel.x = (lin_vel.x - attributes.linear_deceleration.x).max(0.0);
                    } else if lin_vel.x < 0.0 {
                        lin_vel.x = (lin_vel.x + attributes.linear_deceleration.x).min(0.0);
                    }
                }
                MoveBehaviorType::MoveTo(target_pos) => {
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

                    // Limit to max linear speed
                    lin_vel.x = lin_vel.x.clamp(
                        -attributes.max_linear_speed.x,
                        attributes.max_linear_speed.x,
                    );
                    lin_vel.y = lin_vel.y.clamp(
                        -attributes.max_linear_speed.y,
                        attributes.max_linear_speed.y,
                    );
                }
            }
        }
    }
}
