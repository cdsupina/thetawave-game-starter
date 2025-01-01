use crate::{
    assets::MainMenuAssets,
    options::{ApplyOptionsEvent, OptionsRes},
    states::{
        AppState, CharacterSelectionCleanup, MainMenuState, OptionsMenuCleanup, TitleMenuCleanup,
    },
};
use bevy::{
    app::AppExit,
    prelude::{
        Children, Commands, Entity, EntityCommands, EventReader, EventWriter, In, NextState, Query,
        Res, ResMut, With,
    },
    window::{MonitorSelection, WindowMode, WindowResolution},
};
use bevy_alt_ui_navigation_lite::{events::NavEvent, prelude::Focusable};
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
use bevy_egui::{egui, EguiContexts, EguiSettings};
use bevy_hui::prelude::{HtmlComponents, HtmlFunctions, HtmlNode, Tags};
use log::{info, warn};
use webbrowser;

use super::data::ButtonAction;

const GITHUB_URL: &str = "https://github.com/thetawavegame/thetawave";
const BLUESKY_URL: &str = "https://bsky.app/profile/carlo.metalmancy.tech";

/// This function sets up the main menu user interface. It spawns the main menu HTML node and registers the required functions and components.
pub(super) fn setup_ui_system(
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
    main_menu_assets: Res<MainMenuAssets>,
    mut egui_settings: Query<&mut EguiSettings>,
) {
    // Register the "assign_action" function that links UI components and their actions.
    html_funcs.register("assign_action_to_menu_button", assign_action_to_menu_button);

    // Register the footer button component which is used for website links.
    // It uses a spawn function to also establish the focus behaviour on it.
    html_comps.register_with_spawn_fn(
        "website_footer_button",
        main_menu_assets.website_footer_button_html.clone(),
        attach_focusable,
    );

    // Register the main menu button component.
    // It uses a spawn function to also establish the focus behaviour on it.
    html_comps.register_with_spawn_fn(
        "menu_button",
        main_menu_assets.menu_button_html.clone(),
        attach_focusable,
    );

    html_comps.register(
        "menu_button_sprite",
        main_menu_assets.menu_button_sprite_html.clone(),
    );

    // Registers the thetawave logo component
    html_comps.register(
        "thetawave_logo",
        main_menu_assets.thetawave_logo_html.clone(),
    );

    // Registers setup function for the title logo
    html_funcs.register("setup_title_logo", setup_title_logo);

    // Register the setup function for menu button sprites which handles animations
    html_funcs.register("setup_menu_button_sprite", setup_menu_button_sprite);

    // Register the setup function for website footer buttons which handles website linking
    html_funcs.register("setup_website_footer_button", setup_website_footer_button);

    // Set the egui scale factor to 2.0, this ensures visible and readable UI.
    if !cfg!(feature = "world_inspector") {
        egui_settings.single_mut().scale_factor = 2.0;
    }
}

/// This function sets up the character selection interface.
/// It spawns the options menu HTML node and associates the cleanup component with it.
pub(super) fn setup_character_selection_system(
    mut cmds: Commands,
    main_menu_assets: Res<MainMenuAssets>,
) {
    // Create an HTMLNode with options menu HTML and link the OptionsMenuCleanup component.
    cmds.spawn(HtmlNode(main_menu_assets.character_selection_html.clone()))
        .insert(CharacterSelectionCleanup);
}

/// This function sets up the options menu interface.
/// It spawns the options menu HTML node and associates the cleanup component with it.
pub(super) fn setup_options_menu_system(mut cmds: Commands, main_menu_assets: Res<MainMenuAssets>) {
    // Create an HTMLNode with options menu HTML and link the OptionsMenuCleanup component.
    cmds.spawn(HtmlNode(main_menu_assets.options_menu_html.clone()))
        .insert(OptionsMenuCleanup);
}

/// This system sets up the title menu interface.
/// It spawns the main menu HTML node and associates the cleanup component with it.
pub(super) fn setup_title_menu_system(mut cmds: Commands, main_menu_assets: Res<MainMenuAssets>) {
    // Create an HTMLNode with main menu HTML and link the TitleMenuCleanup component.
    cmds.spawn(HtmlNode(main_menu_assets.title_menu_html.clone()))
        .insert(TitleMenuCleanup);
}

