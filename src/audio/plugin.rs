use bevy::{app::Plugin, prelude::OnEnter};
use bevy_kira_audio::{AudioApp, AudioPlugin};

use crate::states::AppState;

use super::{
    data::{EffectsAudioChannel, MusicAudioChannel, UiAudioChannel},
    systems::{play_music_system, transition_music_system},
};

pub(crate) struct ThetawaveAudioPlugin;

impl Plugin for ThetawaveAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<MusicAudioChannel>()
            .add_audio_channel::<EffectsAudioChannel>()
            .add_audio_channel::<UiAudioChannel>()
            .add_systems(OnEnter(AppState::MainMenu), play_music_system)
            .add_systems(OnEnter(AppState::Game), transition_music_system); // temporary for testing
    }
}
