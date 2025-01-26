use super::{Cleanup, MainMenuState, UiAssets};
use bevy::{
    core::Name,
    prelude::{Commands, EventReader, Query, Res, With},
};
use bevy_alt_ui_navigation_lite::{events::NavEvent, prelude::Focusable};
use bevy_aseprite_ultra::prelude::AseUiAnimation;
use bevy_hui::prelude::HtmlNode;

/// This system sets up the title menu interface.
/// It spawns the main menu HTML node and associates the cleanup component with it.
pub(in crate::ui) fn setup_title_menu_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    // Create an HTMLNode with main menu HTML and link the TitleMenuCleanup component.
    cmds.spawn((
        HtmlNode(ui_assets.title_menu_html.clone()),
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::Title],
        },
        Name::new("Title Menu"),
    ));
}

/// System that handles the focus state of website footer buttons
/// Updates the animation state when focus changes between buttons
/// Takes navigation events and queries for focusable animations
pub(in crate::ui) fn website_footer_button_focus_system(
    mut nav_events: EventReader<NavEvent>,
    mut focusable_q: Query<&mut AseUiAnimation, With<Focusable>>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            // Set newly focused button to selected animation
            if let Ok(mut ase_animation) = focusable_q.get_mut(*to.first()) {
                ase_animation.animation.play_loop("selected");
            }

            // Set previously focused button to released animation
            if let Ok(mut ase_animation) = focusable_q.get_mut(*from.first()) {
                ase_animation.animation.play_loop("released");
            }
        }
    }
}
