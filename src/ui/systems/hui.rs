use super::{AppState, ButtonAction, UiAssets};
use crate::ui::data::{MenuButtonState, StartGameButton};
use bevy::{
    core::Name,
    log::{info, warn},
    prelude::{Commands, Entity, In, Query, Res},
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
    html_comps.register("menu_button", ui_assets.menu_button_html.clone());
    html_comps.register(
        "menu_button_sprite",
        ui_assets.menu_button_sprite_html.clone(),
    );

    // Register bevy_hui functions
    html_funcs.register("setup_menu_button", setup_menu_button);
    html_funcs.register("setup_menu_button_sprite", setup_menu_button_sprite);

    // Increase scale of egui options menu
    if !cfg!(feature = "world_inspector") {
        egui_settings.single_mut().scale_factor = 2.0;
    }
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
            // Change the menu button animation to "selected"
            if first_str == "true" {
                animation = Animation::tag("selected");
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

/// This function assigns actions to buttons based on their tags.
fn setup_menu_button(In(entity): In<Entity>, tags: Query<&Tags>, mut cmds: Commands) {
    if let Ok(tags) = tags.get(entity) {
        // Assign button action from tag
        if let Some(button_action_str) = tags.get("button_action") {
            match ButtonAction::try_from(button_action_str) {
                Ok(button_action) => {
                    // If the action is valid, it gets inserted into the entity.
                    cmds.entity(entity).insert(button_action.clone());
                    if matches!(
                        button_action,
                        ButtonAction::EnterAppState(AppState::GameLoading)
                    ) {
                        cmds.entity(entity).insert(StartGameButton);
                    }
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
                    cmds.entity(entity).insert(button_state.clone());
                    if matches!(button_state, MenuButtonState::Normal) {
                        cmds.entity(entity).insert(Focusable::default());
                    }
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
}
