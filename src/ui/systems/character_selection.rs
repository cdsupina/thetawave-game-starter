use super::{
    ButtonAction, CarouselSlotPosition, CharacterCarousel, Cleanup, InputType, MainMenuState,
    OptionsRes, PlayerJoinEvent, PlayerNum, PlayerReadyEvent, UiAssets, VisibleCarouselSlot,
};
use crate::{
    player::{ChosenCharacterData, ChosenCharactersResource},
    ui::data::{CharacterSelector, MenuButtonState, PlayerReadyButton, StartGameButton},
};
use bevy::{
    color::{Alpha, Color},
    core::Name,
    input::ButtonInput,
    log::warn,
    prelude::{
        BuildChildren, Changed, ChildBuild, Children, Commands, DespawnRecursiveExt, Entity,
        EventReader, EventWriter, Gamepad, GamepadButton, ImageNode, KeyCode, Parent, Query, Res,
        ResMut, Text, With, Without,
    },
    text::TextFont,
    ui::{AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::prelude::Focusable;
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
use bevy_hui::prelude::HtmlNode;
use bevy_persistent::Persistent;
use leafwing_input_manager::InputManagerBundle;

/// This function sets up the character selection interface.
/// It spawns the options menu HTML node and associates the cleanup component with it.
pub(in crate::ui) fn setup_character_selection_system(
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    // Create an HTMLNode with options menu HTML and link the OptionsMenuCleanup component.
    cmds.spawn((
        HtmlNode(ui_assets.character_selection_html.clone()),
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::CharacterSelection],
        },
        Name::new("Character Selection Menu"),
    ));
}

/// Cycle the characters in the carousel with player input
pub(in crate::ui) fn cycle_player_one_carousel_system(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads_q: Query<&Gamepad>,
    mut carousel_q: Query<(&mut CharacterCarousel, &PlayerNum)>,
    ready_button_q: Query<&ButtonAction, With<PlayerReadyButton>>,
    chosen_characters_res: Res<ChosenCharactersResource>,
) {
    if let Some(character_data) = chosen_characters_res.players.get(&PlayerNum::One) {
        for (mut carousel, player_num) in carousel_q.iter_mut() {
            if matches!(player_num, PlayerNum::One) {
                // Determine if the carousel can cycle by checking the state of the ready button
                let mut can_cycle = true;

                for button_action in ready_button_q.iter() {
                    if let ButtonAction::UnReady(button_player_num) = button_action {
                        if player_num == button_player_num {
                            can_cycle = false;
                        }
                    }
                }

                // Cycle the carousel with provided input for player one
                if can_cycle {
                    match character_data.input {
                        InputType::Keyboard => {
                            if keys.just_pressed(KeyCode::ArrowLeft)
                                || keys.just_pressed(KeyCode::KeyA)
                            {
                                carousel.cycle_left();
                            } else if keys.just_pressed(KeyCode::ArrowRight)
                                || keys.just_pressed(KeyCode::KeyD)
                            {
                                carousel.cycle_right();
                            }
                        }
                        InputType::Gamepad(entity) => {
                            if let Ok(gamepad) = gamepads_q.get(entity) {
                                if gamepad.just_pressed(GamepadButton::DPadLeft) {
                                    carousel.cycle_left();
                                } else if gamepad.just_pressed(GamepadButton::DPadRight) {
                                    carousel.cycle_right();
                                }
                            }
                        }
                    }
                }

                break;
            }
        }
    }
}

/// Change shown carousel character images when the character carousel changes from cycle_carousel_system
pub(in crate::ui) fn update_carousel_ui_system(
    carousel_q: Query<(&Children, &CharacterCarousel), Changed<CharacterCarousel>>,
    mut carousel_slot_q: Query<(&VisibleCarouselSlot, &mut ImageNode)>,
    ui_assets: Res<UiAssets>,
) {
    for (children, carousel) in carousel_q.iter() {
        // Iterate through all of the visible character image nodes
        for child in children.iter() {
            if let Ok((slot, mut image_node)) = carousel_slot_q.get_mut(*child) {
                let maybe_character_type = match slot.0 {
                    CarouselSlotPosition::Center => carousel.get_active_character(),
                    CarouselSlotPosition::Right => carousel.get_right_character(),
                    CarouselSlotPosition::Left => carousel.get_left_character(),
                };

                // Set the image of the ui node to the new character
                if let Some(character_type) = maybe_character_type {
                    image_node.image = ui_assets.get_character_image(character_type);
                }
            }
        }
    }
}

