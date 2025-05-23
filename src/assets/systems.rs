use super::{GameAssets, LoadingProgressEvent};
use crate::states::AppState;
use bevy::{
    ecs::system::Commands,
    prelude::{EventWriter, Res},
};
use iyes_progress::ProgressTracker;

/// System for getting loading progress and sending the value as an event
pub(super) fn get_loading_progress_system(
    progress: Res<ProgressTracker<AppState>>,
    mut loading_event_writer: EventWriter<LoadingProgressEvent>,
) {
    let progress = progress.get_global_progress();
    loading_event_writer.write(LoadingProgressEvent(
        progress.done as f32 / progress.total as f32,
    ));
}

/// Unload game assets resource
/// Should be called once when exiting the game app state
pub(super) fn unload_game_assets_system(mut cmds: Commands) {
    cmds.remove_resource::<GameAssets>();
}
