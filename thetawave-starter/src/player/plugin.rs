use super::systems::spawn_players_system;
use bevy::{
    app::Plugin,
    ecs::{
        entity::Entity,
        system::{In, SystemId},
    },
    prelude::OnEnter,
};
use bevy_platform::collections::{HashMap, HashSet};
use thetawave_core::AppState;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin {
    pub extended_abilities: HashMap<String, SystemId<In<Entity>>>,
    pub extended_duration_abilities: HashSet<String>,
}

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(thetawave_player::ThetawavePlayerPlugin {
            extended_abilities: self.extended_abilities.clone(),
            extended_duration_abilities: self.extended_duration_abilities.clone(),
        })
        .add_systems(OnEnter(AppState::Game), spawn_players_system);
    }
}
