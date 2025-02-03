use crate::ui::data::{ButtonAction, UiChildBuilderExt};

use super::{Cleanup, MainMenuState, UiAssets};
use bevy::{
    core::Name,
    prelude::{BuildChildren, ChildBuild, Commands, EventReader, Query, Res, With},
    ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::{events::NavEvent, prelude::Focusable};
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};

/// This system sets up the title menu interface.
/// It spawns the main menu HTML node and associates the cleanup component with it.
pub(in crate::ui) fn spawn_title_menu_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    // Create an HTMLNode with main menu HTML and link the TitleMenuCleanup component.
    cmds.spawn((
        //HtmlNode(ui_assets.title_menu_html.clone()),
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::Title],
        },
        Name::new("Title Menu"),
        // Top level parent node
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ))
    .with_children(|parent| {
        // Container node for title logo
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Title Logo"),
                    Node {
                        height: Val::Vh(50.0),
                        ..default()
                    },
                    AseUiAnimation {
                        animation: Animation::tag("title").with_speed(1.25),
                        aseprite: ui_assets.thetawave_logo_aseprite.clone(),
                    },
                ));
            });

        // Container node for menu buttons
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                height: Val::Vh(40.0),
                ..default()
            })
            .with_children(|parent| {
                // Play Button
                parent.spawn_menu_button(
                    &ui_assets,
                    ButtonAction::EnterMainMenuState(MainMenuState::CharacterSelection),
                    300.0,
                    true,
                    false,
                );
                // Options Button
                parent.spawn_menu_button(
                    &ui_assets,
                    ButtonAction::EnterMainMenuState(MainMenuState::Options),
                    300.0,
                    false,
                    false,
                );
                // Exit Button
                parent.spawn_menu_button(&ui_assets, ButtonAction::Exit, 300.0, false, false);
            });

        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Vh(10.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::End,
                ..default()
            })
            .with_children(|parent| {
                // bluesky link
                parent.spawn((
                    Node {
                        height: Val::Vh(4.0),
                        width: Val::Vh(4.0),
                        margin: UiRect::all(Val::Vh(1.0)),
                        ..default()
                    },
                    AseUiAnimation {
                        animation: Animation::tag("released"),
                        aseprite: ui_assets.bluesky_logo_aseprite.clone(),
                    },
                    ButtonAction::OpenBlueskyWebsite,
                    Name::new("Bluesky Website Button"),
                    Focusable::default(),
                ));

                // github link
                parent.spawn((
                    Node {
                        height: Val::Vh(4.0),
                        width: Val::Vh(4.0),
                        margin: UiRect::all(Val::Vh(1.0)),
                        ..default()
                    },
                    AseUiAnimation {
                        animation: Animation::tag("released"),
                        aseprite: ui_assets.github_logo_aseprite.clone(),
                    },
                    ButtonAction::OpenGithubWebsite,
                    Name::new("Github Website Button"),
                    Focusable::default(),
                ));
            });
    });
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
