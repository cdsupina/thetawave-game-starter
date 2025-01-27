use bevy::{
    app::{App, Startup},
    prelude::{DefaultPlugins, ImagePlugin, NonSend, PluginGroup},
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
    winit::WinitWindows,
};
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use winit::window::Icon;

mod assets;
mod audio;
mod backgrounds;
mod camera;
mod input;
mod options;
mod physics;
mod player;
mod states;
mod ui;

const PRIMARY_WINDOW_TITLE: &str = "Thetawave Starter Template";
const STARTING_WINDOW_RESOLUTION: (f32, f32) = (1280.0, 720.0);

fn main() {
    let mut app = App::new();

    app.add_plugins((
        assets::ThetawaveAssetsPlugin, // must be registered before AssetPlugin due to EmbeddedAssetPlugin
        DefaultPlugins
            .set(ImagePlugin::default_nearest()) // necessary for crisp pixel art
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::from(STARTING_WINDOW_RESOLUTION)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    title: PRIMARY_WINDOW_TITLE.to_string(),
                    ..default()
                }),
                ..default()
            }),
        AsepriteUltraPlugin, // plugin for using Aseprite assets
        // custom plugins for Thetawave
        ui::ThetawaveUiPlugin,
        options::ThetawaveOptionsPlugin,
        input::ThetawaveInputPlugin,
        states::ThetawaveStatesPlugin,
        camera::ThetawaveCameraPlugin,
        backgrounds::ThetawaveBackgroundsPlugin,
        audio::ThetawaveAudioPlugin,
        player::ThetawavePlayerPlugin,
        physics::ThetawavePhysicsPlugin,
    ))
    .add_systems(Startup, set_window_icon_system);

    if cfg!(feature = "world_inspector") {
        println!("here");
        app.add_plugins(WorldInspectorPlugin::new());
    }

    app.run();
}

fn set_window_icon_system(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/window_icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
