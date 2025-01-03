use bevy::prelude::{Component, Resource};

#[derive(Resource, Component, Default, Clone)]
pub(super) struct MusicAudioChannel;

#[derive(Resource, Component, Default, Clone)]
pub(super) struct EffectsAudioChannel;

#[derive(Resource, Component, Default, Clone)]
pub(super) struct UiAudioChannel;
