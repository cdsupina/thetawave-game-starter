use bevy::prelude::{Component, Resource};
use bevy_platform::collections::HashMap;
use strum_macros::AsRefStr;
use thetawave_player::{CharacterType, InputType};

/// Tag for indicating multiplayer association
#[derive(Component, Debug, Clone, PartialEq, Eq, AsRefStr, Hash, PartialOrd, Ord)]
pub(crate) enum PlayerNum {
    One,
    Two,
    Three,
    Four,
}

impl PlayerNum {
    pub(crate) fn next(&self) -> Option<Self> {
        match self {
            PlayerNum::One => Some(Self::Two),
            PlayerNum::Two => Some(Self::Three),
            PlayerNum::Three => Some(Self::Four),
            PlayerNum::Four => None,
        }
    }
}

impl TryFrom<&String> for PlayerNum {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "player_one" => Ok(Self::One),
            "player_two" => Ok(Self::Two),
            "player_three" => Ok(Self::Three),
            "player_four" => Ok(Self::Four),
            _ => Err("Invalid player".to_string()),
        }
    }
}

/// Resource for transferring character choices from character selection screen to game
#[derive(Resource, Default, Debug)]
pub(crate) struct ChosenCharactersResource {
    pub players: HashMap<PlayerNum, ChosenCharacterData>,
}

impl ChosenCharactersResource {
    pub(crate) fn contains_input(&self, input_type: InputType) -> bool {
        for (_, data) in self.players.iter() {
            if data.input == input_type {
                return true;
            }
        }
        false
    }

    /// Finds the next available PlayerNum (not yet in ChosenCharactersResource)
    pub(crate) fn next_available_player_num(&self) -> Option<PlayerNum> {
        let max_used_player_num = self.players.keys().max().cloned();

        if let Some(player_num) = max_used_player_num {
            player_num.next()
        } else {
            None
        }
    }
}

/// Resource for transferring character choices from character selection screen to game
#[derive(Clone, Debug)]
pub(crate) struct ChosenCharacterData {
    pub character: CharacterType,
    pub input: InputType,
}
