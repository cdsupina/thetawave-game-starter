use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, not, Condition, IntoScheduleConfigs},
};
use bevy_kira_audio::{AudioApp, AudioPlugin};

use thetawave_core::{AppState, MainMenuState, PauseMenuState};

use super::{
    data::{AudioEffectEvent, EffectsAudioChannel, MusicAudioChannel, UiAudioChannel},
    systems::{
        change_volume_system, play_effect_system, start_music_system, transition_music_system,
    },
    ChangeVolumeEvent, MusicTransitionEvent,
};

pub(crate) struct ThetawaveAudioPlugin;

impl Plugin for ThetawaveAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AudioPlugin)
            .add_event::<MusicTransitionEvent>()
            .add_event::<ChangeVolumeEvent>()
            .add_event::<AudioEffectEvent>()
            .add_audio_channel::<MusicAudioChannel>()
            .add_audio_channel::<EffectsAudioChannel>()
            .add_audio_channel::<UiAudioChannel>()
            .add_systems(
                Update,
                (
                    start_music_system,
                    play_effect_system,
                    transition_music_system,
                )
                    .run_if(not(in_state(AppState::MainMenuLoading))),
            )
            .add_systems(
                Update,
                change_volume_system
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            );
    }
}
