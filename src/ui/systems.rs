use crate::{
    assets::MainMenuAssets,
    states::{AppState, MainMenuCleanup},
};
use bevy::{
    app::AppExit,
    color::palettes::css::{DARK_GRAY, ORANGE_RED},
    prelude::{
        Commands, Entity, EntityCommands, EventReader, EventWriter, In, NextState, Query, Res,
        ResMut,
    },
    ui::BackgroundColor,
    window::{MonitorSelection, WindowMode, WindowResolution},
};
use bevy_alt_ui_navigation_lite::{
    events::NavEvent,
    prelude::{FocusState, Focusable},
};
use bevy_egui::{egui, EguiContexts, EguiSettings};
use bevy_hui::prelude::{HtmlComponents, HtmlFunctions, HtmlNode};
use log::{info, warn};
use webbrowser;

use super::data::OptionsRes;

/// Setup function for the main menu UI
/// Spawns the main menu HTML node and registers necessary functions and components
pub(super) fn setup_ui_system(
    mut cmd: Commands,
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
    main_menu_assets: Res<MainMenuAssets>,
    mut egui_settings: Query<&mut EguiSettings>,
) {
    // Spawn the main menu HTML node with cleanup component
    cmd.spawn(HtmlNode(main_menu_assets.main_menu_html.clone()))
        .insert(MainMenuCleanup);

    // Register action handlers for HTML button clicks:
    // - Play button: Start a new game by entering character selection
    // - Options button: Open the game options menu
    // - Exit button: Close the application
    // - Social links: Open external websites for Bluesky and GitHub
    html_funcs.register("main_menu_play_action", enter_character_selection);
    html_funcs.register("main_menu_options_action", enter_options);
    html_funcs.register("main_menu_exit_action", exit_app);
    html_funcs.register("open_bluesky_action", open_bluesky_website);
    html_funcs.register("open_github_action", open_github_website);

    // Register footer button component for website links
    html_comps.register(
        "website_footer_button",
        main_menu_assets.website_footer_button_html.clone(),
    );
    // Register main menu button component and attach focus behavior
    html_comps.register_with_spawn_fn(
        "menu_button",
        main_menu_assets.menu_button_html.clone(),
        spawn_menu_button,
    );

    egui_settings.single_mut().scale_factor = 2.0;
}

/// Handler for the start game action
fn enter_character_selection(In(entity): In<Entity>, mut next_state: ResMut<NextState<AppState>>) {
    info!("{entity} pressed. Entering AppState::CharacterSelectionMenu.");
    next_state.set(AppState::CharacterSelectionMenu);
}

/// Handler for the start game action
fn enter_options(In(entity): In<Entity>, mut next_state: ResMut<NextState<AppState>>) {
    info!("{entity} pressed. Entering AppState::OptionsMenu.");
    next_state.set(AppState::OptionsMenu);
}

/// Handler for the start game action
fn exit_app(In(entity): In<Entity>, mut exit: EventWriter<AppExit>) {
    info!("{entity} pressed. Exiting game.");
    exit.send(AppExit::Success);
}

/// Handler for opening the Bluesky profile website
fn open_bluesky_website(In(entity): In<Entity>) {
    if webbrowser::open("https://bsky.app/profile/carlo.metalmancy.tech").is_ok() {
        info!("{entity} pressed. Opening Bluesky webiste link.");
    } else {
        warn!("{entity} was pressed, but Bluesky website failed to open.");
    }
}

/// Handler for opening the GitHub repository website
fn open_github_website(In(entity): In<Entity>) {
    if webbrowser::open("https://github.com/thetawavegame/thetawave").is_ok() {
        info!("{entity} pressed. Opening Github webiste link.");
    } else {
        warn!("{entity} was pressed, but Github website failed to open.");
    }
}

/// Spawn function for menu buttons
/// Adds Focusable component to menu button entities
fn spawn_menu_button(mut cmds: EntityCommands) {
    cmds.insert(Focusable::default());
}

/// System to handle button focus states and colors
/// Changes button background color based on focus state
pub(super) fn button_system(mut interaction_query: Query<(&Focusable, &mut BackgroundColor)>) {
    for (focusable, mut color) in interaction_query.iter_mut() {
        if let FocusState::Focused = focusable.state() {
            color.0 = ORANGE_RED.into();
        } else {
            color.0 = DARK_GRAY.into();
        }
    }
}

/// Debug system to print navigation events
pub(super) fn print_nav_events(mut events: EventReader<NavEvent>) {
    for event in events.read() {
        println!("{:?}", event);
    }
}

pub(super) fn options_menu_system(mut contexts: EguiContexts, mut options_res: ResMut<OptionsRes>) {
    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT, // Fully transparent background
            inner_margin: egui::Margin::same(10.0), // Adjust margin as needed for spacing
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            // Center the combo box
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
                            WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                            "Borderless Fullscreen",
                        );
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::Fullscreen(MonitorSelection::Current),
                            "Fullscreen",
                        );
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::SizedFullscreen(MonitorSelection::Current),
                            "Sized Fullscreen",
                        );
                    });
            });

            // Center the combo box
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

fn window_mode_to_string(mode: &WindowMode) -> &str {
    match mode {
        WindowMode::Windowed => "Windowed",
        WindowMode::BorderlessFullscreen(_) => "Borderless Fullscreen",
        WindowMode::Fullscreen(_) => "Fullscreen",
        WindowMode::SizedFullscreen(_) => "Sized Fullscreen",
    }
}

fn window_resolution_to_string(resolution: &WindowResolution) -> String {
    let res_vec = resolution.size();

    format!("{}x{}", res_vec.x, res_vec.y)
}
