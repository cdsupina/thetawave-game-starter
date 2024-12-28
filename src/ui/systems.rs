use crate::{
    assets::MainMenuAssets,
    options::{ApplyOptionsEvent, OptionsRes},
    states::{MainMenuState, OptionsMenuCleanup, TitleMenuCleanup},
};
use bevy::{
    app::AppExit,
    color::palettes::css::{DARK_GRAY, ORANGE_RED},
    prelude::{
        Commands, Entity, EntityCommands, EventReader, EventWriter, In, NextState, Query, Res,
        ResMut, With,
    },
    ui::BackgroundColor,
    window::{MonitorSelection, WindowMode, WindowResolution},
};
use bevy_alt_ui_navigation_lite::{
    events::NavEvent,
    prelude::{FocusState, Focusable},
};
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
    html_funcs.register("assign_action", assign_action_to_button);

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

    // Register the phantom main menu button component.
    // It uses a spawn function to also establish the focus behaviour on it.
    html_comps.register_with_spawn_fn(
        "phantom_menu_button",
        main_menu_assets.phantom_menu_button_html.clone(),
        attach_focusable,
    );

    // Set the egui scale factor to 2.0, this ensures visible and readable UI.
    egui_settings.single_mut().scale_factor = 2.0;
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
    cmds.spawn(HtmlNode(main_menu_assets.main_menu_html.clone()))
        .insert(TitleMenuCleanup);
}

// This function assigns actions to buttons based on their tags.
fn assign_action_to_button(In(entity): In<Entity>, tags: Query<&Tags>, mut cmds: Commands) {
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
        info!("Opening Bluesky webiste: {url}");
    } else {
        // If opening the URL has failed, it is logged as a warning.
        warn!("Failed to open website: {url}");
    }
}

// This function inserts Focusable component into given entity.
fn attach_focusable(mut cmds: EntityCommands) {
    cmds.insert(Focusable::default());
}

/// This system handles button focus states and changes button background color accordingly.
pub(super) fn button_system(mut interaction_query: Query<(&Focusable, &mut BackgroundColor)>) {
    for (focusable, mut color) in interaction_query.iter_mut() {
        // If the button is in focus, it's color is set to ORANGE_RED.
        if let FocusState::Focused = focusable.state() {
            color.0 = ORANGE_RED.into();
        } else {
            // If the button is not focused, it's color is set back to DARK_GRAY.
            color.0 = DARK_GRAY.into();
        }
    }
}

/// This system reads and performs navigation events from bevy_alt_ui_navigation, handling each button action accordingly.
pub(super) fn menu_button_action_system(
    mut nav_events: EventReader<NavEvent>,
    focusable_q: Query<&ButtonAction, With<Focusable>>,
    mut next_state: ResMut<NextState<MainMenuState>>,
    mut exit_events: EventWriter<AppExit>,
    mut apply_options_events: EventWriter<ApplyOptionsEvent>,
) {
    for event in nav_events.read() {
        if let NavEvent::NoChanges { from, .. } = event {
            if let Ok(button_action) = focusable_q.get(*from.first()) {
                match button_action {
                    ButtonAction::EnterOptions => {
                        // Transition to the Options state.
                        next_state.set(MainMenuState::Options);
                    }
                    ButtonAction::EnterCharacterSelection => {
                        // Transition to the CharacterSelection state.
                        next_state.set(MainMenuState::CharacterSelection);
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
                        next_state.set(MainMenuState::Title);
                    }
                    ButtonAction::OpenBlueskyWebsite => {
                        // Open the web browser to navigate to the Bluesky website.
                        open_website(BLUESKY_URL);
                    }
                    ButtonAction::OpenGithubWebsite => {
                        // Open the web browser to navigate to the Github website.
                        open_website(GITHUB_URL);
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
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(800., 600.),
                            "800x600",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1024., 768.),
                            "1024x768",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1280., 720.),
                            "1280x720",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1280., 800.),
                            "1280x800",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1280., 960.),
                            "1280x960",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1366., 768.),
                            "1366x768",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1440., 900.),
                            "1440x900",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1600., 900.),
                            "1600x900",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1680., 1050.),
                            "1680x1050",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1600., 1200.),
                            "1600x1200",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1920., 1080.),
                            "1920x1080",
                        );
                        ui.selectable_value(
                            &mut options_res.window_resolution,
                            WindowResolution::new(1920., 1200.),
                            "1920x1200",
                        );
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
