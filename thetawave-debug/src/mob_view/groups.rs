use avian2d::prelude::RevoluteJoint;
use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Query, ResMut},
    },
    math::Vec2,
    platform::collections::{HashMap, HashSet},
    transform::components::Transform,
};
use thetawave_mobs::{JointsComponent, MobMarker};

use super::data::{MobGroup, MobGroupRegistry};

/// System that discovers and updates mob groups by traversing joint connections
pub fn update_mob_groups_system(
    mut registry: ResMut<MobGroupRegistry>,
    mob_query: Query<(Entity, &Transform), With<MobMarker>>,
    joint_query: Query<&RevoluteJoint>,
    joints_component_query: Query<Entity, With<JointsComponent>>,
) {
    let mut visited: HashSet<Entity> = HashSet::new();
    let mut new_groups: HashMap<Entity, MobGroup> = HashMap::new();

    // Build adjacency from joints - only include entities that are mobs
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for joint in joint_query.iter() {
        // Only add connection if both entities are mobs
        if mob_query.get(joint.body1).is_ok() && mob_query.get(joint.body2).is_ok() {
            adjacency.entry(joint.body1).or_default().push(joint.body2);
            adjacency.entry(joint.body2).or_default().push(joint.body1);
        }
    }

    // Find connected components using BFS
    for (entity, _) in mob_query.iter() {
        if visited.contains(&entity) {
            continue;
        }

        // BFS to find all connected mobs
        let mut group_members = Vec::new();
        let mut queue = vec![entity];
        let mut group_key = entity;

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if mob_query.get(current).is_ok() {
                group_members.push(current);

                // Prefer using a behavior transmitter (has JointsComponent) as the group key
                if joints_component_query.get(current).is_ok() {
                    group_key = current;
                }
            }

            // Add connected entities to queue
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push(*neighbor);
                    }
                }
            }
        }

        if !group_members.is_empty() {
            // Calculate center position
            let center = calculate_group_center(&group_members, &mob_query);

            new_groups.insert(
                group_key,
                MobGroup {
                    members: group_members,
                    center_position: center,
                },
            );
        }
    }

    registry.groups = new_groups;
    registry.rebuild_order();
}

fn calculate_group_center(
    members: &[Entity],
    mob_query: &Query<(Entity, &Transform), With<MobMarker>>,
) -> Vec2 {
    if members.is_empty() {
        return Vec2::ZERO;
    }

    let sum: Vec2 = members
        .iter()
        .filter_map(|e| mob_query.get(*e).ok())
        .map(|(_, t)| t.translation.truncate())
        .sum();

    sum / members.len() as f32
}
