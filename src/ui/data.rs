use crate::{
    assets::UiAssets,
    input::InputType,
    player::{CharacterType, PlayerNum},
    states::{AppState, GameState, MainMenuState, PauseMenuState},
};
use bevy::{
    core::Name,
    hierarchy::{BuildChildren, ChildBuild, ChildBuilder},
    prelude::{Component, Entity, Event},
    text::TextFont,
    time::{Timer, TimerMode},
    ui::{widget::Text, AlignItems, FlexDirection, JustifyContent, JustifySelf, Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::prelude::Focusable;
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
use strum::IntoEnumIterator;

const BUTTON_ACTION_DELAY_TIME: f32 = 0.3;
const CAROUSEL_READY_TIME: f32 = 0.5;

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

impl ButtonAction {
    pub fn to_string(&self) -> Option<String> {
        match self {
            ButtonAction::EnterAppState(app_state) => match app_state {
                AppState::MainMenuLoading => Some("Main Menu".to_string()),
                AppState::MainMenu => None,
                AppState::GameLoading => Some("Start Game".to_string()),
                AppState::Game => None,
            },
            ButtonAction::EnterMainMenuState(main_menu_state) => match main_menu_state {
                MainMenuState::None => None,
                MainMenuState::Title => Some("Back".to_string()),
                MainMenuState::Options => Some("Options".to_string()),
                MainMenuState::CharacterSelection => Some("Play".to_string()),
            },
            ButtonAction::EnterGameState(game_state) => match game_state {
                GameState::Playing => Some("Resume".to_string()),
                GameState::Paused => None,
            },
            ButtonAction::EnterPauseMenuState(pause_menu_state) => match pause_menu_state {
                PauseMenuState::None => None,
                PauseMenuState::Main => None,
                PauseMenuState::Options => Some("Options".to_string()),
            },
            ButtonAction::Exit => Some("Exit".to_string()),
            ButtonAction::ApplyOptions => Some("Apply".to_string()),
            ButtonAction::OpenBlueskyWebsite => None,
            ButtonAction::OpenGithubWebsite => None,
            ButtonAction::Join(_) => Some("Join".to_string()),
            ButtonAction::Ready(_) => Some("Ready".to_string()),
            ButtonAction::UnReady(_) => Some("Unready".to_string()),
        }
    }
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

/// Timer for preventing players from instantly readying when joining
#[derive(Component)]
pub(super) struct CarouselReadyTimer(pub Timer);

impl CarouselReadyTimer {
    pub(super) fn new() -> Self {
        Self(Timer::from_seconds(CAROUSEL_READY_TIME, TimerMode::Once))
    }
}

/// Event for passing ButtonActions to an event for a delayed action
#[derive(Event, Clone)]
pub(super) struct DelayedButtonPressEvent {
    pub button_action: ButtonAction,
    pub button_entity: Entity,
}

/// Timer for delaying button actions for button press animation
pub(super) struct ButtonActionDelayTimer(pub Timer);

impl Default for ButtonActionDelayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            BUTTON_ACTION_DELAY_TIME,
            TimerMode::Repeating,
        ))
    }
}

pub(super) trait UiChildBuilderExt {
    fn spawn_character_selection(&mut self, ui_assets: &UiAssets, player_num: PlayerNum);

    fn spawn_menu_button(
        &mut self,
        ui_assets: &UiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
        is_disabled: bool,
    );
}

impl UiChildBuilderExt for ChildBuilder<'_> {
    /// Spawn a character selection ui for the provided character
    fn spawn_character_selection(&mut self, ui_assets: &UiAssets, player_num: PlayerNum) {
        self.spawn(Node {
            height: Val::Percent(100.0),
            width: Val::Percent(50.0),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            ..default()
        })
        .with_children(|parent| {
            // Spawn character selector
            let mut entity_cmds = parent.spawn((
                Node {
                    width: Val::Percent(85.0),
                    height: Val::Percent(80.0),
                    justify_self: JustifySelf::Start,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                CharacterSelector,
                player_num.clone(),
            ));

            // Spawn input prompt for player 1
            if matches!(player_num, PlayerNum::One) {
                entity_cmds.with_children(|parent| {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                AseUiAnimation {
                                    animation: Animation::tag("key_return"),
                                    aseprite: ui_assets.return_button_aseprite.clone(),
                                },
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                Name::new("Join Prompt Input"),
                            ));

                            parent.spawn((
                                AseUiAnimation {
                                    animation: Animation::tag("a"),
                                    aseprite: ui_assets.xbox_letter_buttons_aseprite.clone(),
                                },
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                Name::new("Join Prompt Input"),
                            ));
                        });
                });
            }

            // Spawn join button
            // Player 1 button is not disabled and is first
            parent.spawn_menu_button(
                ui_assets,
                ButtonAction::Join(player_num.clone()),
                300.0,
                matches!(player_num, PlayerNum::One),
                !matches!(player_num, PlayerNum::One),
            );
        });
    }

    /// Spawns a rectangular stylized menu button
    fn spawn_menu_button(
        &mut self,
        ui_assets: &UiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
        is_disabled: bool,
    ) {
        let mut entity_cmds = self.spawn_empty();

        // if a button is disabled do not spawn it in focusable
        if !is_disabled {
            entity_cmds.insert(if is_first {
                Focusable::default().prioritized() // prioritize the first button so that it is selected
            } else {
                Focusable::default()
            });
        }

        if let ButtonAction::EnterAppState(AppState::GameLoading) = button_action.clone() {
            entity_cmds.insert(StartGameButton);
        }

        entity_cmds
            .insert((
                Name::new("Menu Button"),
                Node {
                    margin: UiRect::all(Val::Vh(1.0)),
                    ..default()
                },
                button_action.clone(),
                if is_disabled {
                    MenuButtonState::Disabled
                } else {
                    MenuButtonState::Normal
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Name::new("Menu Button Sprite"),
                        Node {
                            width: Val::Px(width),
                            aspect_ratio: Some(162.0 / 39.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        AseUiAnimation {
                            animation: Animation::tag(if is_disabled {
                                "disabled"
                            } else if is_first {
                                "selected"
                            } else {
                                "released"
                            }),
                            aseprite: ui_assets.menu_button_aseprite.clone(),
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(Node {
                                margin: UiRect::bottom(Val::Px(14.0)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            })
                            .with_children(|parent| {
                                if let Some(button_text) = button_action.to_string() {
                                    parent.spawn((
                                        Text::new(button_text),
                                        TextFont::from_font_size(25.0),
                                    ));
                                }
                            });
                    });
            });
    }
}
