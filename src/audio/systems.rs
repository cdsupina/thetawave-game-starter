use std::time::Duration;

use bevy::prelude::{EventReader, EventWriter, Res, StateTransitionEvent};
use bevy_kira_audio::{AudioChannel, AudioControl, AudioTween};

use crate::{assets::AppAudioAssets, states::AppState};

use super::{data::MusicAudioChannel, MusicTransitionEvent};

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
                    music_trans_events.send(MusicTransitionEvent {
                        music: app_audio_assets.main_menu_theme.clone(),
                    });
                }
                AppState::Game => {
                    music_trans_events.send(MusicTransitionEvent {
                        music: app_audio_assets.game_theme.clone(),
                    });
                }
                _ => {}
            }
        }
    }
}

/// Start transition between audio tracks on the music audio channel
pub(super) fn transition_music_system(
    audio_channel: Res<AudioChannel<MusicAudioChannel>>,
    mut music_trans_events: EventReader<MusicTransitionEvent>,
) {
    for event in music_trans_events.read() {
        // Fade out of exising audio if playing
        if audio_channel.is_playing_sound() {
            audio_channel.stop().fade_out(AUDIO_FADE);
        }

        // Fade into new audio
        audio_channel
            .play(event.music.clone())
            .looped()
            .fade_in(AUDIO_FADE);
    }
}
