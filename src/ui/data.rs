use bevy::{
    prelude::Resource,
    window::{WindowMode, WindowResolution},
};

#[derive(Resource, Default)]
pub(super) struct OptionsRes {
    pub window_mode: WindowMode,
    pub window_resolution: WindowResolution,
}
