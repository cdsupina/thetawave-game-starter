use std::time::Duration;

use bevy::prelude::Res;
use bevy_kira_audio::{AudioChannel, AudioControl, AudioTween};

use crate::assets::AppAudioAssets;

use super::data::MusicAudioChannel;

const AUDIO_FADE: AudioTween = AudioTween::linear(Duration::from_secs(2));

/// Start a new track of music on the music audio channel
pub(super) fn play_music_system(
    app_audio_assets: Res<AppAudioAssets>,
    audio_channel: Res<AudioChannel<MusicAudioChannel>>,
) {
    audio_channel
        .play(app_audio_assets.main_menu_theme.clone())
        .looped()
        .fade_in(AUDIO_FADE);
}

/// Start transition between audio tracks on the music audio channel
pub(super) fn transition_music_system(
    app_audio_assets: Res<AppAudioAssets>,
    audio_channel: Res<AudioChannel<MusicAudioChannel>>,
) {
    audio_channel.stop().fade_out(AUDIO_FADE);
    audio_channel
        .play(app_audio_assets.game_theme.clone())
        .looped()
        .fade_in(AUDIO_FADE);
}
