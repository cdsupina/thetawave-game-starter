use crate::{
    assets::GameAssets,
    states::{AppState, Cleanup},
};
use bevy::{
    core::Name,
    prelude::{Commands, Res},
};
use bevy_aseprite_ultra::prelude::{Animation, AseSpriteAnimation};

pub(super) fn spawn_players_system(mut cmds: Commands, assets: Res<GameAssets>) {
    cmds.spawn((
        AseSpriteAnimation {
            animation: Animation::tag("idle"),
            aseprite: assets.captain_character_aseprite.clone(),
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Name::new("Player"),
    ));
}
