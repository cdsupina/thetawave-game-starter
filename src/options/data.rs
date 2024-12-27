use bevy::{
    prelude::{Event, Resource},
    window::{WindowMode, WindowResolution},
};

// Resource for storing window options
#[derive(Resource, Default)]
pub(crate) struct OptionsRes {
    // The current window mode (fullscreen, windowed, etc)
    pub window_mode: WindowMode,
    // The current window resolution
    pub window_resolution: WindowResolution,
}

// Event triggered when options should be applied
#[derive(Event)]
pub(crate) struct ApplyOptionsEvent;
