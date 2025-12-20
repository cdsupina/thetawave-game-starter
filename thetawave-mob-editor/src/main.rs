use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};

mod data;
mod file;
mod plugin;
mod preview;
mod states;
mod ui;

use plugin::MobEditorPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Thetawave Mob Editor".to_string(),
                        resolution: WindowResolution::new(1400, 900),
                        present_mode: PresentMode::AutoVsync,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(MobEditorPlugin::default())
        .run();
}
