use bevy::{
    asset::Handle,
    prelude::{Component, Message, Resource},
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

/// Message for triggering background music transitions
#[derive(Message)]
pub(crate) struct MusicTransitionEvent {
    pub music: Handle<AudioSource>,
}

/// Message for changing volumes
#[derive(Message)]
pub(crate) struct ChangeVolumeEvent {
    pub music_volume: f64,
    pub effects_volume: f64,
    pub ui_volume: f64,
}

/// Message for playing ui and effect events
#[derive(Message)]
pub(crate) enum AudioEffectEvent {
    MenuButtonSelected, // ui channel
    MenuButtonReleased, // ui channel
    MenuButtonConfirm,  // ui channel
}
