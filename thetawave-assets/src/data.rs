//! Asset data structures for Thetawave.
//!
//! Uses bevy_asset_loader's AssetCollection derive for managed asset loading.

use bevy::{
    asset::{Assets, Handle},
    color::Color,
    image::Image,
    platform::collections::HashMap,
    prelude::{Res, Resource},
    scene::Scene,
    text::Font,
};
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileStem};
use bevy_enoki::{Particle2dEffect, prelude::ColorParticle2dMaterial};
use bevy_kira_audio::AudioSource;
use rand::Rng;
use thetawave_core::Faction;

use bevy::prelude::Message;

use crate::AssetError;

// ============================================================================
// Game Assets
// ============================================================================

/// Assets used in the game state
#[derive(Resource, AssetCollection)]
pub struct GameAssets {
    #[asset(key = "game_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "game_particle_effects", collection(typed, mapped))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Additional assets used in the game state that are not built in to thetawave-assets
#[derive(Resource, Default, Clone, AssetCollection)]
pub struct ExtendedGameAssets {
    #[asset(key = "extended_game_sprites", collection(typed, mapped), optional)]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[asset(
        key = "extended_game_particle_effects",
        collection(typed, mapped),
        optional
    )]
    pub particle_effects: Option<HashMap<AssetFileStem, Handle<Particle2dEffect>>>,
}

// ============================================================================
// Particle Materials
// ============================================================================

/// Resource for storing particle materials
#[derive(Resource)]
pub struct ParticleMaterials {
    pub ally_material: Handle<ColorParticle2dMaterial>,
    pub enemy_material: Handle<ColorParticle2dMaterial>,
}

impl ParticleMaterials {
    pub fn get_material_for_faction(&self, faction: &Faction) -> Handle<ColorParticle2dMaterial> {
        match faction {
            Faction::Ally => self.ally_material.clone(),
            Faction::Enemy => self.enemy_material.clone(),
        }
    }

    pub fn get_material_for_color(
        &self,
        color: &Color,
        materials: &mut Assets<ColorParticle2dMaterial>,
    ) -> Handle<ColorParticle2dMaterial> {
        if *color == Faction::Ally.get_color() {
            return self.ally_material.clone();
        }
        if *color == Faction::Enemy.get_color() {
            return self.enemy_material.clone();
        }
        materials.add(ColorParticle2dMaterial::new((*color).into()))
    }
}

// ============================================================================
// Music Assets
// ============================================================================

/// Audio assets used throughout all states of the app
#[derive(Resource, AssetCollection)]
pub struct MusicAssets {
    #[asset(key = "music", collection(typed, mapped))]
    pub music: HashMap<AssetFileStem, Handle<AudioSource>>,
}

/// Extended audio assets
#[derive(Resource, Default, AssetCollection)]
pub struct ExtendedMusicAssets {
    #[asset(key = "extended_music", collection(typed, mapped), optional)]
    pub music: Option<HashMap<AssetFileStem, Handle<AudioSource>>>,
}

// ============================================================================
// UI Assets
// ============================================================================

/// Assets for Bevy UI
#[derive(Resource, AssetCollection)]
pub struct UiAssets {
    #[asset(key = "ui_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "ui_images", collection(typed, mapped))]
    pub images: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(key = "ui_fonts", collection(typed, mapped))]
    pub fonts: HashMap<AssetFileStem, Handle<Font>>,
    #[asset(key = "ui_button_select_audio", collection(typed))]
    pub menu_button_select_effects: Vec<Handle<AudioSource>>,
    #[asset(key = "ui_button_release_audio", collection(typed))]
    pub menu_button_release_effects: Vec<Handle<AudioSource>>,
    #[asset(key = "ui_button_confirm_audio", collection(typed))]
    pub menu_button_confirm_effects: Vec<Handle<AudioSource>>,
}

