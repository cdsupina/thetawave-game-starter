use crate::ui::data::MenuButtonState;

use super::{
    ButtonAction, CarouselSlotPosition, CharacterCarousel, PlayerNum, UiAssets, VisibleCarouselSlot,
};
use bevy::{
    color::{Alpha, Color},
    core::Name,
    log::{info, warn},
    prelude::{BuildChildren, ChildBuild, Commands, Entity, ImageNode, In, Query, Res},
    ui::{Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::prelude::Focusable;
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
use bevy_egui::EguiContextSettings;
use bevy_hui::prelude::{HtmlComponents, HtmlFunctions, Tags};

/// This function sets up the main menu user interface. It spawns the main menu HTML node and registers the required functions and components.
pub(in crate::ui) fn setup_hui_system(
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
    ui_assets: Res<UiAssets>,
    mut egui_settings: Query<&mut EguiContextSettings>,
) {
    // Register bevy_hui components
    html_comps.register(
        "website_footer_button",
        ui_assets.website_footer_button_html.clone(),
    );
    html_comps.register("menu_button", ui_assets.menu_button_html.clone());
    html_comps.register("arrow_button", ui_assets.arrow_button_html.clone());
    html_comps.register(
        "menu_button_sprite",
        ui_assets.menu_button_sprite_html.clone(),
    );
    html_comps.register(
        "arrow_button_sprite",
        ui_assets.arrow_button_sprite_html.clone(),
    );
    html_comps.register("thetawave_logo", ui_assets.thetawave_logo_html.clone());
    html_comps.register(
        "character_carousel",
        ui_assets.character_carousel_html.clone(),
    );

    // Register bevy_hui functions
    html_funcs.register("setup_menu_button", setup_menu_button);
    html_funcs.register("setup_arrow_button", setup_arrow_button);
    html_funcs.register("setup_title_logo", setup_title_logo);
    html_funcs.register("setup_menu_button_sprite", setup_menu_button_sprite);
    html_funcs.register("setup_arrow_button_sprite", setup_arrow_button_sprite);
    html_funcs.register("setup_character_carousel", setup_character_carousel);
    html_funcs.register("setup_website_footer_button", setup_website_footer_button);

    // Increase scale of egui options menu
    if !cfg!(feature = "world_inspector") {
        egui_settings.single_mut().scale_factor = 2.0;
    }
}

/// Spawn child nodes for character carousel slots
fn setup_character_carousel(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    if let Ok(tags) = tags.get(entity) {
        if let Some(player_str) = tags.get("player") {
            match PlayerNum::try_from(player_str) {
                Ok(player_num) => {
                    let carousel = CharacterCarousel::new();

                    cmds.entity(entity)
                        .insert((player_num, carousel.clone()))
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
                }
                Err(msg) => {
                    warn!("{}", msg);
                }
            }
        }
    }
}

/// Sets up website footer buttons with appropriate animations and actions
/// Takes an entity, queries for tags, and sets up the button based on its action type
fn setup_website_footer_button(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    if let Ok(tags) = tags.get(entity) {
        if let Some(button_action_str) = tags.get("button_action") {
            match ButtonAction::try_from(button_action_str) {
                Ok(button_action) => match button_action {
                    // Handle Bluesky website button - add animation and action
                    ButtonAction::OpenBlueskyWebsite => {
                        cmds.entity(entity).insert((
                            AseUiAnimation {
                                animation: Animation::tag("released"),
                                aseprite: ui_assets.bluesky_logo_aseprite.clone(),
                            },
                            ButtonAction::OpenBlueskyWebsite,
                            Name::new("Bluesky Website Button"),
                        ));
                    }
                    // Handle Github website button - add animation and action
                    ButtonAction::OpenGithubWebsite => {
                        cmds.entity(entity).insert((
                            AseUiAnimation {
                                animation: Animation::tag("released"),
                                aseprite: ui_assets.github_logo_aseprite.clone(),
                            },
                            ButtonAction::OpenGithubWebsite,
                            Name::new("Github Website Button"),
                        ));
                    }
                    _ => {
                        warn!("Button action was not able to be mapped to a website action.")
                    }
                },
                Err(msg) => {
                    // If the action fails to convert, it is logged as a warning.
                    warn!("{}", msg);
                }
            };
        }
    } else {
        warn!("No tags found for website footer button.")
    }

    cmds.entity(entity).insert(Focusable::default());
}

/// Sets up menu button sprite animations based on whether it's the first button
/// Takes an entity, queries tags, and configures the animation state
fn setup_menu_button_sprite(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    let mut animation = Animation::tag("released");

    // Get tags for the entity
    if let Ok(tags) = tags.get(entity) {
        // Check if this is marked as the first button
        if let Some(first_str) = tags.get("first") {
            // Change the menu button animation to "pressed"
            if first_str == "true" {
                animation = Animation::tag("pressed");
            }
        } else {
            warn!("No tag \"first\" found for {entity}. Please insert a tag indicating whether the button should be the first button to focus.");
        }

        // Change animation if necessary given the button_state tag
        if let Some(state_str) = tags.get("button_state") {
            match state_str.as_ref() {
                "disabled" => {
                    animation = Animation::tag("disabled");
                }
                "ready" => {
                    animation = Animation::tag("ready");
                }
                "normal" => {}
                _ => {
                    warn!("Given button_state str for {entity} did not match any options.")
                }
            }
        } else {
            warn!("No tag \"button_state\" found for {entity}. Please insert a tag indicating the state of the button.");
        }
    }

    cmds.entity(entity).insert((
        AseUiAnimation {
            animation,
            aseprite: ui_assets.menu_button_aseprite.clone(),
        },
        Name::new("Menu Button Sprite"),
    ));
}

/// Sets up menu button sprite animations based on whether it's the first button
/// Takes an entity, queries tags, and configures the animation state
fn setup_arrow_button_sprite(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    ui_assets: Res<UiAssets>,
) {
    if let Ok(tags) = tags.get(entity) {
        if let Some(direction_str) = tags.get("direction") {
            // Insert animation component with pressed/released state based on first status
            cmds.entity(entity).insert((
                AseUiAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: ui_assets.arrow_button_aseprite.clone(),
                },
                // Flip the sprite if direction was specified as right
                if direction_str == "right" {
                    ImageNode::default().with_flip_x()
                } else {
                    ImageNode::default()
                },
                Name::new("Arrow Button Sprite"),
            ));
        }
    }
}

/// Sets up the title logo animation for the game's main menu
fn setup_title_logo(In(entity): In<Entity>, mut cmds: Commands, ui_assets: Res<UiAssets>) {
    cmds.entity(entity).insert((
        AseUiAnimation {
            animation: Animation::tag("title").with_speed(1.25),
            aseprite: ui_assets.thetawave_logo_aseprite.clone(),
        },
        Name::new("Title Logo"),
    ));
}

/// This function assigns actions to buttons based on their tags.
fn setup_menu_button(In(entity): In<Entity>, tags: Query<&Tags>, mut cmds: Commands) {
    if let Ok(tags) = tags.get(entity) {
        // Assign button action from tag
        if let Some(button_action_str) = tags.get("button_action") {
            match ButtonAction::try_from(button_action_str) {
                Ok(button_action) => {
                    // If the action is valid, it gets inserted into the entity.
                    cmds.entity(entity).insert(button_action);
                }
                Err(msg) => {
                    // If the action fails to convert, it is logged as a warning.
                    warn!("{}", msg);
                }
            };

            cmds.entity(entity)
                .insert(Name::new(format!("Menu Button {}", button_action_str)));
        }

        // Assign button action from tag
        if let Some(button_state_str) = tags.get("button_state") {
            match MenuButtonState::try_from(button_state_str) {
                Ok(button_state) => {
                    // If the state is valid, it gets inserted into the entity.
                    cmds.entity(entity).insert(button_state);
                }
                Err(msg) => {
                    // If the state fails to convert, it is logged as a warning.
                    warn!("{}", msg);
                }
            };
        } else {
            info!("No button state tag found for menu button. Inserting a MenuButtonState::Normal component into {entity}.");
            cmds.entity(entity).insert(MenuButtonState::default());
        }
    } else {
        warn!("No tags not found for menu button.");
    }

    cmds.entity(entity).insert(Focusable::default());
}

/// This function assigns actions to buttons based on their tags.
fn setup_arrow_button(In(entity): In<Entity>, tags: Query<&Tags>, mut cmds: Commands) {
    if let Ok(tags) = tags.get(entity) {
        if let Some(button_action_str) = tags.get("button_action") {
            match ButtonAction::try_from(button_action_str) {
                Ok(button_action) => {
                    // If the action is valid, it gets inserted into the entity.
                    cmds.entity(entity).insert(button_action);
                }
                Err(msg) => {
                    // If the action fails to convert, it is logged as a warning.
                    warn!("{}", msg);
                }
            };

            cmds.entity(entity)
                .insert(Name::new(format!("Arrow Button {}", button_action_str)));
        }
    } else {
        warn!("No tags not found for menu button.");
    }
}
