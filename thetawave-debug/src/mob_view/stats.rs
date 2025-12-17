use avian2d::prelude::{AngularVelocity, Friction, LinearVelocity, Restitution};
use bevy::{
    ecs::{
        entity::Entity,
        name::Name,
        query::With,
        system::{Query, Res, ResMut},
    },
    transform::components::Transform,
};
use thetawave_core::HealthComponent;
use thetawave_mobs::{
    BehaviorReceiverComponent, JointsComponent, MobAttributesComponent, MobMarker,
};

use super::data::{MobDisplayStats, MobGroupDisplayStats, MobGroupRegistry};

/// System that collects stats for all mobs in the selected group
pub fn collect_mob_stats_system(
    registry: Res<MobGroupRegistry>,
    mut group_stats: ResMut<MobGroupDisplayStats>,
    mob_query: Query<
        (
            Entity,
            &MobMarker,
            &Transform,
            &HealthComponent,
            &MobAttributesComponent,
            &LinearVelocity,
            &AngularVelocity,
            &Restitution,
            &Friction,
            Option<&BehaviorReceiverComponent>,
        ),
        With<MobMarker>,
    >,
    name_query: Query<&Name>,
    joints_query: Query<&JointsComponent>,
) {
    let Some(selected) = registry.selected_group else {
        *group_stats = MobGroupDisplayStats::default();
        return;
    };

    let Some(group) = registry.groups.get(&selected) else {
        *group_stats = MobGroupDisplayStats::default();
        return;
    };

    let mut member_stats = Vec::new();
    let mut total_health = 0u32;
    let mut max_total_health = 0u32;

    for &entity in &group.members {
        if let Ok((
            e,
            marker,
            transform,
            health,
            attributes,
            lin_vel,
            ang_vel,
            restitution,
            friction,
            behavior_receiver,
        )) = mob_query.get(entity)
        {
            let name = name_query
                .get(entity)
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|_| format!("Entity {:?}", entity));

            total_health += health.current_health;
            max_total_health += health.max_health;

            member_stats.push(MobDisplayStats {
                entity: e,
                mob_type: marker.mob_type().to_string(),
                name,
                current_health: health.current_health,
                max_health: health.max_health,
                linear_velocity: lin_vel.0,
                angular_velocity: ang_vel.0,
                max_linear_speed: attributes.max_linear_speed,
                linear_acceleration: attributes.linear_acceleration,
                max_angular_speed: attributes.max_angular_speed,
                angular_acceleration: attributes.angular_acceleration,
                position: transform.translation.truncate(),
                // to_euler is safe in Bevy - returns valid angles even for edge cases
                rotation: transform.rotation.to_euler(bevy::math::EulerRot::ZYX).0.to_degrees(),
                restitution: restitution.coefficient,
                friction: friction.dynamic_coefficient,
                projectile_speed: attributes.projectile_speed,
                projectile_damage: attributes.projectile_damage,
                targeting_range: attributes.targeting_range,
                is_behavior_transmitter: joints_query.get(entity).is_ok(),
                behavior_receiver_from: behavior_receiver.map(|b| b.0),
            });
        }
    }

    *group_stats = MobGroupDisplayStats {
        group_center: group.center_position,
        total_health,
        max_total_health,
        member_stats,
    };
}