/// Extended UI assets
#[derive(Resource, Default, AssetCollection)]
pub struct ExtendedUiAssets {
    #[asset(key = "extended_ui_sprites", collection(typed, mapped), optional)]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[asset(key = "extended_ui_images", collection(typed, mapped), optional)]
    pub images: Option<HashMap<AssetFileStem, Handle<Image>>>,
    #[asset(key = "extended_ui_fonts", collection(typed, mapped), optional)]
    pub fonts: Option<HashMap<AssetFileStem, Handle<Font>>>,
    #[asset(key = "extended_ui_button_select_audio", collection(typed), optional)]
    pub menu_button_select_effects: Option<Vec<Handle<AudioSource>>>,
    #[asset(key = "extended_ui_button_release_audio", collection(typed), optional)]
    pub menu_button_release_effects: Option<Vec<Handle<AudioSource>>>,
    #[asset(key = "extended_ui_button_confirm_audio", collection(typed), optional)]
    pub menu_button_confirm_effects: Option<Vec<Handle<AudioSource>>>,
}

// ============================================================================
// Background Assets
// ============================================================================

/// Assets for background images
#[derive(Resource, AssetCollection)]
pub struct BackgroundAssets {
    #[asset(key = "space_backgrounds", collection(typed))]
    pub space_backgrounds: Vec<Handle<Image>>,
    #[asset(key = "planets", collection(typed))]
    pub planets: Vec<Handle<Scene>>,
}

/// Extended background assets
#[derive(Resource, Default, AssetCollection)]
pub struct ExtendedBackgroundAssets {
    #[asset(key = "extended_space_backgrounds", collection(typed), optional)]
    pub space_backgrounds: Option<Vec<Handle<Image>>>,
    #[asset(key = "extended_planets", collection(typed), optional)]
    pub planets: Option<Vec<Handle<Scene>>>,
}

// ============================================================================
// Mod Assets (Tier 3 - User/Modder assets from mods:// source)
// ============================================================================

/// Mod game assets loaded from mods:// source
#[derive(Resource, Default, Clone, AssetCollection)]
pub struct ModGameAssets {
    #[asset(key = "mod_game_sprites", collection(typed, mapped), optional)]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[asset(key = "mod_game_particle_effects", collection(typed, mapped), optional)]
    pub particle_effects: Option<HashMap<AssetFileStem, Handle<Particle2dEffect>>>,
}

/// Mod music assets
#[derive(Resource, Default, AssetCollection)]
pub struct ModMusicAssets {
    #[asset(key = "mod_music", collection(typed, mapped), optional)]
    pub music: Option<HashMap<AssetFileStem, Handle<AudioSource>>>,
}

/// Mod UI assets
#[derive(Resource, Default, AssetCollection)]
pub struct ModUiAssets {
    #[asset(key = "mod_ui_sprites", collection(typed, mapped), optional)]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[asset(key = "mod_ui_images", collection(typed, mapped), optional)]
    pub images: Option<HashMap<AssetFileStem, Handle<Image>>>,
    #[asset(key = "mod_ui_fonts", collection(typed, mapped), optional)]
    pub fonts: Option<HashMap<AssetFileStem, Handle<Font>>>,
    #[asset(key = "mod_ui_button_select_audio", collection(typed), optional)]
    pub menu_button_select_effects: Option<Vec<Handle<AudioSource>>>,
    #[asset(key = "mod_ui_button_release_audio", collection(typed), optional)]
    pub menu_button_release_effects: Option<Vec<Handle<AudioSource>>>,
    #[asset(key = "mod_ui_button_confirm_audio", collection(typed), optional)]
    pub menu_button_confirm_effects: Option<Vec<Handle<AudioSource>>>,
}

/// Mod background assets
#[derive(Resource, Default, AssetCollection)]
pub struct ModBackgroundAssets {
    #[asset(key = "mod_space_backgrounds", collection(typed), optional)]
    pub space_backgrounds: Option<Vec<Handle<Image>>>,
    #[asset(key = "mod_planets", collection(typed), optional)]
    pub planets: Option<Vec<Handle<Scene>>>,
}

// ============================================================================
// Merged Assets (post-loading combined resources)
// ============================================================================

