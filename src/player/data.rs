use bevy::reflect::Reflect;
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect)]
pub(super) enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
}
