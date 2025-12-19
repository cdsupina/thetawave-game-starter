//! Loading UI systems.

use bevy::{
    color::Color,
    prelude::{Commands, MessageReader, Query, With},
    ui::{BackgroundColor, Node, Val},
    utils::default,
};

use crate::ui::data::LoadingBar;

use super::{AppState, Cleanup, LoadingProgressEvent};

/// Setup loading bar ui
pub(in crate::ui) fn setup_loading_ui_system(mut cmds: Commands) {
    cmds.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0.05, 0.05, 0.05, 0.1)),
        LoadingBar,
        Cleanup::<AppState> {
            states: vec![AppState::MainMenuLoading, AppState::GameLoading],
        },
    ));
}

/// Update the loading bars based on the loading bar progress
pub(in crate::ui) fn update_loading_bar_system(
    mut loading_bar_q: Query<&mut Node, With<LoadingBar>>,
    mut loading_event_reader: MessageReader<LoadingProgressEvent>,
) {
    for event in loading_event_reader.read() {
        for mut node in loading_bar_q.iter_mut() {
            node.width = Val::Percent((1.0 - event.0) * 100.0);
        }
    }
}
