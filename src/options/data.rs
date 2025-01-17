use bevy::{
    prelude::{Event, GamepadButton, KeyCode, MouseButton, Resource},
    window::{WindowMode, WindowResolution},
};
use leafwing_input_manager::prelude::InputMap;
use serde::{Deserialize, Serialize};

use crate::input::{PlayerAbilities, PlayerAction};

// Resource for storing window options
#[derive(Resource, Serialize, Deserialize, Clone)]
pub(crate) struct OptionsRes {
    // The current window mode (fullscreen, windowed, etc)
    pub window_mode: WindowMode,
    // The current window resolution
    pub window_resolution: WindowResolution,
    // Maximum value of any audio channel
    pub master_volume: f64,
    // Volumes of the different audio channels, percentage of the master volume
    pub music_volume: f64,
    pub effects_volume: f64,
    pub ui_volume: f64,
    // Keyboard input map for the player
    pub player_keyboard_input_map: InputMap<PlayerAction>,
    // Keyboard input map for player abilities
    pub player_keyboard_abilities_input_map: InputMap<PlayerAbilities>,
    // Gamepad input map for the player
    pub player_gamepad_input_map: InputMap<PlayerAction>,
    // Gamepad input map for player abilities
    pub player_gamepad_abilities_input_map: InputMap<PlayerAbilities>,
    // All of the available resolutions
    resolutions: Vec<WindowResolution>,
}

impl Default for OptionsRes {
    fn default() -> Self {
        Self {
            window_mode: WindowMode::Windowed,
            window_resolution: WindowResolution::new(1280., 720.),
            resolutions: vec![
                WindowResolution::new(800., 600.),
                WindowResolution::new(1024., 768.),
                WindowResolution::new(1280., 720.),
                WindowResolution::new(1280., 800.),
                WindowResolution::new(1280., 960.),
                WindowResolution::new(1366., 768.),
                WindowResolution::new(1440., 900.),
                WindowResolution::new(1600., 900.),
                WindowResolution::new(1680., 1050.),
                WindowResolution::new(1600., 1200.),
                WindowResolution::new(1920., 1080.),
                WindowResolution::new(1920., 1200.),
            ],
            master_volume: 0.5,
            music_volume: 1.0,
            effects_volume: 1.0,
            ui_volume: 1.0,
            player_keyboard_input_map: InputMap::new([
                (PlayerAction::Up, KeyCode::KeyW),
                (PlayerAction::Down, KeyCode::KeyS),
                (PlayerAction::Left, KeyCode::KeyA),
                (PlayerAction::Right, KeyCode::KeyD),
            ]),
            player_keyboard_abilities_input_map: InputMap::new([
                (PlayerAbilities::Utility, KeyCode::AltLeft),
                (PlayerAbilities::Ultimate, KeyCode::Space),
            ])
            .insert_multiple([
                (PlayerAbilities::BasicAttack, MouseButton::Left),
                (PlayerAbilities::SecondaryAttack, MouseButton::Right),
            ])
            .to_owned(),
            player_gamepad_input_map: InputMap::new([
                (PlayerAction::Up, GamepadButton::DPadUp),
                (PlayerAction::Down, GamepadButton::DPadDown),
                (PlayerAction::Left, GamepadButton::DPadLeft),
                (PlayerAction::Right, GamepadButton::DPadRight),
            ]),
            player_gamepad_abilities_input_map: InputMap::new([
                (PlayerAbilities::BasicAttack, GamepadButton::South),
                (PlayerAbilities::SecondaryAttack, GamepadButton::East),
                (PlayerAbilities::Utility, GamepadButton::West),
                (PlayerAbilities::Ultimate, GamepadButton::North),
            ]),
        }
    }
}

impl OptionsRes {
    pub(crate) fn get_resolutions(&self) -> Vec<WindowResolution> {
        self.resolutions.clone()
    }
}

// Event triggered when options should be applied
#[derive(Event)]
pub(crate) struct ApplyOptionsEvent;