/// Update the chosen characters resource with the characters from the carousels
pub(in crate::ui) fn set_characters_system(
    character_carousel_q: Query<(&CharacterCarousel, &PlayerNum), Changed<CharacterCarousel>>,
    mut chosen_characters_res: ResMut<ChosenCharactersResource>,
) {
    for (carousel, player_num) in character_carousel_q.iter() {
        if let Some(character_type) = carousel.get_active_character() {
            chosen_characters_res.players.insert(
                player_num.clone(),
                ChosenCharacterData {
                    character: character_type.clone(),
                    input: carousel.input_type.clone(),
                },
            );
        }
    }
}

/// Spawn character carousel when PlayerJoinEvent is read
pub(in crate::ui) fn spawn_carousel_system(
    mut player_join_events: EventReader<PlayerJoinEvent>,
    character_selector_q: Query<(Entity, &PlayerNum), With<CharacterSelector>>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
    options_res: Res<Persistent<OptionsRes>>,
) {
    for event in player_join_events.read() {
        for (entity, player_num) in character_selector_q.iter() {
            if *player_num == event.player_num {
                cmds.entity(entity).despawn_descendants();

                let carousel = CharacterCarousel::new(event.input.clone());

                cmds.entity(entity).with_children(|parent| {
                    // Spawn left arrow
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::End,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            Name::new(format!("Left Arrow Button {}", player_num.as_ref())),
                        ))
                        .with_child((
                            Node {
                                width: Val::Px(108.0),
                                height: Val::Px(48.0),
                                ..default()
                            },
                            AseUiAnimation {
                                animation: Animation::tag("idle"),
                                aseprite: ui_assets.arrow_button_aseprite.clone(),
                            },
                            Name::new("Arrow Button Sprite"),
                        ));

                    let mut carousel_builder = parent.spawn((
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
                        player_num.clone(),
                        carousel.clone(),
                    ));

                    if !matches!(player_num, PlayerNum::One) {
                        carousel_builder.insert(InputManagerBundle::with_map(match event.input {
                            InputType::Keyboard => options_res.carousel_keyboard_input_map.clone(),
                            InputType::Gamepad(entity) => options_res
                                .carousel_gamepad_input_map
                                .clone()
                                .with_gamepad(entity),
                        }));
                    }

                    carousel_builder.with_children(|parent| {
                        // spawn child nodes containing carousel character images
                        if let Some(left_character_type) = carousel.get_left_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Left),
                                ImageNode::new(ui_assets.get_character_image(left_character_type))
                                    .with_color(Color::default().with_alpha(0.5)),
                                Node {
                                    width: Val::Percent(30.0),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            warn!("No left character found in carousel.");
                        }

                        if let Some(active_character_type) = carousel.get_active_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Center),
                                ImageNode::new(
                                    ui_assets.get_character_image(active_character_type),
                                ),
                                Node {
                                    width: Val::Percent(40.0),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            warn!("No active character found in carousel.");
                        }

                        if let Some(right_character_type) = carousel.get_right_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Right),
                                ImageNode::new(ui_assets.get_character_image(right_character_type))
                                    .with_color(Color::default().with_alpha(0.5)),
                                Node {
                                    width: Val::Percent(30.0),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            warn!("No right character found in carousel.");
                        }
                    });

                    // Spawn right arrow
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::End,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            Name::new(format!("Right Arrow Button {}", player_num.as_ref())),
                        ))
                        .with_child((
                            Node {
                                width: Val::Px(108.0),
                                height: Val::Px(48.0),
                                ..default()
                            },
                            AseUiAnimation {
                                animation: Animation::tag("idle"),
                                aseprite: ui_assets.arrow_button_aseprite.clone(),
                            },
                            ImageNode::default().with_flip_x(),
                            Name::new("Arrow Button Sprite"),
                        ));
                });
            }
        }
    }
}

