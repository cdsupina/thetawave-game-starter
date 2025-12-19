//! Systems for asset management in Thetawave.

use std::hash::Hash;

use bevy::{
    asset::Assets,
    ecs::system::Commands,
    log::info,
    platform::collections::HashMap,
    prelude::{MessageWriter, Res, ResMut},
};
use bevy_enoki::prelude::ColorParticle2dMaterial;
use iyes_progress::ProgressTracker;
use thetawave_core::{AppState, Faction};

use super::data::{
    BackgroundAssets, GameAssets, LoadingProgressEvent, MusicAssets, ParticleMaterials, UiAssets,
};
use crate::{
    ExtendedBackgroundAssets, ExtendedGameAssets, ExtendedMusicAssets, ExtendedUiAssets,
};

/// Helper to get length of an optional HashMap, returning 0 if None.
fn opt_map_len<K: Eq + Hash, V>(opt: &Option<HashMap<K, V>>) -> usize {
    opt.as_ref().map_or(0, |m| m.len())
}

/// Helper to get length of an optional Vec, returning 0 if None.
fn opt_vec_len<T>(opt: &Option<Vec<T>>) -> usize {
    opt.as_ref().map_or(0, |v| v.len())
}

/// System for getting loading progress and sending the value as a message
pub(super) fn get_loading_progress_system(
    progress: Res<ProgressTracker<AppState>>,
    mut loading_event_writer: MessageWriter<LoadingProgressEvent>,
) {
    let progress = progress.get_global_progress();
    loading_event_writer.write(LoadingProgressEvent(
        progress.done as f32 / progress.total as f32,
    ));
}

/// Setup particle materials with faction-specific colors
/// Should be called when entering the game loading state
pub(super) fn setup_particle_materials_system(
    mut cmds: Commands,
    mut materials: ResMut<Assets<ColorParticle2dMaterial>>,
) {
    // Create faction-specific color materials
    let ally_material = materials.add(ColorParticle2dMaterial::new(
        Faction::Ally.get_color().into(),
    ));

    let enemy_material = materials.add(ColorParticle2dMaterial::new(
        Faction::Enemy.get_color().into(),
    ));

    // Insert the ParticleMaterials resource
    cmds.insert_resource(ParticleMaterials {
        ally_material,
        enemy_material,
    });
}

/// Unload game assets resource
/// Should be called once when exiting the game app state
pub(super) fn unload_game_assets_system(mut cmds: Commands) {
    cmds.remove_resource::<GameAssets>();
    cmds.remove_resource::<ExtendedGameAssets>();
    cmds.remove_resource::<ParticleMaterials>();
}

/// Log loaded main menu assets (UI, music, backgrounds)
/// Should be called on entering MainMenu state after assets are loaded
pub(super) fn log_main_menu_assets_system(
    ui_assets: Res<UiAssets>,
    extended_ui_assets: Res<ExtendedUiAssets>,
    music_assets: Res<MusicAssets>,
    extended_music_assets: Res<ExtendedMusicAssets>,
    bg_assets: Res<BackgroundAssets>,
    extended_bg_assets: Res<ExtendedBackgroundAssets>,
) {
    // Log UI assets
    let ext_ui_sprites = opt_map_len(&extended_ui_assets.sprites);
    let ext_ui_images = opt_map_len(&extended_ui_assets.images);
    let ext_ui_fonts = opt_map_len(&extended_ui_assets.fonts);
    let ext_ui_btn_select = opt_vec_len(&extended_ui_assets.menu_button_select_effects);
    let ext_ui_btn_release = opt_vec_len(&extended_ui_assets.menu_button_release_effects);
    let ext_ui_btn_confirm = opt_vec_len(&extended_ui_assets.menu_button_confirm_effects);

    info!(
        "UI Assets: {} sprites (+{} extended), {} images (+{} extended), {} fonts (+{} extended)",
        ui_assets.sprites.len(), ext_ui_sprites,
        ui_assets.images.len(), ext_ui_images,
        ui_assets.fonts.len(), ext_ui_fonts,
    );
    info!(
        "UI Audio: {} select (+{} extended), {} release (+{} extended), {} confirm (+{} extended)",
        ui_assets.menu_button_select_effects.len(), ext_ui_btn_select,
        ui_assets.menu_button_release_effects.len(), ext_ui_btn_release,
        ui_assets.menu_button_confirm_effects.len(), ext_ui_btn_confirm,
    );

    // Log music assets
    let ext_music = opt_map_len(&extended_music_assets.music);
    info!(
        "Music Assets: {} tracks (+{} extended)",
        music_assets.music.len(), ext_music
    );

    // Log background assets
    let ext_backgrounds = opt_vec_len(&extended_bg_assets.space_backgrounds);
    let ext_planets = opt_vec_len(&extended_bg_assets.planets);
    info!(
        "Background Assets: {} backgrounds (+{} extended), {} planets (+{} extended)",
        bg_assets.space_backgrounds.len(), ext_backgrounds,
        bg_assets.planets.len(), ext_planets
    );
}

/// Log loaded game assets (sprites, particle effects)
/// Should be called on entering Game state after assets are loaded
pub(super) fn log_game_assets_system(
    game_assets: Res<GameAssets>,
    extended_game_assets: Res<ExtendedGameAssets>,
) {
    let ext_sprites = opt_map_len(&extended_game_assets.sprites);
    let ext_particles = opt_map_len(&extended_game_assets.particle_effects);

    info!(
        "Game Assets: {} sprites (+{} extended), {} particle effects (+{} extended)",
        game_assets.sprites.len(), ext_sprites,
        game_assets.particle_effects.len(), ext_particles
    );

    // Log extended sprite names if any
    if let Some(sprites) = &extended_game_assets.sprites {
        for key in sprites.keys() {
            info!("  Extended sprite: {:?}", key);
        }
    }

    // Log extended particle effect names if any
    if let Some(effects) = &extended_game_assets.particle_effects {
        for key in effects.keys() {
            info!("  Extended particle effect: {:?}", key);
        }
    }
}
