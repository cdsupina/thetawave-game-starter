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
};
use bevy_alt_ui_navigation_lite::{
    events::NavEvent,
    prelude::{FocusState, Focusable},
};
use bevy_hui::prelude::{HtmlComponents, HtmlFunctions, HtmlNode};
use log::{info, warn};
use webbrowser;

/// Setup function for the main menu UI
/// Spawns the main menu HTML node and registers necessary functions and components
pub(super) fn setup_ui_system(
    mut cmd: Commands,
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
    main_menu_assets: Res<MainMenuAssets>,
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
