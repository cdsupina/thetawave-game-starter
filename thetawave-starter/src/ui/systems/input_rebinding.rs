use crate::{
    input::{DummyGamepad, InputType, PlayerAbility, PlayerAction},
    options::OptionsRes,
    ui::data::{ButtonAction, UiChildBuilderExt},
};
use bevy::{
    ecs::{
        entity::Entity,
        prelude::Name,
        query::With,
        system::{Commands, Local, Query, Res, ResMut},
    },
    input::{
        gamepad::{Gamepad, GamepadButton},
        keyboard::KeyCode,
        mouse::MouseButton,
        ButtonInput,
    },
    ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_egui::{
    egui::{CentralPanel, Color32, ComboBox, Frame, Grid, Margin, RichText, Ui},
    EguiContexts,
};
use bevy_persistent::Persistent;
use itertools::{EitherOrBoth, Itertools};
use strum::IntoEnumIterator;
use thetawave_assets::UiAssets;
use thetawave_states::{Cleanup, MainMenuState};

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

fn create_player_action_rebind_button(
    ui: &mut Ui,
    active_input_method: &InputType,
    options_res: &OptionsRes,
    player_action: &PlayerAction,
    rebinding_flag: &Option<RebindingTarget>,
) -> Option<RebindingTarget> {
    if ui
        .button(match active_input_method {
            InputType::Keyboard => {
                let mut button_string = "".to_string();

                if let Some(key_code) = options_res
                    .player_keyboard_action_input_mappings
                    .get(player_action)
                {
                    button_string = key_code.to_string();
                } else if let Some(key_code) = options_res
                    .player_mouse_action_input_mappings
                    .get(player_action)
                {
                    button_string = key_code.to_string();
                }

                if let Some(RebindingTarget::PlayerAction(rebinding_target)) = rebinding_flag {
                    if *rebinding_target == *player_action {
                        button_string = "Press Input".to_string();
                    }
                }

                button_string
            }
            InputType::Gamepad(_) => {
                let mut button_string = "".to_string();

                if let Some(gamepad_button) = options_res
                    .player_gamepad_action_input_mappings
                    .get(player_action)
                {
                    button_string = gamepad_button.to_string();
                }

                if let Some(RebindingTarget::PlayerAction(rebinding_target)) = rebinding_flag {
                    if *rebinding_target == *player_action {
                        button_string = "Press Input".to_string();
                    }
                }

                button_string
            }
        })
        .clicked()
    {
        return Some(RebindingTarget::PlayerAction(player_action.clone()));
    }
    rebinding_flag.clone()
}

fn create_player_ability_rebind_button(
    ui: &mut Ui,
    active_input_method: &InputType,
    options_res: &OptionsRes,
    player_ability: &PlayerAbility,
    rebinding_flag: &Option<RebindingTarget>,
) -> Option<RebindingTarget> {
    if ui
        .button(match active_input_method {
            InputType::Keyboard => {
                let mut button_string = "".to_string();

                if let Some(key_code) = options_res
                    .player_keyboard_abilities_input_mappings
                    .get(player_ability)
                {
                    button_string = key_code.to_string();
                } else if let Some(key_code) = options_res
                    .player_mouse_abilities_input_mappings
                    .get(player_ability)
                {
                    button_string = key_code.to_string();
                }

                if let Some(RebindingTarget::PlayerAbility(rebinding_target)) = rebinding_flag {
                    if *rebinding_target == *player_ability {
                        button_string = "Press Input".to_string();
                    }
                }

                button_string
            }
            InputType::Gamepad(_) => {
                let mut button_string = "".to_string();

                if let Some(gamepad_button) = options_res
                    .player_gamepad_abilities_input_mappings
                    .get(player_ability)
                {
                    button_string = gamepad_button.to_string();
                }

                if let Some(RebindingTarget::PlayerAbility(rebinding_target)) = rebinding_flag {
                    if *rebinding_target == *player_ability {
                        button_string = "Press Input".to_string();
                    }
                }

                button_string
            }
        })
        .clicked()
    {
        return Some(RebindingTarget::PlayerAbility(player_ability.clone()));
    }
    rebinding_flag.clone()
}

#[derive(Debug, Clone)]
pub(in crate::ui) enum RebindingTarget {
    PlayerAction(PlayerAction),
    PlayerAbility(PlayerAbility),
}

/// This function is a system that handles the egui input rebinding menu
pub(in crate::ui) fn input_rebinding_menu_system(
    mut contexts: EguiContexts,
    mut options_res: ResMut<Persistent<OptionsRes>>,
    dummy_gamepad_q: Query<Entity, With<DummyGamepad>>,
    mut active_input_method: Local<InputType>,
    mut rebinding_flag: Local<Option<RebindingTarget>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    gamepads_q: Query<&Gamepad>,
) {
    if let Some(rebinding_target) = rebinding_flag.clone() {
        match active_input_method.clone() {
            InputType::Keyboard => {
                // First check if any keyboard keys released, the check for mouse buttons
                if let Some(key_code) = keys.get_just_released().next() {
                    match rebinding_target {
                        RebindingTarget::PlayerAction(player_action) => {
                            // Set action to new key
                            options_res
                                .player_keyboard_action_input_mappings
                                .insert(player_action.clone(), *key_code);
                            // Remove mouse binding for action
                            options_res
                                .player_mouse_action_input_mappings
                                .remove(&player_action);
                            *rebinding_flag = None;
                        }
                        RebindingTarget::PlayerAbility(player_ability) => {
                            // Set ability to new key
                            options_res
                                .player_keyboard_abilities_input_mappings
                                .insert(player_ability.clone(), *key_code);
                            // Remove mouse binding for ability
                            options_res
                                .player_mouse_abilities_input_mappings
                                .remove(&player_ability);
                            *rebinding_flag = None;
                        }
                    }
                } else if let Some(mouse_button) = mouse_buttons.get_just_released().next() {
                    match rebinding_target {
                        RebindingTarget::PlayerAction(player_action) => {
                            // Set action to new mouse button
                            options_res
                                .player_mouse_action_input_mappings
                                .insert(player_action.clone(), *mouse_button);
                            // Remove keyboard binding for action
                            options_res
                                .player_keyboard_action_input_mappings
                                .remove(&player_action);
                            *rebinding_flag = None;
                        }
                        RebindingTarget::PlayerAbility(player_ability) => {
                            // Set ability to new mouse button
                            options_res
                                .player_mouse_abilities_input_mappings
                                .insert(player_ability.clone(), *mouse_button);
                            // Remove keyboard binding for ability
                            options_res
                                .player_keyboard_abilities_input_mappings
                                .remove(&player_ability);
                            *rebinding_flag = None;
                        }
                    }
                }
            }
            InputType::Gamepad(_) => {
                for gamepad in gamepads_q.iter() {
                    if let Some(gamepad_button) = gamepad.get_just_released().next() {
                        match rebinding_target {
                            RebindingTarget::PlayerAction(player_action) => {
                                options_res
                                    .player_gamepad_action_input_mappings
                                    .insert(player_action, *gamepad_button);
                                *rebinding_flag = None;
                            }
                            RebindingTarget::PlayerAbility(player_ability) => {
                                options_res
                                    .player_gamepad_abilities_input_mappings
                                    .insert(player_ability, *gamepad_button);
                                *rebinding_flag = None;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    CentralPanel::default()
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            inner_margin: Margin::same(10),
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            Grid::new("input_grid").num_columns(4).show(ui, |ui| {
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
                        if let Ok(entity) = dummy_gamepad_q.single() {
                            ui.selectable_value(
                                &mut *active_input_method,
                                InputType::Gamepad(entity),
                                "Gamepad",
                            );
                        }
                    });
                ui.end_row();

                // Add labels and buttons for all player inputs and abilities
                for pair in PlayerAction::iter().zip_longest(PlayerAbility::iter()) {
                    match pair {
                        EitherOrBoth::Both(player_action, player_ability) => {
                            ui.label(RichText::new(player_action.as_ref()).size(LABEL_TEXT_SIZE));
                            *rebinding_flag = create_player_action_rebind_button(
                                ui,
                                &active_input_method,
                                &options_res,
                                &player_action,
                                &rebinding_flag,
                            );
                            ui.label(RichText::new(player_ability.as_ref()).size(LABEL_TEXT_SIZE));
                            *rebinding_flag = create_player_ability_rebind_button(
                                ui,
                                &active_input_method,
                                &options_res,
                                &player_ability,
                                &rebinding_flag,
                            );
                        }
                        EitherOrBoth::Left(player_action) => {
                            ui.label(RichText::new(player_action.as_ref()).size(LABEL_TEXT_SIZE));
                            *rebinding_flag = create_player_action_rebind_button(
                                ui,
                                &active_input_method,
                                &options_res,
                                &player_action,
                                &rebinding_flag,
                            );
                            ui.label("");
                            ui.label("");
                        }
                        EitherOrBoth::Right(player_ability) => {
                            ui.label("");
                            ui.label("");
                            ui.label(RichText::new(player_ability.as_ref()).size(LABEL_TEXT_SIZE));
                            *rebinding_flag = create_player_ability_rebind_button(
                                ui,
                                &active_input_method,
                                &options_res,
                                &player_ability,
                                &rebinding_flag,
                            );
                        }
                    }

                    ui.end_row();
                }
            });
        });
}
