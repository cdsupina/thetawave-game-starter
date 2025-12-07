//! Asset data structures for Thetawave.
//!
//! When `asset_loader` feature is enabled, uses bevy_asset_loader's AssetCollection derive.
//! When disabled, same struct fields but populated manually.

use bevy::{
    asset::{Assets, Handle},
    color::Color,
    image::Image,
    platform::collections::HashMap,
    prelude::Resource,
    scene::Scene,
    text::Font,
};
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_enoki::{Particle2dEffect, prelude::ColorParticle2dMaterial};
use bevy_kira_audio::AudioSource;
use rand::Rng;
use thetawave_core::Faction;

use crate::AssetError;

// ============================================================================
// Feature-gated imports and type aliases
// ============================================================================

#[cfg(feature = "asset_loader")]
use bevy::prelude::Message;

#[cfg(feature = "asset_loader")]
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileStem};

/// When asset_loader is disabled, use String as the key type
#[cfg(not(feature = "asset_loader"))]
pub type AssetFileStem = String;

// ============================================================================
// Game Assets
// ============================================================================

/// Assets used in the game state
#[derive(Resource)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct GameAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "game_sprites", collection(typed, mapped)))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "game_particle_effects", collection(typed, mapped)))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

#[cfg(not(feature = "asset_loader"))]
impl Default for GameAssets {
    fn default() -> Self {
        Self {
            sprites: HashMap::default(),
            particle_effects: HashMap::default(),
        }
    }
}

/// Additional assets used in the game state that are not built in to thetawave-assets
#[derive(Resource, Default, Clone)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct ExtendedGameAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_game_sprites", collection(typed, mapped), optional))]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_game_particle_effects", collection(typed, mapped), optional))]
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
#[derive(Resource)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct MusicAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "music", collection(typed, mapped)))]
    pub music: HashMap<AssetFileStem, Handle<AudioSource>>,
}

#[cfg(not(feature = "asset_loader"))]
impl Default for MusicAssets {
    fn default() -> Self {
        Self {
            music: HashMap::default(),
        }
    }
}

/// Extended audio assets
#[derive(Resource, Default)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct ExtendedMusicAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_music", collection(typed, mapped), optional))]
    pub music: Option<HashMap<AssetFileStem, Handle<AudioSource>>>,
}

// ============================================================================
// UI Assets
// ============================================================================

/// Assets for Bevy UI
#[derive(Resource)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct UiAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_sprites", collection(typed, mapped)))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_images", collection(typed, mapped)))]
    pub images: HashMap<AssetFileStem, Handle<Image>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_fonts", collection(typed, mapped)))]
    pub fonts: HashMap<AssetFileStem, Handle<Font>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_button_select_audio", collection(typed)))]
    pub menu_button_select_effects: Vec<Handle<AudioSource>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_button_release_audio", collection(typed)))]
    pub menu_button_release_effects: Vec<Handle<AudioSource>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "ui_button_confirm_audio", collection(typed)))]
    pub menu_button_confirm_effects: Vec<Handle<AudioSource>>,
}

#[cfg(not(feature = "asset_loader"))]
impl Default for UiAssets {
    fn default() -> Self {
        Self {
            sprites: HashMap::default(),
            images: HashMap::default(),
            fonts: HashMap::default(),
            menu_button_select_effects: Vec::default(),
            menu_button_release_effects: Vec::default(),
            menu_button_confirm_effects: Vec::default(),
        }
    }
}

/// Extended UI assets
#[derive(Resource, Default)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct ExtendedUiAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_sprites", collection(typed, mapped), optional))]
    pub sprites: Option<HashMap<AssetFileStem, Handle<Aseprite>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_images", collection(typed, mapped), optional))]
    pub images: Option<HashMap<AssetFileStem, Handle<Image>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_fonts", collection(typed, mapped), optional))]
    pub fonts: Option<HashMap<AssetFileStem, Handle<Font>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_button_select_audio", collection(typed), optional))]
    pub menu_button_select_effects: Option<Vec<Handle<AudioSource>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_button_release_audio", collection(typed), optional))]
    pub menu_button_release_effects: Option<Vec<Handle<AudioSource>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_ui_button_confirm_audio", collection(typed), optional))]
    pub menu_button_confirm_effects: Option<Vec<Handle<AudioSource>>>,
}

// ============================================================================
// Background Assets
// ============================================================================

/// Assets for background images
#[derive(Resource)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct BackgroundAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "space_backgrounds", collection(typed)))]
    pub space_backgrounds: Vec<Handle<Image>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "planets", collection(typed)))]
    pub planets: Vec<Handle<Scene>>,
}

#[cfg(not(feature = "asset_loader"))]
impl Default for BackgroundAssets {
    fn default() -> Self {
        Self {
            space_backgrounds: Vec::default(),
            planets: Vec::default(),
        }
    }
}

