use super::data::SaveRes;
use bevy::ecs::system::Commands;
use bevy_persistent::{Persistent, StorageFormat};
use std::path::Path;

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
