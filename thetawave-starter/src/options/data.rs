use bevy::{
    prelude::{Event, GamepadButton, KeyCode, MouseButton, Resource},
    window::{WindowMode, WindowResolution},
};
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use thetawave_player::{CharacterCarouselAction, PlayerAbility, PlayerAction};

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
    // Bloom effects enabled
    pub bloom_enabled: bool,
    // Keyboard input map for the player
    pub player_keyboard_action_input_mappings: HashMap<PlayerAction, KeyCode>,
    pub player_mouse_action_input_mappings: HashMap<PlayerAction, MouseButton>,
    // Keyboard input map for player abilities
    pub player_keyboard_abilities_input_mappings: HashMap<PlayerAbility, KeyCode>,
    pub player_mouse_abilities_input_mappings: HashMap<PlayerAbility, MouseButton>,
    // Gamepad input map for the player
    pub player_gamepad_action_input_mappings: HashMap<PlayerAction, GamepadButton>,
    // Gamepad input map for player abilities
    pub player_gamepad_abilities_input_mappings: HashMap<PlayerAbility, GamepadButton>,
    // Keyboard input map for the character carousel
    pub carousel_keyboard_input_mappings: HashMap<CharacterCarouselAction, KeyCode>,
    // Gamepad input map for the character carousel
    pub carousel_gamepad_input_mappings: HashMap<CharacterCarouselAction, GamepadButton>,
    // All resolution options available in options
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
            bloom_enabled: true,
            player_keyboard_action_input_mappings: HashMap::from([
                (PlayerAction::Up, KeyCode::KeyW),
                (PlayerAction::Down, KeyCode::KeyS),
                (PlayerAction::Left, KeyCode::KeyA),
                (PlayerAction::Right, KeyCode::KeyD),
                (PlayerAction::Pause, KeyCode::Escape),
            ]),
            player_mouse_action_input_mappings: HashMap::from([]),
            player_keyboard_abilities_input_mappings: HashMap::from([
                (PlayerAbility::Utility, KeyCode::AltLeft),
                (PlayerAbility::Ultimate, KeyCode::Space),
            ]),
            player_mouse_abilities_input_mappings: HashMap::from([
                (PlayerAbility::BasicAttack, MouseButton::Left),
                (PlayerAbility::SecondaryAttack, MouseButton::Right),
            ]),
            player_gamepad_action_input_mappings: HashMap::from([
                (PlayerAction::Up, GamepadButton::DPadUp),
                (PlayerAction::Down, GamepadButton::DPadDown),
                (PlayerAction::Left, GamepadButton::DPadLeft),
                (PlayerAction::Right, GamepadButton::DPadRight),
                (PlayerAction::Pause, GamepadButton::Start),
            ]),
            player_gamepad_abilities_input_mappings: HashMap::from([
                (PlayerAbility::BasicAttack, GamepadButton::South),
                (PlayerAbility::SecondaryAttack, GamepadButton::East),
                (PlayerAbility::Utility, GamepadButton::West),
                (PlayerAbility::Ultimate, GamepadButton::North),
            ]),
            carousel_keyboard_input_mappings: HashMap::from([
                (CharacterCarouselAction::CycleLeft, KeyCode::KeyA),
                (CharacterCarouselAction::CycleRight, KeyCode::KeyD),
                (CharacterCarouselAction::CycleLeft, KeyCode::ArrowLeft),
                (CharacterCarouselAction::CycleRight, KeyCode::ArrowRight),
                (CharacterCarouselAction::Ready, KeyCode::Enter),
                (CharacterCarouselAction::Unready, KeyCode::Escape),
            ]),
            carousel_gamepad_input_mappings: HashMap::from([
                (CharacterCarouselAction::CycleLeft, GamepadButton::DPadLeft),
                (
                    CharacterCarouselAction::CycleRight,
                    GamepadButton::DPadRight,
                ),
                (CharacterCarouselAction::Ready, GamepadButton::South),
                (CharacterCarouselAction::Unready, GamepadButton::East),
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
