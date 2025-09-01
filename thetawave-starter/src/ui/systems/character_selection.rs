use super::{
    AudioEffectEvent, ButtonAction, Cleanup, MainMenuState, OptionsRes, PlayerJoinEvent, PlayerNum,
    PlayerReadyEvent, UiAssets,
};
use crate::ui::data::{
    CarouselReadyTimer, CarouselSlotPosition, CharacterCarousel, CharacterSelector,
    MenuButtonState, PlayerReadyButton, StartGameButton, UiChildBuilderExt, VisibleCarouselSlot,
};
use bevy::{
    asset::Handle,
    color::{Alpha, Color},
    ecs::hierarchy::ChildOf,
    image::Image,
    input::ButtonInput,
    log::warn,
    prelude::{
        Changed, Children, Commands, Entity, EventReader, EventWriter, Gamepad, GamepadButton,
        ImageNode, KeyCode, Name, Query, Res, ResMut, With, Without,
    },
    time::Time,
    ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::prelude::Focusable;
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation};
use bevy_persistent::Persistent;
use leafwing_input_manager::prelude::{ActionState, InputMap};
use thetawave_assets::AssetResolver;
use thetawave_player::{
    CharacterCarouselAction, CharacterType, ChosenCharacterData, ChosenCharactersResource,
    InputType,
};
use thetawave_states::AppState;

