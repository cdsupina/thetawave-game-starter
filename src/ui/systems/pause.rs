use crate::ui::data::{ButtonAction, UiChildBuilderExt};

use super::{AppState, Cleanup, GameState, PauseMenuState, UiAssets};
use bevy::{
    color::Color,
    core::Name,
    hierarchy::BuildChildren,
    prelude::{Commands, Res},
    ui::{AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, Val},
    utils::default,
};
use bevy_hui::prelude::HtmlNode;

/// Spawns the pause menu ui
pub(in crate::ui) fn spawn_pause_menu_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    cmds.spawn((
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
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
    ))
    .with_children(|parent| {
        parent.spawn_menu_button(
            &ui_assets,
            ButtonAction::EnterGameState(GameState::Playing),
            300.0,
            true,
            false,
        );

        parent.spawn_menu_button(
            &ui_assets,
            ButtonAction::EnterPauseMenuState(PauseMenuState::Options),
            300.0,
            false,
            false,
        );

        parent.spawn_menu_button(
            &ui_assets,
            ButtonAction::EnterAppState(AppState::MainMenuLoading),
            300.0,
            false,
            false,
        );
    });
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
