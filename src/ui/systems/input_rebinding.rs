use crate::{
    assets::UiAssets,
    input::{DummyGamepad, InputType, PlayerAbility, PlayerAction},
    options::OptionsRes,
    states::{Cleanup, MainMenuState},
    ui::data::{ButtonAction, UiChildBuilderExt},
};
use bevy::{
    core::Name,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Local, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, ChildBuild},
    input::{gamepad::GamepadButton, keyboard::KeyCode, mouse::MouseButton},
    ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_egui::{
    egui::{CentralPanel, Color32, ComboBox, Frame, Grid, Margin, RichText},
    EguiContexts,
};
use bevy_persistent::Persistent;
use itertools::{EitherOrBoth, Itertools};
use strum::IntoEnumIterator;

const LABEL_TEXT_SIZE: f32 = 12.0;

/// Spawns options menu ui for the main menu
pub(in crate::ui) fn spawn_input_rebinding_menu_system(
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    cmds.spawn((
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::InputRebinding],
        },
        Name::new("Input Rebinding Menu"),
        // Top level parent node
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        },
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
                    ButtonAction::ApplyOptions,
                    300.0,
                    true,
                    false,
                );
                parent.spawn_menu_button(
                    &ui_assets,
                    ButtonAction::EnterMainMenuState(MainMenuState::Title),
                    300.0,
                    false,
                    false,
                );
            });
    });
}

trait InputCodeToStringExt {
    fn to_string(&self) -> String;
}

impl InputCodeToStringExt for KeyCode {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl InputCodeToStringExt for MouseButton {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl InputCodeToStringExt for GamepadButton {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// This function is a system that handles the egui input rebinding menu
pub(in crate::ui) fn input_rebinding_menu_system(
    mut contexts: EguiContexts,
    mut options_res: ResMut<Persistent<OptionsRes>>,
    dummy_gamepad_q: Query<Entity, With<DummyGamepad>>,
    mut active_input_method: Local<InputType>,
) {
    CentralPanel::default()
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            inner_margin: Margin::same(10.0),
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            Grid::new("input_grid").num_columns(6).show(ui, |ui| {
                // Top row for selecting input method to be edited
                ui.label(RichText::new("Input Method").size(LABEL_TEXT_SIZE));
                ComboBox::from_id_salt("input_method_combobox")
                    .selected_text(match *active_input_method {
                        InputType::Keyboard => "Keyboard",
                        InputType::Gamepad(_) => "Gamepad",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut *active_input_method,
                            InputType::Keyboard,
                            "Keyboard",
                        );
                        if let Ok(entity) = dummy_gamepad_q.get_single() {
                            ui.selectable_value(
                                &mut *active_input_method,
                                InputType::Gamepad(entity),
                                "Gamepad",
                            );
                        }
                    });
                ui.end_row();

                /*
                let player_action_input_map = match *active_input_method {
                    InputType::Keyboard => options_res.player_keyboard_action_input_map.clone(),
                    InputType::Gamepad(entity) => {
                        options_res.player_gamepad_action_input_map.clone()
                    }
                };

                let player_abilities_input_map = match *active_input_method {
                    InputType::Keyboard => options_res.player_keyboard_abilities_input_map.clone(),
                    InputType::Gamepad(entity) => {
                        options_res.player_gamepad_abilities_input_map.clone()
                    }
                };
                */

                // Add labels and buttons for all player inputs and abilities
                for pair in PlayerAction::iter().zip_longest(PlayerAbility::iter()) {
                    match pair {
                        EitherOrBoth::Both(player_action, player_ability) => {
                            ui.label(RichText::new(player_action.as_ref()).size(LABEL_TEXT_SIZE));
                            let _ = ui.button(match *active_input_method {
                                InputType::Keyboard => {
                                    if let Some(key_code) = options_res
                                        .player_keyboard_action_input_mappings
                                        .get(&player_action)
                                    {
                                        key_code.to_string()
                                    } else if let Some(key_code) = options_res
                                        .player_mouse_action_input_mappings
                                        .get(&player_action)
                                    {
                                        key_code.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                }
                                InputType::Gamepad(_) => {
                                    if let Some(gamepad_button) = options_res
                                        .player_gamepad_action_input_mappings
                                        .get(&player_action)
                                    {
                                        gamepad_button.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                }
                            });
                            ui.label(RichText::new(player_ability.as_ref()).size(LABEL_TEXT_SIZE));
                            ui.button("Key");
                        }
                        EitherOrBoth::Left(player_action) => {
                            ui.label(RichText::new(player_action.as_ref()).size(LABEL_TEXT_SIZE));
                            ui.button("Key");
                            ui.label("");
                        }
                        EitherOrBoth::Right(player_ability) => {
                            ui.label("");
                            ui.label(RichText::new(player_ability.as_ref()).size(LABEL_TEXT_SIZE));
                            ui.button("Key");
                        }
                    }

                    ui.end_row();
                }
            });
        });
}
