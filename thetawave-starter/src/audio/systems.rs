use crate::options::OptionsRes;
use bevy::prelude::{MessageReader, MessageWriter, Res, StateTransitionEvent};
use bevy_kira_audio::{AudioChannel, AudioControl, AudioTween, prelude::Decibels};
use bevy_persistent::Persistent;
use std::time::Duration;
use thetawave_assets::{AssetResolver, MergedMusicAssets, MergedUiAssets};
use thetawave_core::AppState;

use super::{
    MusicTransitionEvent,
    data::{
        AudioEffectEvent, ChangeVolumeEvent, EffectsAudioChannel, MusicAudioChannel, UiAudioChannel,
    },
};

const AUDIO_FADE: AudioTween = AudioTween::linear(Duration::from_secs(2));

/// Convert amplitude (0.0-1.0) to Decibels
fn amplitude_to_decibels(amplitude: f64) -> Decibels {
    if amplitude <= 0.0 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * (amplitude as f32).log10())
    }
}

/// Start a new track of music on the music audio channel
pub(super) fn start_music_system(
    music_assets: Res<MergedMusicAssets>,
    mut music_trans_events: MessageWriter<MusicTransitionEvent>,
    mut state_trans_events: MessageReader<StateTransitionEvent<AppState>>,
) -> bevy::ecs::error::Result {
    for event in state_trans_events.read() {
        if let Some(entered_state) = event.entered {
            match entered_state {
                AppState::MainMenu => {
                    music_trans_events.write(MusicTransitionEvent {
                        music: AssetResolver::get_music("main_menu_theme", &music_assets)?,
                    });
                }
                AppState::Game => {
                    music_trans_events.write(MusicTransitionEvent {
                        music: AssetResolver::get_music("game_theme", &music_assets)?,
                    });
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// System for playing audio effects, listens for AudioEffectEvents
pub(super) fn play_effect_system(
    ui_assets: Res<MergedUiAssets>,
    mut effect_events: MessageReader<AudioEffectEvent>,
    ui_audio_channel: Res<AudioChannel<UiAudioChannel>>,
    options_res: Res<Persistent<OptionsRes>>,
) -> bevy::ecs::error::Result {
    if !effect_events.is_empty() {
        // volume for ui event channel (convert amplitude to decibels)
        let ui_volume = amplitude_to_decibels(options_res.master_volume * options_res.ui_volume);

        // play all audio effect events in queue on correct channel
        for event in effect_events.read() {
            match event {
                AudioEffectEvent::MenuButtonSelected => {
                    ui_audio_channel
                        .play(AssetResolver::get_random_button_press_effect(&ui_assets)?)
                        .with_volume(ui_volume);
                }
                AudioEffectEvent::MenuButtonReleased => {
                    ui_audio_channel
                        .play(AssetResolver::get_random_button_release_effect(&ui_assets)?)
                        .with_volume(ui_volume);
                }
                AudioEffectEvent::MenuButtonConfirm => {
                    ui_audio_channel
                        .play(AssetResolver::get_random_button_confirm_effect(&ui_assets)?)
                        .with_volume(ui_volume);
                }
            }
        }
    }
    Ok(())
}

/// Start transition between audio tracks on the music audio channel
pub(super) fn transition_music_system(
    audio_channel: Res<AudioChannel<MusicAudioChannel>>,
    mut music_trans_events: MessageReader<MusicTransitionEvent>,
    options_res: Res<Persistent<OptionsRes>>,
) {
    for event in music_trans_events.read() {
        // Fade out of existing audio if playing
        if audio_channel.is_playing_sound() {
            audio_channel.stop().fade_out(AUDIO_FADE);
        }

        // Convert amplitude to decibels
        let volume = amplitude_to_decibels(options_res.master_volume * options_res.music_volume);

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
    mut change_volume_events: MessageReader<ChangeVolumeEvent>,
) {
    for event in change_volume_events.read() {
        music_audio_channel.set_volume(amplitude_to_decibels(event.music_volume));
        effects_audio_channel.set_volume(amplitude_to_decibels(event.effects_volume));
        ui_audio_channel.set_volume(amplitude_to_decibels(event.ui_volume));
    }
}
