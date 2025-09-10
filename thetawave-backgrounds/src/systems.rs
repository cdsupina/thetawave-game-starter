use bevy::{
    ecs::system::{Query, Res},
    time::Time,
    transform::components::Transform,
};

use crate::data::PlanetRotationComponent;

/// System to handle planet rotation animation
pub(super) fn rotate_planet_system(
    mut planet_q: Query<(&mut Transform, &PlanetRotationComponent)>,
    time: Res<Time>,
) {
    for (mut transform, planet_rotation) in planet_q.iter_mut() {
        transform.rotate_y(planet_rotation.rotation_speed * time.delta_secs());
    }
}
