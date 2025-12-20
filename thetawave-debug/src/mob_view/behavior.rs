use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Query, Res, ResMut},
    },
    platform::collections::HashMap,
    prelude::Children,
};
use bevy_behave::prelude::{BehaveCtx, BehaveTree};
use thetawave_mobs::{
    BehaviorActionName, BehaviorReceiverComponent, MobBehaviorComponent, MobMarker,
};

use super::data::{BehaviorTreeDisplay, BehaviorTreeDisplays, MobGroupRegistry};

/// Stores the active action node name and behavior list for a target entity
struct ActiveBehaviorInfo {
    node_name: Option<String>,
    behaviors: Vec<String>,
}

/// System that collects behavior tree display data for all mobs in the selected group
pub fn collect_behavior_tree_display_system(
    registry: Res<MobGroupRegistry>,
    mut tree_displays: ResMut<BehaviorTreeDisplays>,
    mob_query: Query<
        (
            Entity,
            &MobMarker,
            Option<&Children>,
            Option<&BehaviorReceiverComponent>,
        ),
        With<MobMarker>,
    >,
    behavior_tree_query: Query<&BehaveTree>,
    active_behavior_query: Query<(
        &MobBehaviorComponent,
        &BehaveCtx,
        Option<&BehaviorActionName>,
    )>,
) {
    tree_displays.displays.clear();

    let Some(selected) = registry.selected_group else {
        return;
    };

    let Some(group) = registry.groups.get(&selected) else {
        return;
    };

    // Collect active action info for all entities
    // The BehaveCtx.target_entity() gives us the mob entity the behavior tree operates on
    let mut active_info_by_target: HashMap<Entity, ActiveBehaviorInfo> = HashMap::new();

    for (behavior_comp, ctx, action_name) in active_behavior_query.iter() {
        let target = ctx.target_entity();
        let behavior_names: Vec<String> = behavior_comp
            .behaviors
            .iter()
            .map(|b| format!("{:?}", b))
            .collect();

        let entry = active_info_by_target
            .entry(target)
            .or_insert_with(|| ActiveBehaviorInfo {
                node_name: None,
                behaviors: Vec::new(),
            });

        // Store the action node name if available
        if let Some(name) = action_name {
            entry.node_name = Some(name.0.clone());
        }
        entry.behaviors.extend(behavior_names);
    }

    // Collect displays for ALL members of the group
    for &entity in &group.members {
        if let Ok((_, marker, children, behavior_receiver)) = mob_query.get(entity) {
            // Check if this mob has a behavior tree child
            let has_tree = children
                .map(|c| {
                    c.iter()
                        .any(|child| behavior_tree_query.get(*child).is_ok())
                })
                .unwrap_or(false);

            let info = active_info_by_target.get(&entity);
            tree_displays.displays.push(BehaviorTreeDisplay {
                mob_entity: entity,
                mob_type: marker.mob_type().to_string(),
                has_own_tree: has_tree,
                receives_from: behavior_receiver.map(|r| r.0),
                active_node_name: info.and_then(|i| i.node_name.clone()),
                active_actions: info.map(|i| i.behaviors.clone()).unwrap_or_default(),
            });
        }
    }
}