/// Extended background assets
#[derive(Resource, Default)]
#[cfg_attr(feature = "asset_loader", derive(AssetCollection))]
pub struct ExtendedBackgroundAssets {
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_space_backgrounds", collection(typed), optional))]
    pub space_backgrounds: Option<Vec<Handle<Image>>>,
    #[cfg_attr(feature = "asset_loader", asset(key = "extended_planets", collection(typed), optional))]
    pub planets: Option<Vec<Handle<Scene>>>,
}

// ============================================================================
// Loading Progress Event (only with asset_loader feature)
// ============================================================================

/// Message for sending percentage of loading progress
#[cfg(feature = "asset_loader")]
#[derive(Message)]
pub struct LoadingProgressEvent(pub f32);

// ============================================================================
// Asset Resolver
// ============================================================================

/// Utility for resolving assets with Extended*Assets priority and base assets fallback
///
/// Note: With bevy_asset_loader's `Files(paths: [...])` format and `collection(typed, mapped)`,
/// assets are keyed by their full file path as specified in the paths array.
/// For example, "media/aseprite/bullet_projectile.aseprite" gets key "media/aseprite/bullet_projectile.aseprite".
pub struct AssetResolver;

impl AssetResolver {
    /// Get an Aseprite handle by key, checking ExtendedGameAssets first, then GameAssets
    pub fn get_game_sprite(
        key: &str,
        extended_assets: &ExtendedGameAssets,
        assets: &GameAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        extended_assets
            .sprites
            .as_ref()
            .and_then(|sprites| sprites.get(key))
            .or_else(|| assets.sprites.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    /// Get a Particle2DEffect handle by key, checking ExtendedGameAssets first, then GameAssets
    pub fn get_game_particle_effect(
        key: &str,
        extended_assets: &ExtendedGameAssets,
        assets: &GameAssets,
    ) -> Result<Handle<Particle2dEffect>, AssetError> {
        extended_assets
            .particle_effects
            .as_ref()
            .and_then(|effects| effects.get(key))
            .or_else(|| assets.particle_effects.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_image(
        key: &str,
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<Image>, AssetError> {
        extended_assets
            .images
            .as_ref()
            .and_then(|images| images.get(key))
            .or_else(|| assets.images.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_sprite(
        key: &str,
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        extended_assets
            .sprites
            .as_ref()
            .and_then(|sprites| sprites.get(key))
            .or_else(|| assets.sprites.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_font(
        key: &str,
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<Font>, AssetError> {
        extended_assets
            .fonts
            .as_ref()
            .and_then(|fonts| fonts.get(key))
            .or_else(|| assets.fonts.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_random_button_press_effect(
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if let Some(effects) = &extended_assets.menu_button_select_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        if !assets.menu_button_select_effects.is_empty() {
            let idx = rand::rng().random_range(0..assets.menu_button_select_effects.len());
            Ok(assets.menu_button_select_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button press effects".to_string(),
            ))
        }
    }

    pub fn get_random_button_release_effect(
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if let Some(effects) = &extended_assets.menu_button_release_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        if !assets.menu_button_release_effects.is_empty() {
            let idx = rand::rng().random_range(0..assets.menu_button_release_effects.len());
            Ok(assets.menu_button_release_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button release effects".to_string(),
            ))
        }
    }

    pub fn get_random_button_confirm_effect(
        extended_assets: &ExtendedUiAssets,
        assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        if let Some(effects) = &extended_assets.menu_button_confirm_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        if !assets.menu_button_confirm_effects.is_empty() {
            let idx = rand::rng().random_range(0..assets.menu_button_confirm_effects.len());
            Ok(assets.menu_button_confirm_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button confirm effects".to_string(),
            ))
        }
    }

    pub fn get_random_space_bg(
        extended_assets: &ExtendedBackgroundAssets,
        assets: &BackgroundAssets,
    ) -> Result<Handle<Image>, AssetError> {
        let mut all_backgrounds = Vec::new();
        all_backgrounds.extend(assets.space_backgrounds.iter().cloned());
        if let Some(extended_backgrounds) = &extended_assets.space_backgrounds {
            all_backgrounds.extend(extended_backgrounds.iter().cloned());
        }
        if all_backgrounds.is_empty() {
            return Err(AssetError::EmptyCollections(
                "space backgrounds".to_string(),
            ));
        }
        let idx = rand::rng().random_range(0..all_backgrounds.len());
        Ok(all_backgrounds[idx].clone())
    }

    pub fn get_random_planet(
        extended_assets: &ExtendedBackgroundAssets,
        assets: &BackgroundAssets,
    ) -> Result<Handle<Scene>, AssetError> {
        let mut all_planets = Vec::new();
        all_planets.extend(assets.planets.iter().cloned());
        if let Some(extended_planets) = &extended_assets.planets {
            all_planets.extend(extended_planets.iter().cloned());
        }
        if all_planets.is_empty() {
            return Err(AssetError::EmptyCollections("planets".to_string()));
        }
        let idx = rand::rng().random_range(0..all_planets.len());
        Ok(all_planets[idx].clone())
    }

    pub fn get_music(
        key: &str,
        extended_assets: &ExtendedMusicAssets,
        assets: &MusicAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        extended_assets
            .music
            .as_ref()
            .and_then(|music| music.get(key))
            .or_else(|| assets.music.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }
}
