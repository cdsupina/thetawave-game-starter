use super::data::SaveRes;
use crate::ui::{GameEndResultResource, GameEndResultType};
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy_persistent::{Persistent, StorageFormat};
use std::path::Path;

/// Setup persistent save files
/// Load existing or default
pub(super) fn setup_save_res(mut cmds: Commands) {
    let config_dir = dirs::data_local_dir()
        .map(|native_config_dir| native_config_dir.join("thetawave_game_starter"))
        .unwrap_or(Path::new("local").join("data"));

    cmds.insert_resource(
        Persistent::<SaveRes>::builder()
            .name("save")
            .format(StorageFormat::Bincode)
            .path(config_dir.join("save_1"))
            .default(SaveRes::default())
            .build()
            .expect("failed to initialize save file"),
    )
}

/// Increment and save the run count
/// Should be called when entering AppState::Game
pub(super) fn increment_run_count_system(mut save_res: ResMut<Persistent<SaveRes>>) {
    save_res
        .update(|save_res| {
            save_res.run_count += 1;
        })
        .expect("Failed to update run count in save file");
}

/// Increment and save the win/loss count
/// Should be called when entering GameState::End
pub(super) fn increment_win_loss_count_system(
    mut save_res: ResMut<Persistent<SaveRes>>,
    game_end_result_res: Res<GameEndResultResource>,
) {
    save_res
        .update(|save_res| match game_end_result_res.result {
            GameEndResultType::Loss => save_res.loss_count += 1,
            GameEndResultType::Win => save_res.win_count += 1,
        })
        .expect("Failed to update win/loss count in save file");
}