/// Sets up website footer buttons with appropriate animations and actions
/// Takes an entity, queries for tags, and sets up the button based on its action type
fn setup_website_footer_button(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    main_menu_assets: Res<MainMenuAssets>,
) {
    if let Ok(tags) = tags.get(entity) {
        if let Some(button_action_str) = tags.get("button_action") {
            match ButtonAction::try_from(button_action_str) {
                Ok(button_action) => match button_action {
                    // Handle Bluesky website button - add animation and action
                    ButtonAction::OpenBlueskyWebsite => {
                        cmds.entity(entity)
                            .insert(AseUiAnimation {
                                animation: Animation::tag("released"),
                                aseprite: main_menu_assets.bluesky_logo_aseprite.clone(),
                            })
                            .insert(ButtonAction::OpenBlueskyWebsite);
                    }
                    // Handle Github website button - add animation and action
                    ButtonAction::OpenGithubWebsite => {
                        cmds.entity(entity)
                            .insert(AseUiAnimation {
                                animation: Animation::tag("released"),
                                aseprite: main_menu_assets.github_logo_aseprite.clone(),
                            })
                            .insert(ButtonAction::OpenGithubWebsite);
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
    }
}

/// Sets up menu button sprite animations based on whether it's the first button
/// Takes an entity, queries tags, and configures the animation state
fn setup_menu_button_sprite(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmds: Commands,
    main_menu_assets: Res<MainMenuAssets>,
) {
    // Get tags for the entity
    if let Ok(tags) = tags.get(entity) {
        // Check if this is marked as the first button
        if let Some(first_str) = tags.get("first") {
            // Insert animation component with pressed/released state based on first status
            cmds.entity(entity).insert(AseUiAnimation {
                animation: Animation::tag(if first_str == "true" {
                    "pressed"
                } else {
                    "released"
                }),
                aseprite: main_menu_assets.menu_button_aseprite.clone(),
            });
        }
    }
}

/// Sets up the title logo animation for the game's main menu
fn setup_title_logo(
    In(entity): In<Entity>,
    mut cmds: Commands,
    main_menu_assets: Res<MainMenuAssets>,
) {
    cmds.entity(entity).insert(AseUiAnimation {
        animation: Animation::tag("title").with_speed(1.25),
        aseprite: main_menu_assets.thetawave_logo_aseprite.clone(),
    });
}

// This function assigns actions to buttons based on their tags.
fn assign_action_to_menu_button(In(entity): In<Entity>, tags: Query<&Tags>, mut cmds: Commands) {
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
        }
    }
}

/// This function handles the opening of certain websites.
// It opens the URL in a web browser.
fn open_website(url: &str) {
    if webbrowser::open(url).is_ok() {
        // If opening the URL was successful, it is logged as an information.
        info!("Opening webiste: {url}");
    } else {
        // If opening the URL has failed, it is logged as a warning.
        warn!("Failed to open website: {url}");
    }
}

// This function inserts Focusable component into given entity.
fn attach_focusable(mut cmds: EntityCommands) {
    cmds.insert(Focusable::default());
}

/// System that handles the focus state of menu buttons
/// Updates the animation state of buttons when focus changes
/// Takes navigation events and queries for focusable entities and their animations
pub(super) fn menu_button_focus_system(
    mut nav_events: EventReader<NavEvent>,
    focusable_q: Query<&Children, With<Focusable>>,
    mut ase_q: Query<&mut AseUiAnimation>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            // Handle newly focused button - set to pressed animation
            if let Ok(children) = focusable_q.get(*to.first()) {
                for child in children.iter() {
                    if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                        ase_animation.animation.play_loop("pressed");
                    }
                }
            }

            // Handle previously focused button - set to released animation
            if let Ok(children) = focusable_q.get(*from.first()) {
                for child in children.iter() {
                    if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                        ase_animation.animation.play_loop("released");
                    }
                }
            }
        }
    }
}

/// System that handles the focus state of website footer buttons
/// Updates the animation state when focus changes between buttons
/// Takes navigation events and queries for focusable animations
pub(super) fn website_footer_button_focus_system(
    mut nav_events: EventReader<NavEvent>,
    mut focusable_q: Query<&mut AseUiAnimation, With<Focusable>>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            // Set newly focused button to pressed animation
            if let Ok(mut ase_animation) = focusable_q.get_mut(*to.first()) {
                ase_animation.animation.play_loop("pressed");
            }

            // Set previously focused button to released animation
            if let Ok(mut ase_animation) = focusable_q.get_mut(*from.first()) {
                ase_animation.animation.play_loop("released");
            }
        }
    }
}

