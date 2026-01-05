use crate::ui::data::{ButtonAction, UiChildBuilderExt};

use super::{AppState, Cleanup, GameState, PauseMenuState, UiAssets};
use bevy::{
    color::Color,
    prelude::{Commands, Name, Res},
    ui::{AlignItems, BackgroundColor, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use thetawave_assets::{ExtendedUiAssets, ModUiAssets};

/// Spawns the pause menu ui
pub(in crate::ui) fn spawn_pause_menu_system(
    mut cmds: Commands,
    mod_ui_assets: Res<ModUiAssets>,
    extended_ui_assets: Res<ExtendedUiAssets>,
    ui_assets: Res<UiAssets>,
) {
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
            &mod_ui_assets,
            &extended_ui_assets,
            &ui_assets,
            ButtonAction::EnterGameState(GameState::Playing),
            300.0,
            true,
            false,
        );

        parent.spawn_menu_button(
            &mod_ui_assets,
            &extended_ui_assets,
            &ui_assets,
            ButtonAction::EnterPauseMenuState(PauseMenuState::Options),
            300.0,
            false,
            false,
        );

        parent.spawn_menu_button(
            &mod_ui_assets,
            &extended_ui_assets,
            &ui_assets,
            ButtonAction::EnterAppState(AppState::MainMenuLoading),
            300.0,
            false,
            false,
        );
    });
}

/// Spawns ui for options pause menu
pub(in crate::ui) fn spawn_pause_options_system(
    mut cmds: Commands,
    mod_ui_assets: Res<ModUiAssets>,
    extended_ui_assets: Res<ExtendedUiAssets>,
    ui_assets: Res<UiAssets>,
) {
    cmds.spawn((
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
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
    ))
    .with_children(|parent| {
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_menu_button(
                    &mod_ui_assets,
                    &extended_ui_assets,
                    &ui_assets,
                    ButtonAction::ApplyOptions,
                    300.0,
                    true,
                    false,
                );

                parent.spawn_menu_button(
                    &mod_ui_assets,
                    &extended_ui_assets,
                    &ui_assets,
                    ButtonAction::EnterPauseMenuState(PauseMenuState::Main),
                    300.0,
                    false,
                    false,
                );
            });
    });
}
