use bevy::app::{Plugin, Update};
use bevy_kira_audio::{AudioApp, AudioPlugin};

use super::{
    data::{EffectsAudioChannel, MusicAudioChannel, UiAudioChannel},
    systems::{change_volume_system, start_music_system, transition_music_system},
    ChangeVolumeEvent, MusicTransitionEvent,
};

pub(crate) struct ThetawaveAudioPlugin;

impl Plugin for ThetawaveAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AudioPlugin)
            .add_event::<MusicTransitionEvent>()
            .add_event::<ChangeVolumeEvent>()
            .add_audio_channel::<MusicAudioChannel>()
            .add_audio_channel::<EffectsAudioChannel>()
            .add_audio_channel::<UiAudioChannel>()
            .add_systems(Update, start_music_system)
            .add_systems(Update, transition_music_system)
            .add_systems(Update, change_volume_system);
    }
}
