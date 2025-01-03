use bevy::prelude::Res;
use iyes_progress::ProgressTracker;

use crate::states::AppState;

pub(super) fn print_progress(progress: Res<ProgressTracker<AppState>>) {
    let progress = progress.get_global_progress();
    println!("{}/{}", progress.done, progress.total);
}
