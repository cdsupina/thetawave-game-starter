use super::{
    CarouselSlotPosition, CharacterCarousel, Cleanup, MainMenuState, PlayerJoinEvent, PlayerNum,
    UiAssets, VisibleCarouselSlot,
};
use crate::{player::ChosenCharactersEvent, ui::data::CharacterSelector};
use bevy::{
    color::{Alpha, Color},
    core::Name,
    input::ButtonInput,
    log::warn,
    prelude::{
        BuildChildren, Changed, ChildBuild, Children, Commands, DespawnRecursiveExt, Entity,
        EventReader, EventWriter, ImageNode, KeyCode, Query, Res, With,
    },
    ui::{AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
use bevy_hui::prelude::HtmlNode;

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
pub(in crate::ui) fn cycle_carousel_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut carousel_q: Query<&mut CharacterCarousel>,
) {
    if let Ok(mut carousel) = carousel_q.get_single_mut() {
        if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
            carousel.cycle_left();
        } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
            carousel.cycle_right();
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

/// Send an event containing the chosen characters for each player
/// Should be sent once when exiting the MainMenu state
pub(in crate::ui) fn set_characters_system(
    mut chosen_character_events: EventWriter<ChosenCharactersEvent>,
    character_carousel_q: Query<(&CharacterCarousel, &PlayerNum)>,
) {
    let mut players = vec![];

    for (carousel, player_num) in character_carousel_q.iter() {
        if let Some(character_type) = carousel.get_active_character() {
            players.push((player_num.clone(), character_type.clone()));
        }
    }

    chosen_character_events.send(ChosenCharactersEvent { players });
}

/// Spawn character carousel when PlayerJoinEvent is read
pub(in crate::ui) fn spawn_carousel_system(
    mut player_join_events: EventReader<PlayerJoinEvent>,
    character_selector_q: Query<(Entity, &PlayerNum), With<CharacterSelector>>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    for event in player_join_events.read() {
        for (entity, player_num) in character_selector_q.iter() {
            if *player_num == event.0 {
                cmds.entity(entity).despawn_descendants();

                let carousel = CharacterCarousel::new();

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

                    parent
                        .spawn((
                            Node {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
                            player_num.clone(),
                            carousel.clone(),
                        ))
                        .with_children(|parent| {
                            // spawn child nodes containing carousel character images
                            if let Some(left_character_type) = carousel.get_left_character() {
                                parent.spawn((
                                    VisibleCarouselSlot(CarouselSlotPosition::Left),
                                    ImageNode::new(
                                        ui_assets.get_character_image(left_character_type),
                                    )
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
                                    ImageNode::new(
                                        ui_assets.get_character_image(right_character_type),
                                    )
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
