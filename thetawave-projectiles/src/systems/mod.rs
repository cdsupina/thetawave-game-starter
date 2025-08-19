use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::Time,
};

use crate::attributes::ProjectileRangeComponent;

/// Despawns projectiles after a set amount of time passes
pub(crate) fn timed_range_system(
    mut cmds: Commands,
    mut projectile_q: Query<(Entity, &mut ProjectileRangeComponent)>,
    time: Res<Time>,
) {
    for (entity, mut range) in projectile_q.iter_mut() {
        if range.timer.tick(time.delta()).just_finished() {
            cmds.entity(entity).despawn();
        }
    }
}
