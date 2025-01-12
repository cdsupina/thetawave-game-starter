use super::{
    CarouselSlotPosition, CharacterCarousel, Cleanup, MainMenuState, PlayerNum, UiAssets,
    VisibleCarouselSlot,
};
use crate::player::ChosenCharactersEvent;
use bevy::{
    core::Name,
    input::ButtonInput,
    prelude::{Changed, Children, Commands, EventWriter, ImageNode, KeyCode, Query, Res},
};
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
