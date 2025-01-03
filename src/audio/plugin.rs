use bevy::{app::Plugin, prelude::OnEnter};
use bevy_kira_audio::AudioPlugin;

use crate::states::AppState;

use super::systems::play_background_music;

pub(crate) struct ThetawaveAudioPlugin;

impl Plugin for ThetawaveAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AudioPlugin)
            .add_systems(OnEnter(AppState::MainMenu), play_background_music);
    }
}
