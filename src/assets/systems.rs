use bevy::prelude::{EventWriter, Res};
use iyes_progress::ProgressTracker;

use crate::states::AppState;

use super::LoadingProgressEvent;

/// System for getting loading progress and sending the value as an event
pub(super) fn get_loading_progress_system(
    progress: Res<ProgressTracker<AppState>>,
    mut loading_event_writer: EventWriter<LoadingProgressEvent>,
) {
    let progress = progress.get_global_progress();
    loading_event_writer.send(LoadingProgressEvent(
        progress.done as f32 / progress.total as f32,
    ));
}