/// Replaces the join button with a ready button when player joins
pub(in crate::ui) fn spawn_ready_button_system(
    mut player_join_events: EventReader<PlayerJoinEvent>,
    button_q: Query<(&ButtonAction, Entity, &Parent)>,
    ui_assets: Res<UiAssets>,
    mut cmds: Commands,
) {
    for event in player_join_events.read() {
        for (action, entity, parent) in button_q.iter() {
            if let ButtonAction::Join(player_num) = action {
                if event.player_num == *player_num {
                    cmds.entity(entity).despawn_recursive();
                    cmds.entity(parent.get()).with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    margin: UiRect::all(Val::Vh(1.0)),
                                    ..default()
                                },
                                ButtonAction::Ready(player_num.clone()),
                                MenuButtonState::Normal,
                                PlayerReadyButton,
                                Focusable::new().prioritized(), // Focus on this button
                                Name::new("Menu Button Ready"),
                            ))
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        Node {
                                            width: Val::Px(364.5),
                                            height: Val::Px(87.75),
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        AseUiAnimation {
                                            animation: Animation::tag("pressed"),
                                            aseprite: ui_assets.menu_button_aseprite.clone(),
                                        },
                                        Name::new("Menu Button Sprite"),
                                    ))
                                    .with_children(|parent| {
                                        parent
                                            .spawn(Node {
                                                margin: UiRect::new(
                                                    Val::Px(1.0),
                                                    Val::Px(1.0),
                                                    Val::Px(1.0),
                                                    Val::Px(14.0),
                                                ),
                                                flex_direction: FlexDirection::Column,
                                                justify_content: JustifyContent::FlexEnd,
                                                ..default()
                                            })
                                            .with_child((
                                                Text::new("Ready"),
                                                TextFont::from_font_size(30.0),
                                            ));
                                    });
                            });
                    });
                }
            }
        }
    }
}