/// Spawn ui for character selection
pub(in crate::ui) fn spawn_character_selection_system(
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    cmds.spawn((
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::CharacterSelection],
        },
        Name::new("Character Selection Menu"),
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
            .spawn((Node {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|parent| {
                // First row of character selection
                parent
                    .spawn((Node {
                        height: Val::Percent(50.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Player 1 character selection
                        parent.spawn_character_selection(&ui_assets, PlayerNum::One);

                        // Player 2 character selection
                        parent.spawn_character_selection(&ui_assets, PlayerNum::Two);
                    });

                // Second row
                parent
                    .spawn((Node {
                        height: Val::Percent(50.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Player 3 character selection
                        parent.spawn_character_selection(&ui_assets, PlayerNum::Three);

                        // Player 4 character selection
                        parent.spawn_character_selection(&ui_assets, PlayerNum::Four);
                    });
            });

        // Menu buttons
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
                    ButtonAction::EnterAppState(AppState::GameLoading),
                    300.0,
                    false,
                    true,
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

/// Cycle the characters in the carousel with player input
pub(in crate::ui) fn cycle_player_one_carousel_system(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads_q: Query<&Gamepad>,
    mut carousel_q: Query<(&mut CharacterCarousel, &PlayerNum)>,
    ready_button_q: Query<&ButtonAction, With<PlayerReadyButton>>,
    chosen_characters_res: Res<ChosenCharactersResource>,
    mut effect_events: EventWriter<AudioEffectEvent>,
) {
    if let Some(character_data) = chosen_characters_res.players.get(&PlayerNum::One) {
        for (mut carousel, player_num) in carousel_q.iter_mut() {
            if matches!(player_num, PlayerNum::One) {
                // Determine if the carousel can cycle by checking the state of the ready button
                let mut can_cycle = true;

                for button_action in ready_button_q.iter() {
                    if let ButtonAction::UnReady(button_player_num) = button_action
                        && player_num == button_player_num
                    {
                        can_cycle = false;
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
                                effect_events.write(AudioEffectEvent::MenuButtonSelected);
                            } else if keys.just_pressed(KeyCode::ArrowRight)
                                || keys.just_pressed(KeyCode::KeyD)
                            {
                                carousel.cycle_right();
                                effect_events.write(AudioEffectEvent::MenuButtonSelected);
                            }
                        }
                        InputType::Gamepad(entity) => {
                            if let Ok(gamepad) = gamepads_q.get(entity) {
                                if gamepad.just_pressed(GamepadButton::DPadLeft) {
                                    carousel.cycle_left();
                                    effect_events.write(AudioEffectEvent::MenuButtonSelected);
                                } else if gamepad.just_pressed(GamepadButton::DPadRight) {
                                    carousel.cycle_right();
                                    effect_events.write(AudioEffectEvent::MenuButtonSelected);
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

fn get_character_image(character_type: &CharacterType, ui_assets: &UiAssets) -> Handle<Image> {
    let key = match character_type {
        CharacterType::Captain => "captain_character",
        CharacterType::Juggernaut => "juggernaut_character",
        CharacterType::Doomwing => "doomwing_character",
    };

    AssetResolver::get_ui_image(key, ui_assets)
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
                    image_node.image = get_character_image(character_type, &ui_assets);
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
                cmds.entity(entity).despawn_related::<Children>();

                let carousel = CharacterCarousel::new(event.input.clone());

                cmds.entity(entity).with_children(|parent| {
                    // Spawn left arrow
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Name::new(format!("Left Arrow Button {}", player_num.as_ref())),
                        ))
                        .with_child((
                            Node {
                                height: Val::Px(40.0),
                                ..default()
                            },
                            ImageNode::default(),
                            AseAnimation {
                                animation: Animation::tag("idle"),
                                aseprite: AssetResolver::get_ui_sprite("arrow_button", &ui_assets),
                            },
                            Name::new("Arrow Button Sprite"),
                        ));

                    let mut carousel_builder = parent.spawn((
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Percent(45.0),
                            ..default()
                        },
                        player_num.clone(),
                        carousel.clone(),
                    ));

                    // Add input manager for non-player one carousels
                    if !matches!(player_num, PlayerNum::One) {
                        carousel_builder.insert((
                            match event.input {
                                InputType::Keyboard => InputMap::new(
                                    options_res.carousel_keyboard_input_mappings.clone(),
                                ),
                                InputType::Gamepad(entity) => InputMap::new(
                                    options_res.carousel_gamepad_input_mappings.clone(),
                                )
                                .with_gamepad(entity),
                            },
                            CarouselReadyTimer::new(),
                        ));
                    }

                    carousel_builder.with_children(|parent| {
                        // spawn child nodes containing carousel character images
                        if let Some(left_character_type) = carousel.get_left_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Left),
                                ImageNode::new(get_character_image(
                                    left_character_type,
                                    &ui_assets,
                                ))
                                .with_color(Color::default().with_alpha(0.5)),
                                Node {
                                    width: Val::Percent(30.0),
                                    margin: UiRect::all(Val::Percent(3.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            warn!("No left character found in carousel.");
                        }

                        if let Some(active_character_type) = carousel.get_active_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Center),
                                ImageNode::new(get_character_image(
                                    active_character_type,
                                    &ui_assets,
                                )),
                                Node {
                                    width: Val::Percent(40.0),
                                    margin: UiRect::all(Val::Percent(3.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            warn!("No active character found in carousel.");
                        }

                        if let Some(right_character_type) = carousel.get_right_character() {
                            parent.spawn((
                                VisibleCarouselSlot(CarouselSlotPosition::Right),
                                ImageNode::new(get_character_image(
                                    right_character_type,
                                    &ui_assets,
                                ))
                                .with_color(Color::default().with_alpha(0.5)),
                                Node {
                                    width: Val::Percent(30.0),
                                    margin: UiRect::all(Val::Percent(3.0)),
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
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Name::new(format!("Right Arrow Button {}", player_num.as_ref())),
                        ))
                        .with_child((
                            Node {
                                height: Val::Px(40.0),
                                ..default()
                            },
                            AseAnimation {
                                animation: Animation::tag("idle"),
                                aseprite: AssetResolver::get_ui_sprite("arrow_button", &ui_assets),
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
    button_q: Query<(&ButtonAction, Entity, &ChildOf)>,
    ui_assets: Res<UiAssets>,
    mut cmds: Commands,
) {
    for event in player_join_events.read() {
        for (action, entity, childof) in button_q.iter() {
            if let ButtonAction::Join(player_num) = action
                && event.player_num == *player_num
            {
                cmds.entity(entity).despawn();
                cmds.entity(childof.parent()).with_children(|parent| {
                    let mut entity_cmds = parent.spawn_menu_button(
                        &ui_assets,
                        ButtonAction::Ready(player_num.clone()),
                        300.0,
                        true,
                        false,
                    );

                    entity_cmds.insert(PlayerReadyButton);

                    // Remove focusable component for non-player one
                    if !matches!(player_num, PlayerNum::One) {
                        entity_cmds.remove::<Focusable>();
                    }
                });
            }
        }
    }
}

/// Change normal ready button to locked in green ready button
pub(in crate::ui) fn lock_in_player_button_system(
    mut button_q: Query<(&mut MenuButtonState, &mut ButtonAction, &Children)>,
    mut button_sprite_q: Query<&mut AseAnimation>,
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
                                    ase_animation.animation = Animation::tag("ready_selected");
                                } else {
                                    ase_animation.animation = Animation::tag("selected");
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
    mut button_sprite_q: Query<&mut AseAnimation>,
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
        if let Ok((entity, mut start_game_button_state, children)) = disabled_button_q.single_mut()
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
    mut button_sprite_q: Query<&mut AseAnimation>,
) {
    for event in player_join_events.read() {
        if let Some(next_player_num) = event.player_num.next() {
            for (button_action, mut menu_button_state, children) in join_button_q.iter_mut() {
                if let ButtonAction::Join(player_num) = button_action
                    && next_player_num == *player_num
                {
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
                    cmds.entity(entity).with_children(|parent| {
                        parent.spawn_join_prompt(&ui_assets);
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
    mut effect_events: EventWriter<AudioEffectEvent>,
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

    if let (Some(input), Some(player_num)) = (
        join_input,
        chosen_characters_res.next_available_player_num(),
    ) {
        player_join_events.write(PlayerJoinEvent { player_num, input });
        effect_events.write(AudioEffectEvent::MenuButtonConfirm);
    }
}

/// Handles inputs for character carousels (players 2-4)
pub(in crate::ui) fn carousel_input_system(
    mut carousel_q: Query<(
        &mut CharacterCarousel,
        &ActionState<CharacterCarouselAction>,
        &PlayerNum,
        &mut CarouselReadyTimer,
    )>,
    mut player_ready_events: EventWriter<PlayerReadyEvent>,
    time: Res<Time>,
    mut effect_events: EventWriter<AudioEffectEvent>,
    ready_button_q: Query<&ButtonAction, With<PlayerReadyButton>>,
) {
    for (mut carousel, carousel_action, player_num, mut ready_timer) in carousel_q.iter_mut() {
        // Advance the ready timer
        ready_timer.0.tick(time.delta());

        // Determine if the carousel can cycle by checking the state of the ready button
        let mut can_cycle = true;

        for button_action in ready_button_q.iter() {
            if let ButtonAction::UnReady(button_player_num) = button_action
                && player_num == button_player_num
            {
                can_cycle = false;
            }
        }

        for action in carousel_action.get_just_pressed().iter() {
            match action {
                CharacterCarouselAction::CycleLeft => {
                    if can_cycle {
                        carousel.cycle_left();
                        effect_events.write(AudioEffectEvent::MenuButtonConfirm);
                    }
                }
                CharacterCarouselAction::CycleRight => {
                    if can_cycle {
                        carousel.cycle_right();
                        effect_events.write(AudioEffectEvent::MenuButtonConfirm);
                    }
                }
                CharacterCarouselAction::Ready => {
                    // Only let player ready after a the timer is complete
                    if ready_timer.0.finished() {
                        player_ready_events.write(PlayerReadyEvent {
                            player_num: player_num.clone(),
                            is_ready: true,
                        });
                        effect_events.write(AudioEffectEvent::MenuButtonConfirm);
                    }
                }
                CharacterCarouselAction::Unready => {
                    player_ready_events.write(PlayerReadyEvent {
                        player_num: player_num.clone(),
                        is_ready: false,
                    });
                    effect_events.write(AudioEffectEvent::MenuButtonConfirm);
                }
            }
        }
    }
}
