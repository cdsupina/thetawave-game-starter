use avian2d::prelude::LinearVelocity;
use bevy::ecs::system::Query;
use bevy_behave::prelude::BehaveCtx;

use crate::{
    attributes::MobAttributesComponent,
    behavior::{MoveBehavior, MoveBehaviorType},
};

pub(super) fn move_system(
    move_q: Query<(&MoveBehavior, &BehaveCtx)>,
    mut mob_q: Query<(&mut LinearVelocity, &MobAttributesComponent)>,
) {
    for (move_behavior, ctx) in move_q.iter() {
        let Ok((mut lin_vel, attributes)) = mob_q.get_mut(ctx.target_entity()) else {
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
            }
        }
    }
}
