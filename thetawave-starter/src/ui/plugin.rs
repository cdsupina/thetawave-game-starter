use super::{
    GameEndResultResource,
    data::{DelayedButtonPressEvent, PlayerReadyEvent},
    systems::{
        character_selection::{
            additional_players_join_system, carousel_input_system,
            cycle_player_one_carousel_system, enable_join_button_system,
            enable_start_game_button_system, lock_in_player_button_system, set_characters_system,
            spawn_carousel_system, spawn_character_selection_system, spawn_join_prompt_system,
            spawn_ready_button_system, update_carousel_ui_system,
        },
        egui::{setup_egui_system, update_egui_scale_system},
        game_end::{reset_game_end_result_resource_system, spawn_game_end_system},
        input_rebinding::{input_rebinding_menu_system, spawn_input_rebinding_menu_system},
        loading::{setup_loading_ui_system, update_loading_bar_system},
        menu_button_action_system, menu_button_delayed_action_system, menu_button_focus_system,
        options::{options_menu_system, persist_options_system, spawn_options_menu_system},
        pause::{spawn_pause_menu_system, spawn_pause_options_system},
        title::{spawn_title_menu_system, website_footer_button_focus_system},
    },
};
use bevy::{
    app::{Plugin, Update},
    prelude::{Condition, IntoScheduleConfigs, OnEnter, in_state},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_asset_loader::loading_state::LoadingStateSet;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use thetawave_states::{AppState, GameState, MainMenuState, PauseMenuState};

// Plugin responsible for managing the Thetawave user interface components and systems
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Initialize required UI plugins - HuiPlugin for UI components and EguiPlugin for immediate mode GUI
        app.add_plugins(EguiPlugin::default())
            .init_resource::<GameEndResultResource>()
            .add_event::<PlayerReadyEvent>()
            .add_event::<DelayedButtonPressEvent>()
            .add_systems(OnEnter(AppState::MainMenuLoading), setup_loading_ui_system)
            // Setup core UI components and main menu systems when entering the MainMenu state
            .add_systems(OnEnter(AppState::MainMenu), setup_egui_system)
            // Initialize and setup the title menu UI components when entering Title state
            .add_systems(
                OnEnter(MainMenuState::Title),
                (spawn_title_menu_system, update_egui_scale_system),
            )
            // Initialize and setup the options menu UI components when entering Options state
            .add_systems(OnEnter(MainMenuState::Options), spawn_options_menu_system)
            // Initialize and setup the input rebinding menu Main Menu state
            .add_systems(
                OnEnter(MainMenuState::InputRebinding),
                spawn_input_rebinding_menu_system,
            )
            // Initialize and setup the character selection UI components when entering Character Selection state
            .add_systems(
                OnEnter(MainMenuState::CharacterSelection),
                spawn_character_selection_system,
            )
            // Initialize and setup the pause menu ui components when entering the paused state
            .add_systems(OnEnter(PauseMenuState::Main), spawn_pause_menu_system)
            // Initialize and setup the options pause menu when inetering the paused options state
            .add_systems(OnEnter(PauseMenuState::Options), spawn_pause_options_system)
            // Add update systems that run every frame:
            .add_systems(
                Update,
                (
                    menu_button_action_system.after(NavRequestSystem),
                    menu_button_focus_system.after(NavRequestSystem),
                    website_footer_button_focus_system.after(NavRequestSystem),
                    menu_button_delayed_action_system,
                    update_loading_bar_system
                        .run_if(
                            in_state(AppState::MainMenuLoading).or(in_state(AppState::GameLoading)),
                        )
                        .after(LoadingStateSet(AppState::MainMenuLoading))
                        .after(LoadingStateSet(AppState::GameLoading)),
                ),
            )
            // Run options systems in main menu and pause menu options states
            .add_systems(
                EguiPrimaryContextPass,
                (
                    options_menu_system,
                    persist_options_system,
                    update_egui_scale_system,
                )
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            )
            // Run input rebinding menu system in the input rebinding menu
            .add_systems(
                EguiPrimaryContextPass,
                input_rebinding_menu_system.run_if(in_state(MainMenuState::InputRebinding)),
            )
            // Run carousel systems in character selection state
            .add_systems(
                Update,
                (
                    set_characters_system,
                    cycle_player_one_carousel_system,
                    update_carousel_ui_system,
                    spawn_carousel_system,
                    spawn_ready_button_system,
                    lock_in_player_button_system,
                    enable_start_game_button_system,
                    enable_join_button_system,
                    spawn_join_prompt_system,
                    additional_players_join_system,
                    carousel_input_system,
                )
                    .run_if(in_state(MainMenuState::CharacterSelection)),
            )
            .add_systems(OnEnter(GameState::End), spawn_game_end_system)
            .add_systems(
                OnEnter(AppState::GameLoading),
                reset_game_end_result_resource_system,
            );
    }
}
