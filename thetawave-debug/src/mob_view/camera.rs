use bevy::{
    ecs::{
        query::With,
        system::{Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    transform::components::Transform,
};

use super::data::{MobGroupRegistry, MobViewCamera, MobViewWindowState};

/// System that makes the mob view camera follow the selected group's center
/// Note: This system only runs when mob view is open (controlled by run_if in plugin)
pub fn mob_view_camera_follow_system(
    registry: Res<MobGroupRegistry>,
    mut camera_query: Query<&mut Transform, With<MobViewCamera>>,
) {
    let Some(selected) = registry.selected_group else {
        return;
    };

    let Some(group) = registry.groups.get(&selected) else {
        return;
    };

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        // Smooth follow to group center
        let target = group.center_position.extend(camera_transform.translation.z);
        camera_transform.translation = camera_transform.translation.lerp(target, 0.1);
    }
}

/// System that handles zoom for the mob view camera via transform scale
/// Note: This system only runs when mob view is open (controlled by run_if in plugin)
pub fn mob_view_camera_zoom_system(
    mut state: ResMut<MobViewWindowState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MobViewCamera>>,
) {
    // Zoom with +/- keys
    let zoom_delta = if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        -0.02
    } else if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        0.02
    } else {
        0.0
    };

    if zoom_delta != 0.0 {
        state.zoom_level = (state.zoom_level + zoom_delta).clamp(0.5, 3.0);

        if let Ok(mut transform) = camera_query.single_mut() {
            transform.scale.x = state.zoom_level;
            transform.scale.y = state.zoom_level;
        }
    }
}
