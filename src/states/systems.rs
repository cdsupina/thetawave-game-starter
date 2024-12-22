use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, With};

/// A system that cleans up entities marked with a specific component type
///
/// # Type Parameters
/// * `T` - Component type used to identify entities for cleanup
///
/// # Arguments
/// * `cmds` - Commands to execute despawn operations
/// * `cleanup_entities_q` - Query to find entities with component T
pub(super) fn cleanup_state_system<T: Component>(
    mut cmds: Commands,
    cleanup_entities_q: Query<Entity, With<T>>,
) {
    // Iterate through all entities with component T and despawn them
    for e in cleanup_entities_q.iter() {
        cmds.entity(e).despawn_recursive();
    }
}
