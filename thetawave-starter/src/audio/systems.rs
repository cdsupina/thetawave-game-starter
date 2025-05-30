use crate::{assets::AppAudioAssets, options::OptionsRes};
use bevy::prelude::{EventReader, EventWriter, Res, StateTransitionEvent};
use bevy_kira_audio::{AudioChannel, AudioControl, AudioTween};
use bevy_persistent::Persistent;
use std::time::Duration;
use thetawave_states::AppState;

use super::{
    data::{
        AudioEffectEvent, ChangeVolumeEvent, EffectsAudioChannel, MusicAudioChannel, UiAudioChannel,
    },
    MusicTransitionEvent,
};

const AUDIO_FADE: AudioTween = AudioTween::linear(Duration::from_secs(2));

/// Start a new track of music on the music audio channel
pub(super) fn start_music_system(
    app_audio_assets: Res<AppAudioAssets>,
    mut music_trans_events: EventWriter<MusicTransitionEvent>,
    mut state_trans_events: EventReader<StateTransitionEvent<AppState>>,
) {
    for event in state_trans_events.read() {
        if let Some(entered_state) = event.entered {
            match entered_state {
                AppState::MainMenu => {
                    music_trans_events.write(MusicTransitionEvent {
                        music: app_audio_assets.main_menu_theme.clone(),
                    });
                }
                AppState::Game => {
                    music_trans_events.write(MusicTransitionEvent {
                        music: app_audio_assets.game_theme.clone(),
                    });
                }
                _ => {}
            }
        }
    }
}

/// System for playing audio effects, listens for AudioEffectEvents
pub(super) fn play_effect_system(
    app_audio_assets: Res<AppAudioAssets>,
    mut effect_events: EventReader<AudioEffectEvent>,
    //effects_audio_channel: Res<AudioChannel<EffectsAudioChannel>>,
    ui_audio_channel: Res<AudioChannel<UiAudioChannel>>,
    options_res: Res<Persistent<OptionsRes>>,
) {
    if !effect_events.is_empty() {
        // volume for ui event channel
        let ui_volume = options_res.master_volume * options_res.ui_volume;

        // play all audio effect events in queue on correct channel
        for event in effect_events.read() {
            match event {
                AudioEffectEvent::MenuButtonSelected => {
                    ui_audio_channel
                        .play(app_audio_assets.get_random_button_press_effect())
                        .with_volume(ui_volume);
                }
                AudioEffectEvent::MenuButtonReleased => {
                    ui_audio_channel
                        .play(app_audio_assets.get_random_button_release_effect())
                        .with_volume(ui_volume);
                }
                AudioEffectEvent::MenuButtonConfirm => {
                    ui_audio_channel
                        .play(app_audio_assets.get_random_button_confirm_effect())
                        .with_volume(ui_volume);
                }
            }
        }
    }
}

/// Start transition between audio tracks on the music audio channel
pub(super) fn transition_music_system(
    audio_channel: Res<AudioChannel<MusicAudioChannel>>,
    mut music_trans_events: EventReader<MusicTransitionEvent>,
    options_res: Res<Persistent<OptionsRes>>,
) {
    for event in music_trans_events.read() {
        // Fade out of exising audio if playing
        if audio_channel.is_playing_sound() {
            audio_channel.stop().fade_out(AUDIO_FADE);
        }

        // This value should be between 0.0 and 1.0
        let volume = options_res.master_volume * options_res.music_volume;

        // Fade into new audio
        audio_channel
            .play(event.music.clone())
            .looped()
            .with_volume(volume)
            .fade_in(AUDIO_FADE);
    }
}

/// Change volumes of audio channels when ChangeVolumenEvents are read
pub(super) fn change_volume_system(
    music_audio_channel: Res<AudioChannel<MusicAudioChannel>>,
    effects_audio_channel: Res<AudioChannel<EffectsAudioChannel>>,
    ui_audio_channel: Res<AudioChannel<UiAudioChannel>>,
    mut change_volume_events: EventReader<ChangeVolumeEvent>,
) {
    for event in change_volume_events.read() {
        music_audio_channel.set_volume(event.music_volume);
        effects_audio_channel.set_volume(event.effects_volume);
        ui_audio_channel.set_volume(event.ui_volume);
    }
}
