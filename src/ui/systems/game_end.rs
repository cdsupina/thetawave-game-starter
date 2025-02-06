use bevy::{
    color::Color,
    core::Name,
    ecs::system::{Commands, Res},
    hierarchy::{BuildChildren, ChildBuild},
    ui::{AlignItems, BackgroundColor, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};

use crate::{
    assets::UiAssets,
    states::{AppState, Cleanup, GameState},
    ui::data::{ButtonAction, UiChildBuilderExt},
};

/// Spawns the game over/victory ui
pub(in crate::ui) fn spawn_game_end_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    cmds.spawn((
        Cleanup::<GameState> {
            states: vec![GameState::End],
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Name::new("End Game Menu"),
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
                    &ui_assets,
                    ButtonAction::EnterAppState(AppState::MainMenuLoading),
                    300.0,
                    false,
                    false,
                );
            });
    });
}
