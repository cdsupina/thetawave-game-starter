use bevy::ecs::system::ResMut;

use crate::ChosenCharactersResource;

/// Reset the ChosenCharactersResource when entering the title state
pub(crate) fn reset_chosen_characters_resource_system(
    mut chosen_characters_res: ResMut<ChosenCharactersResource>,
) {
    *chosen_characters_res = ChosenCharactersResource::default();
}