/// This system reads and performs navigation events from bevy_alt_ui_navigation, handling each button action accordingly.
pub(super) fn menu_button_action_system(
    mut nav_events: EventReader<NavEvent>,
    focusable_q: Query<&ButtonAction, With<Focusable>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut exit_events: EventWriter<AppExit>,
    mut apply_options_events: EventWriter<ApplyOptionsEvent>,
) {
    for event in nav_events.read() {
        if let NavEvent::NoChanges { from, .. } = event {
            if let Ok(button_action) = focusable_q.get(*from.first()) {
                match button_action {
                    ButtonAction::EnterOptions => {
                        // Transition to the Options state.
                        next_main_menu_state.set(MainMenuState::Options);
                    }
                    ButtonAction::EnterCharacterSelection => {
                        // Transition to the CharacterSelection state.
                        next_main_menu_state.set(MainMenuState::CharacterSelection);
                    }
                    ButtonAction::Exit => {
                        // Trigger the AppExit event.
                        exit_events.send(AppExit::Success);
                    }
                    ButtonAction::ApplyOptions => {
                        // Trigger the ApplyOptionsEvent.
                        apply_options_events.send(ApplyOptionsEvent);
                    }
                    ButtonAction::EnterTitle => {
                        // Transition to the Title state.
                        next_main_menu_state.set(MainMenuState::Title);
                    }
                    ButtonAction::OpenBlueskyWebsite => {
                        // Open the web browser to navigate to the Bluesky website.
                        open_website(BLUESKY_URL);
                    }
                    ButtonAction::OpenGithubWebsite => {
                        // Open the web browser to navigate to the Github website.
                        open_website(GITHUB_URL);
                    }
                    ButtonAction::EnterGame => {
                        // Enter the game loading state and reset the main menu state
                        next_app_state.set(AppState::GameLoading);
                        next_main_menu_state.set(MainMenuState::None);
                    }
                }
            }
        }
    }
}

/// This function is a system that handles the egui options menu
pub(super) fn options_menu_system(mut contexts: EguiContexts, mut options_res: ResMut<OptionsRes>) {
    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT,       // Set transparent background
            inner_margin: egui::Margin::same(10.0), // Establish inner margin for UI layout
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            // Combo box for selecting Window Mode.
            ui.horizontal(|ui| {
                ui.label("Window Mode");
                egui::ComboBox::from_id_salt("window_mode_combobox")
                    .selected_text(format!(
                        "{}",
                        window_mode_to_string(&options_res.window_mode)
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::Windowed,
                            "Windowed",
                        );
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::Fullscreen(MonitorSelection::Current),
                            "Fullscreen",
                        );
                    });
            });

            // Combo box for selecting screen resolution.
            ui.horizontal(|ui| {
                ui.label("Resolution");
                egui::ComboBox::from_id_salt("resolution_combobox")
                    .selected_text(format!(
                        "{}",
                        window_resolution_to_string(&options_res.window_resolution)
                    ))
                    .show_ui(ui, |ui| {
                        // Iterate through every available resolution and create a selectable value
                        for resolution in options_res.get_resolutions() {
                            ui.selectable_value(
                                &mut options_res.window_resolution,
                                resolution.clone(),
                                window_resolution_to_string(&resolution),
                            );
                        }
                    });
            });
        });
}

/// Converts WindowMode enum to a string representation
/// Returns a string slice describing the window mode (e.g. "Windowed", "Fullscreen", etc.)
fn window_mode_to_string(mode: &WindowMode) -> &str {
    match mode {
        WindowMode::Windowed => "Windowed",
        WindowMode::BorderlessFullscreen(_) => "Borderless Fullscreen",
        WindowMode::Fullscreen(_) => "Fullscreen",
        WindowMode::SizedFullscreen(_) => "Sized Fullscreen",
    }
}

/// Converts WindowResolution to a formatted string
/// Takes a WindowResolution reference and returns a string in the format "WIDTHxHEIGHT"
fn window_resolution_to_string(resolution: &WindowResolution) -> String {
    let res_vec = resolution.size();
    format!("{}x{}", res_vec.x, res_vec.y)
}
