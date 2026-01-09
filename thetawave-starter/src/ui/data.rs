use bevy::{
    ecs::{resource::Resource, system::EntityCommands},
    prelude::{ChildSpawnerCommands, Component, Entity, Message, Name},
    text::TextFont,
    time::{Timer, TimerMode},
    ui::{
        AlignItems, FlexDirection, JustifyContent, JustifySelf, Node, UiRect, Val,
        widget::{ImageNode, Text},
    },
    utils::default,
};
use bevy_alt_ui_navigation_lite::prelude::Focusable;
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation};
use thetawave_assets::{AssetResolver, MergedUiAssets};
use thetawave_core::{AppState, GameState, MainMenuState, PauseMenuState};
use thetawave_player::{CharactersResource, InputType, PlayerNum};

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
                AppState::MainMenuLoading => Some("MAIN MENU".to_string()),
                AppState::GameLoading => Some("START GAME".to_string()),
                _ => None,
            },
            ButtonAction::EnterMainMenuState(main_menu_state) => match main_menu_state {
                MainMenuState::Title => Some("BACK".to_string()),
                MainMenuState::Options => Some("OPTIONS".to_string()),
                MainMenuState::CharacterSelection => Some("PLAY".to_string()),
                MainMenuState::InputRebinding => Some("INPUT".to_string()),
                _ => None,
            },
            ButtonAction::EnterGameState(GameState::Playing) => Some("RESUME".to_string()),
            ButtonAction::EnterGameState(_) => None,
            ButtonAction::EnterPauseMenuState(pause_menu_state) => match pause_menu_state {
                PauseMenuState::Main => Some("BACK".to_string()),
                PauseMenuState::Options => Some("OPTIONS".to_string()),
                _ => None,
            },
            ButtonAction::Exit => Some("EXIT".to_string()),
            ButtonAction::ApplyOptions => Some("APPLY".to_string()),
            ButtonAction::Join(_) => Some("JOIN".to_string()),
            ButtonAction::Ready(_) => Some("READY".to_string()),
            ButtonAction::UnReady(_) => Some("UNREADY".to_string()),
            _ => None,
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
    characters: Vec<String>,
    pub input_type: InputType,
}

impl CharacterCarousel {
    /// Create a new carousel from all character types
    pub(super) fn new(input_type: InputType, characters_resource: &CharactersResource) -> Self {
        let mut characters: Vec<String> = characters_resource.characters.keys().cloned().collect();
        characters.sort(); // Ensure consistent ordering across platforms
        Self {
            characters,
            input_type,
        }
    }

    /// The selected (active) character is at index 0
    pub(super) fn get_active_character(&self) -> Option<&String> {
        self.characters.first()
    }

    /// The character to the right is at index 1
    pub(super) fn get_right_character(&self) -> Option<&String> {
        self.characters.get(1)
    }

    /// The character to the left is at the last index of the vec
    pub(super) fn get_left_character(&self) -> Option<&String> {
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

/// Tag for container holding the carousel and arrows for character selection
#[derive(Component)]
pub(super) struct CharacterSelector;

/// Message for sending when player readys or unreadys on the character selection screen
#[derive(Message)]
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

/// Tag for container node entity for text
#[derive(Component)]
pub(super) struct MenuButtonTextContainer;

impl CarouselReadyTimer {
    pub(super) fn new() -> Self {
        Self(Timer::from_seconds(CAROUSEL_READY_TIME, TimerMode::Once))
    }
}

/// Message for passing ButtonActions for a delayed action
#[derive(Message, Clone)]
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
    fn spawn_join_prompt(&mut self, ui_assets: &MergedUiAssets);

    fn spawn_character_selection(
        &mut self,
        ui_assets: &MergedUiAssets,
        player_num: PlayerNum,
    );

    fn spawn_menu_button(
        &mut self,
        ui_assets: &MergedUiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
        is_disabled: bool,
    ) -> EntityCommands<'_>;
}

impl UiChildBuilderExt for ChildSpawnerCommands<'_> {
    /// Spawn a join prompt
    fn spawn_join_prompt(&mut self, ui_assets: &MergedUiAssets) {
        // Resolve assets outside the closure, panic on failure
        let return_button_sprite =
            AssetResolver::get_ui_sprite("return_button", ui_assets)
                .expect("Failed to load return_button sprite");
        let xbox_buttons_sprite =
            AssetResolver::get_ui_sprite("xbox_letter_buttons", ui_assets)
                .expect("Failed to load xbox_letter_buttons sprite");

        self.spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                AseAnimation {
                    animation: Animation::tag("key_return"),
                    aseprite: return_button_sprite,
                },
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                Name::new("Join Prompt Input"),
            ));
            parent.spawn((
                AseAnimation {
                    animation: Animation::tag("a"),
                    aseprite: xbox_buttons_sprite,
                },
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                Name::new("Join Prompt Input"),
            ));
        });
    }

    /// Spawn a character selection ui for the provided character
    fn spawn_character_selection(
        &mut self,
        ui_assets: &MergedUiAssets,
        player_num: PlayerNum,
    ) {
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
                    parent.spawn_join_prompt(ui_assets);
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
        ui_assets: &MergedUiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
        is_disabled: bool,
    ) -> EntityCommands<'_> {
        // Resolve assets outside the closures, panic on failure
        let menu_button_sprite =
            AssetResolver::get_ui_sprite("menu_button", ui_assets)
                .expect("Failed to load menu_button sprite");
        let font = AssetResolver::get_ui_font("Dank-Depths", ui_assets)
            .expect("Failed to load Dank-Depths font");

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
                        ImageNode::default(),
                        Node {
                            width: Val::Px(width),
                            aspect_ratio: Some(162.0 / 39.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        AseAnimation {
                            animation: Animation::tag(if is_disabled {
                                "disabled"
                            } else if is_first {
                                "selected"
                            } else {
                                "released"
                            }),
                            aseprite: menu_button_sprite,
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    margin: UiRect::bottom(Val::Px(18.0)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexEnd,
                                    ..default()
                                },
                                MenuButtonTextContainer,
                            ))
                            .with_children(|parent| {
                                if let Some(button_text) = button_action.to_string() {
                                    parent.spawn((
                                        Text::new(button_text),
                                        TextFont::from_font_size(20.0).with_font(font),
                                    ));
                                }
                            });
                    });
            });

        entity_cmds
    }
}

/// Enum for all types of run results
#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum GameEndResultType {
    #[default]
    Loss,
    Win,
}

impl From<GameEndResultType> for String {
    fn from(val: GameEndResultType) -> Self {
        match val {
            GameEndResultType::Loss => "GAME OVER".to_string(),
            GameEndResultType::Win => "VICTORY!".to_string(),
        }
    }
}

/// Resource for carrying the game result to the end screen and stats
#[derive(Resource, Default, Debug)]
pub(crate) struct GameEndResultResource {
    pub result: GameEndResultType,
}
