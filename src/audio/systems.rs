use bevy::prelude::Res;
use bevy_kira_audio::{Audio, AudioControl};

use crate::assets::AppAudioAssets;

pub(super) fn play_background_music(app_audio_assets: Res<AppAudioAssets>, audio: Res<Audio>) {
    audio.play(app_audio_assets.main_theme_soundtrack.clone());
}
