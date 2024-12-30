use bevy::{
    prelude::{Event, Resource},
    window::{WindowMode, WindowResolution},
};

// Resource for storing window options
#[derive(Resource)]
pub(crate) struct OptionsRes {
    // The current window mode (fullscreen, windowed, etc)
    pub window_mode: WindowMode,
    // The current window resolution
    pub window_resolution: WindowResolution,
    // All of the available resolutions
    resolutions: Vec<WindowResolution>,
}

impl Default for OptionsRes {
    fn default() -> Self {
        Self {
            window_mode: WindowMode::Windowed,
            window_resolution: WindowResolution::new(1280., 720.),
            resolutions: vec![
                WindowResolution::new(800., 600.),
                WindowResolution::new(1024., 768.),
                WindowResolution::new(1280., 720.),
                WindowResolution::new(1280., 800.),
                WindowResolution::new(1280., 960.),
                WindowResolution::new(1366., 768.),
                WindowResolution::new(1440., 900.),
                WindowResolution::new(1600., 900.),
                WindowResolution::new(1680., 1050.),
                WindowResolution::new(1600., 1200.),
                WindowResolution::new(1920., 1080.),
                WindowResolution::new(1920., 1200.),
            ],
        }
    }
}

impl OptionsRes {
    pub(crate) fn get_resolutions(&self) -> Vec<WindowResolution> {
        self.resolutions.clone()
    }
}

// Event triggered when options should be applied
#[derive(Event)]
pub(crate) struct ApplyOptionsEvent;
