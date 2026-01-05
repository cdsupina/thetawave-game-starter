use crate::ui::{
    GameEndResultResource,
    data::{ButtonAction, UiChildBuilderExt},
};
use bevy::{
    color::Color,
    ecs::system::{Commands, Res, ResMut},
    prelude::Name,
    text::TextFont,
    ui::{
        AlignItems, BackgroundColor, Display, FlexDirection, JustifyContent, Node, UiRect, Val,
        widget::Text,
    },
    utils::default,
};
use thetawave_assets::{AssetResolver, ExtendedUiAssets, ModUiAssets, UiAssets};
use thetawave_core::{AppState, Cleanup, GameState};

/// Spawns the game over/victory ui
pub(in crate::ui) fn spawn_game_end_system(
    mut cmds: Commands,
    mod_ui_assets: Res<ModUiAssets>,
    extended_ui_assets: Res<ExtendedUiAssets>,
    ui_assets: Res<UiAssets>,
    game_end_result_res: Res<GameEndResultResource>,
) {
    // Pre-resolve assets - will panic on failure
    let dank_depths_font =
        AssetResolver::get_ui_font("Dank-Depths", &mod_ui_assets, &extended_ui_assets, &ui_assets)
            .expect("Failed to load Dank-Depths font");

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
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|parent| {
                // Top node containing end result/title text of the end screen
                // Game over, victory, etc
                parent
                    .spawn(Node {
                        height: Val::Percent(40.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(game_end_result_res.result.clone()),
                            TextFont::from_font_size(150.0).with_font(dank_depths_font.clone()),
                        ));
                    });

                // Center Node of the screen containing for containg information about the run
                // or high scores, etc
                parent.spawn(Node {
                    height: Val::Percent(60.0),
                    width: Val::Percent(100.0),
                    ..default()
                });
            });

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
                    ButtonAction::EnterAppState(AppState::MainMenuLoading),
                    300.0,
                    false,
                    false,
                );
            });
    });
}

/// Reset the game end result resource
/// Should be called once before restarting the run
pub(in crate::ui) fn reset_game_end_result_resource_system(
    mut game_end_result_res: ResMut<GameEndResultResource>,
) {
    *game_end_result_res = GameEndResultResource::default();
}
