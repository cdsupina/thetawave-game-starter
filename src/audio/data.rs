use bevy::{
    asset::Handle,
    prelude::{Component, Event, Resource},
};
use bevy_kira_audio::AudioSource;

/// Audio channel for playing background music
#[derive(Resource, Component, Default, Clone)]
pub(super) struct MusicAudioChannel;

/// Audio channel for playing sound effects
#[derive(Resource, Component, Default, Clone)]
pub(super) struct EffectsAudioChannel;

/// Audio channel for playing ui sounds
#[derive(Resource, Component, Default, Clone)]
pub(super) struct UiAudioChannel;

/// Event for triggering background music transitions
#[derive(Event)]
pub(crate) struct MusicTransitionEvent {
    pub music: Handle<AudioSource>,
}
/// Event for changing volumes
#[derive(Event)]
pub(crate) struct ChangeVolumeEvent {
    pub music_volume: f64,
    pub effects_volume: f64,
    pub ui_volume: f64,
}