/// Change normal ready button to locked in green ready button
pub(in crate::ui) fn lock_in_player_button_system(
    mut button_q: Query<(&mut MenuButtonState, &mut ButtonAction, &Children)>,
    mut button_sprite_q: Query<&mut AseUiAnimation>,
    mut player_ready_events: EventReader<PlayerReadyEvent>,
) {
    for event in player_ready_events.read() {
        for (mut button_state, mut action, children) in button_q.iter_mut() {
            match action.clone() {
                ButtonAction::Ready(player_num) | ButtonAction::UnReady(player_num) => {
                    if event.player_num == player_num {
                        // Set the action and state based on the whether is_ready is set
                        if event.is_ready {
                            *button_state = MenuButtonState::Ready;
                            *action = ButtonAction::UnReady(player_num);
                        } else {
                            *button_state = MenuButtonState::Normal;
                            *action = ButtonAction::Ready(player_num);
                        }

                        // Set the animation tag based on whether is_ready is set
                        for child in children.iter() {
                            if let Ok(mut ase_animation) = button_sprite_q.get_mut(*child) {
                                if event.is_ready {
                                    ase_animation.animation = Animation::tag("ready_pressed");
                                } else {
                                    ase_animation.animation = Animation::tag("pressed");
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Enables the start game button when all players are ready, disables the button if one or more players is not ready
pub(in crate::ui) fn enable_start_game_button_system(
    ready_button_q: Query<&MenuButtonState, With<PlayerReadyButton>>,
    mut disabled_button_q: Query<
        (Entity, &mut MenuButtonState, &Children),
        (With<StartGameButton>, Without<PlayerReadyButton>),
    >,
    mut button_sprite_q: Query<&mut AseUiAnimation>,
    mut cmds: Commands,
) {
    // Bool for tracking if all players are ready
    let mut players_ready = true;

    if !ready_button_q.is_empty() {
        // Query all ready buttons and update ready variable
        for menu_button_state in ready_button_q.iter() {
            if !matches!(menu_button_state, MenuButtonState::Ready) {
                players_ready = false;
                break;
            }
        }

        // Change the state and animation of the start game button depending on player readiness
        if let Ok((entity, mut start_game_button_state, children)) =
            disabled_button_q.get_single_mut()
        {
            if players_ready {
                if matches!(*start_game_button_state, MenuButtonState::Disabled) {
                    *start_game_button_state = MenuButtonState::Normal;
                    cmds.entity(entity).insert(Focusable::default());
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = button_sprite_q.get_mut(*child) {
                            ase_animation.animation = Animation::tag("released");
                        }
                    }
                }
            } else if matches!(*start_game_button_state, MenuButtonState::Normal) {
                *start_game_button_state = MenuButtonState::Disabled;
                cmds.entity(entity).remove::<Focusable>();
                for child in children.iter() {
                    if let Ok(mut ase_animation) = button_sprite_q.get_mut(*child) {
                        ase_animation.animation = Animation::tag("disabled");
                    }
                }
            }
        }
    }
}

/// Enables the join button for the next player after a player joins
pub(in crate::ui) fn enable_join_button_system(
    mut player_join_events: EventReader<PlayerJoinEvent>,
    mut join_button_q: Query<(&ButtonAction, &mut MenuButtonState, &Children)>,
    mut button_sprite_q: Query<&mut AseUiAnimation>,
) {
    for event in player_join_events.read() {
        if let Some(next_player_num) = event.player_num.next() {
            for (button_action, mut menu_button_state, children) in join_button_q.iter_mut() {
                if let ButtonAction::Join(player_num) = button_action {
                    if next_player_num == *player_num {
                        *menu_button_state = MenuButtonState::Normal;
                        for child in children.iter() {
                            if let Ok(mut ase_animation) = button_sprite_q.get_mut(*child) {
                                ase_animation.animation = Animation::tag("released");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Spawns the input prompt for the next player after a player joins
pub(in crate::ui) fn spawn_join_prompt_system(
    mut player_join_events: EventReader<PlayerJoinEvent>,
    character_selector_q: Query<(Entity, &PlayerNum), With<CharacterSelector>>,
    ui_assets: Res<UiAssets>,
    mut cmds: Commands,
) {
    for event in player_join_events.read() {
        if let Some(next_player_num) = event.player_num.next() {
            for (entity, player_num) in character_selector_q.iter() {
                if next_player_num == *player_num {
                    cmds.entity(entity)
                        .insert(Node {
                            flex_direction: FlexDirection::Row,
                            margin: UiRect::new(
                                Val::Px(1.0),
                                Val::Px(1.0),
                                Val::Px(20.0),
                                Val::Px(1.0),
                            ),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                AseUiAnimation {
                                    animation: Animation::tag("key_return"),
                                    aseprite: ui_assets.return_button_aseprite.clone(),
                                },
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                Name::new("Join Prompt Input"),
                            ));
                            parent.spawn((
                                AseUiAnimation {
                                    animation: Animation::tag("a"),
                                    aseprite: ui_assets.xbox_letter_buttons_aseprite.clone(),
                                },
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                Name::new("Join Prompt Input"),
                            ));
                        });
                }
            }
        }
    }
}

/// Sending join events after detecting join inputs from additonal players
pub(in crate::ui) fn additional_players_join_system(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads_q: Query<(Entity, &Gamepad)>,
    chosen_characters_res: Res<ChosenCharactersResource>,
    mut player_join_events: EventWriter<PlayerJoinEvent>,
) {
    // Set the join input to keyboard if input pressed and input is not yet used
    let mut join_input = if keys.just_pressed(KeyCode::Enter)
        && !chosen_characters_res.contains_input(InputType::Keyboard)
    {
        Some(InputType::Keyboard)
    } else {
        None
    };

    // If keyboard input not used, find the first gamepad that joined
    if join_input.is_none() {
        for (entity, gamepad) in gamepads_q.iter() {
            if gamepad.just_pressed(GamepadButton::South)
                && !chosen_characters_res.contains_input(InputType::Gamepad(entity))
            {
                join_input = Some(InputType::Gamepad(entity));
                break;
            }
        }
    }

    if let Some(input) = join_input {
        if let Some(player_num) = chosen_characters_res.next_available_player_num() {
            player_join_events.send(PlayerJoinEvent { player_num, input });
        }
    }
}
