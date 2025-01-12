use super::{AppState, Cleanup, GameState, PauseMenuState, UiAssets};
use bevy::{
    core::Name,
    prelude::{Commands, Res},
};
use bevy_hui::prelude::HtmlNode;

/// This system sets up the title menu interface.
/// It spawns the main menu HTML node and associates the cleanup component with it.
pub(in crate::ui) fn setup_pause_menu_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    // Create an HTMLNode with main menu HTML and link the TitleMenuCleanup component.

    cmds.spawn((
        HtmlNode(ui_assets.pause_menu_html.clone()),
        Cleanup::<GameState> {
            states: vec![GameState::Paused],
        },
        Cleanup::<PauseMenuState> {
            states: vec![PauseMenuState::Main],
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Name::new("Pause Menu"),
    ));
}

/// This system sets up the title menu interface.
/// It spawns the main menu HTML node and associates the cleanup component with it.
pub(in crate::ui) fn setup_pause_options_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    // Create an HTMLNode with main menu HTML and link the TitleMenuCleanup component.
    cmds.spawn((
        HtmlNode(ui_assets.options_pause_menu_html.clone()),
        Cleanup::<GameState> {
            states: vec![GameState::Paused],
        },
        Cleanup::<PauseMenuState> {
            states: vec![PauseMenuState::Options],
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Name::new("Pause Menu"),
    ));
}
