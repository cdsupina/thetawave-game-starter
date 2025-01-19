use crate::{
    input::InputType,
    player::{CharacterType, PlayerNum},
    states::{AppState, GameState, MainMenuState, PauseMenuState},
};
use bevy::prelude::{Component, Event};
use strum::IntoEnumIterator;

/// All actions for menu buttons
#[derive(Component, Debug, Clone)]
pub(super) enum ButtonAction {
    EnterAppState(AppState),
    EnterMainMenuState(MainMenuState),
    EnterGameState(GameState),
    EnterPauseMenuState(PauseMenuState),
    Exit,
    ApplyOptions,
    OpenBlueskyWebsite,
    OpenGithubWebsite,
    Join(PlayerNum),
    Ready(PlayerNum),
    UnReady(PlayerNum),
}

/// Used for converting strings from hui tags into button actions
impl TryFrom<&String> for ButtonAction {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        // Sub tags have a colon delimeter
        let split_str: Vec<&str> = value.split(":").collect();

        let button_action_str = split_str[0];
        let maybe_player_str = split_str.get(1);

        match button_action_str {
            "enter_main_pause" => Ok(Self::EnterPauseMenuState(PauseMenuState::Main)),
            "enter_playing" => Ok(Self::EnterGameState(GameState::Playing)),
            "enter_main_menu_options" => Ok(Self::EnterMainMenuState(MainMenuState::Options)),
            "enter_pause_menu_options" => Ok(Self::EnterPauseMenuState(PauseMenuState::Options)),
            "enter_main_menu" => Ok(Self::EnterAppState(AppState::MainMenuLoading)),
            "enter_character_selection" => {
                Ok(Self::EnterMainMenuState(MainMenuState::CharacterSelection))
            }
            "enter_game" => Ok(Self::EnterAppState(AppState::GameLoading)),
            "exit" => Ok(Self::Exit),
            "apply_options" => Ok(Self::ApplyOptions),
            "enter_title" => Ok(Self::EnterMainMenuState(MainMenuState::Title)),
            "open_bluesky_website" => Ok(Self::OpenBlueskyWebsite),
            "open_github_website" => Ok(Self::OpenGithubWebsite),
            "join" => {
                if let Some(player_str) = maybe_player_str {
                    match PlayerNum::try_from(&player_str.to_string()) {
                        Ok(player_num) => Ok(ButtonAction::Join(player_num)),
                        Err(msg) => Err(msg),
                    }
                } else {
                    Err("No player string found, add player number in the format \"join:player_num\".".to_string())
                }
            }
            "ready" => {
                if let Some(player_str) = maybe_player_str {
                    match PlayerNum::try_from(&player_str.to_string()) {
                        Ok(player_num) => Ok(ButtonAction::Ready(player_num)),
                        Err(msg) => Err(msg),
                    }
                } else {
                    Err("No player string found, add player number in the format \"ready:player_num\".".to_string())
                }
            }
            _ => Err("Invalid action".to_string()),
        }
    }
}

/// States representing how a button should function
#[derive(Component, Default, Clone)]
pub(super) enum MenuButtonState {
    #[default]
    Normal,
    // "Greyed out" button that can't be selected
    Disabled,
    // Green menu button that indicates player has pressed
    // Often used in multiplayer for indicating that a player is ready
    Ready,
}

impl TryFrom<&String> for MenuButtonState {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "normal" => Ok(Self::Normal),
            "disabled" => Ok(Self::Disabled),
            "ready" => Ok(Self::Ready),
            _ => Err("Invalid state".to_string()),
        }
    }
}

/// Loading bar tag component
#[derive(Component)]
pub(super) struct LoadingBar;

/// Carousel positions center is the selected one
pub(super) enum CarouselSlotPosition {
    Center,
    Right,
    Left,
}

/// Component for tagging and storing the position for carousel ui
#[derive(Component)]
pub(super) struct VisibleCarouselSlot(pub CarouselSlotPosition);

/// Component for storing the Vec which represents the rotating carousel
#[derive(Component, Clone)]
pub(super) struct CharacterCarousel {
    characters: Vec<CharacterType>,
    pub input_type: InputType,
}

impl CharacterCarousel {
    /// Create a new carousel from all character types
    pub(super) fn new(input_type: InputType) -> Self {
        Self {
            characters: CharacterType::iter().collect(),
            input_type,
        }
    }

    /// The selected (active) character is at index 0
    pub(super) fn get_active_character(&self) -> Option<&CharacterType> {
        self.characters.first()
    }

    /// The character to the right is at index 1
    pub(super) fn get_right_character(&self) -> Option<&CharacterType> {
        self.characters.get(1)
    }

    /// The character to the left is at the last index of the vec
    pub(super) fn get_left_character(&self) -> Option<&CharacterType> {
        self.characters.last()
    }

    /// Shifts every element to the left
    /// Wrapping the index 0 character back to the last element of the Vec
    pub(super) fn cycle_left(&mut self) {
        if !self.characters.is_empty() {
            let first = self.characters.remove(0);
            self.characters.push(first);
        }
    }

    /// Shifts every element to the right
    /// Wrapping the last index character to the first element of the Vec
    pub(super) fn cycle_right(&mut self) {
        if let Some(last) = self.characters.pop() {
            self.characters.insert(0, last);
        }
    }
}

/// Event for when a player presses a join button on character selection screen
#[derive(Event, Debug)]
pub(crate) struct PlayerJoinEvent {
    pub player_num: PlayerNum,
    pub input: InputType,
}

/// Tag for container holding the carousel and arrows for character selection
#[derive(Component)]
pub(super) struct CharacterSelector;

/// Event for sending when player ready or unreadys on the character selection screen
#[derive(Event)]
pub(super) struct PlayerReadyEvent {
    pub player_num: PlayerNum,
    pub is_ready: bool,
}

/// Tag for ready button entities
#[derive(Component)]
pub(super) struct PlayerReadyButton;

/// Tag for button for entering GameLoading AppState
#[derive(Component)]
pub(super) struct StartGameButton;