/// Merged game assets (all tiers combined with mods > game > base priority)
#[derive(Resource, Default)]
pub struct MergedGameAssets {
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Merged UI assets (all tiers combined with mods > game > base priority)
#[derive(Resource, Default)]
pub struct MergedUiAssets {
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    pub images: HashMap<AssetFileStem, Handle<Image>>,
    pub fonts: HashMap<AssetFileStem, Handle<Font>>,
    pub menu_button_select_effects: Vec<Handle<AudioSource>>,
    pub menu_button_release_effects: Vec<Handle<AudioSource>>,
    pub menu_button_confirm_effects: Vec<Handle<AudioSource>>,
}

/// Merged music assets (all tiers combined with mods > game > base priority)
#[derive(Resource, Default)]
pub struct MergedMusicAssets {
    pub music: HashMap<AssetFileStem, Handle<AudioSource>>,
}

/// Merged background assets (all tiers combined)
#[derive(Resource, Default)]
pub struct MergedBackgroundAssets {
    pub space_backgrounds: Vec<Handle<Image>>,
    pub planets: Vec<Handle<Scene>>,
}

// ============================================================================
// Loading Progress Event
// ============================================================================

/// Message for sending percentage of loading progress
#[derive(Message)]
pub struct LoadingProgressEvent(pub f32);

// ============================================================================
// Asset Resolver
// ============================================================================

/// Utility for resolving assets from merged resources.
///
/// Note: With `collection(typed, mapped)`, assets are keyed by their file stem.
/// For example, "media/aseprite/bullet_projectile.aseprite" gets key "bullet_projectile".
pub struct AssetResolver;

impl AssetResolver {
    /// Get an Aseprite handle by key
    pub fn get_game_sprite(
        key: &str,
        assets: &MergedGameAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        assets
            .sprites
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    /// Get a Particle2DEffect handle by key
    pub fn get_game_particle_effect(
        key: &str,
        assets: &MergedGameAssets,
    ) -> Result<Handle<Particle2dEffect>, AssetError> {
        assets
            .particle_effects
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_image(
        key: &str,
        assets: &MergedUiAssets,
    ) -> Result<Handle<Image>, AssetError> {
        assets
            .images
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_sprite(
        key: &str,
        assets: &MergedUiAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        assets
            .sprites
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_font(
        key: &str,
        assets: &MergedUiAssets,
    ) -> Result<Handle<Font>, AssetError> {
        assets
            .fonts
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_random_button_press_effect(
        assets: &MergedUiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if assets.menu_button_select_effects.is_empty() {
            return Err(AssetError::EmptyCollections(
                "button press effects".to_string(),
            ));
        }
        let idx = rand::rng().random_range(0..assets.menu_button_select_effects.len());
        Ok(assets.menu_button_select_effects[idx].clone())
    }

    pub fn get_random_button_release_effect(
        assets: &MergedUiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if assets.menu_button_release_effects.is_empty() {
            return Err(AssetError::EmptyCollections(
                "button release effects".to_string(),
            ));
        }
        let idx = rand::rng().random_range(0..assets.menu_button_release_effects.len());
        Ok(assets.menu_button_release_effects[idx].clone())
    }

    pub fn get_random_button_confirm_effect(
        assets: &MergedUiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if assets.menu_button_confirm_effects.is_empty() {
            return Err(AssetError::EmptyCollections(
                "button confirm effects".to_string(),
            ));
        }
        let idx = rand::rng().random_range(0..assets.menu_button_confirm_effects.len());
        Ok(assets.menu_button_confirm_effects[idx].clone())
    }

    pub fn get_random_space_bg(
        assets: &MergedBackgroundAssets,
    ) -> Result<Handle<Image>, AssetError> {
        if assets.space_backgrounds.is_empty() {
            return Err(AssetError::EmptyCollections(
                "space backgrounds".to_string(),
            ));
        }
        let idx = rand::rng().random_range(0..assets.space_backgrounds.len());
        Ok(assets.space_backgrounds[idx].clone())
    }

    pub fn get_random_planet(
        assets: &MergedBackgroundAssets,
    ) -> Result<Handle<Scene>, AssetError> {
        if assets.planets.is_empty() {
            return Err(AssetError::EmptyCollections("planets".to_string()));
        }
        let idx = rand::rng().random_range(0..assets.planets.len());
        Ok(assets.planets[idx].clone())
    }

    pub fn get_music(
        key: &str,
        assets: &MergedMusicAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        assets
            .music
            .get(key)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }
}
