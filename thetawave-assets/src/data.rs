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
// Loading Progress Event
// ============================================================================

/// Message for sending percentage of loading progress
#[derive(Message)]
pub struct LoadingProgressEvent(pub f32);

// ============================================================================
// Asset Resolver
// ============================================================================

/// Utility for resolving assets with 3-tier priority: Mods -> Game -> Base
///
/// Note: With `collection(typed, mapped)`, assets are keyed by their file stem.
/// For example, "media/aseprite/bullet_projectile.aseprite" gets key "bullet_projectile".
pub struct AssetResolver;

impl AssetResolver {
    /// Get an Aseprite handle by key, checking Mods first, then Game, then Base
    pub fn get_game_sprite(
        key: &str,
        mod_assets: &ModGameAssets,
        game_assets: &ExtendedGameAssets,
        base_assets: &GameAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        mod_assets
            .sprites
            .as_ref()
            .and_then(|sprites| sprites.get(key))
            .or_else(|| {
                game_assets
                    .sprites
                    .as_ref()
                    .and_then(|sprites| sprites.get(key))
            })
            .or_else(|| base_assets.sprites.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    /// Get a Particle2DEffect handle by key, checking Mods first, then Game, then Base
    pub fn get_game_particle_effect(
        key: &str,
        mod_assets: &ModGameAssets,
        game_assets: &ExtendedGameAssets,
        base_assets: &GameAssets,
    ) -> Result<Handle<Particle2dEffect>, AssetError> {
        mod_assets
            .particle_effects
            .as_ref()
            .and_then(|effects| effects.get(key))
            .or_else(|| {
                game_assets
                    .particle_effects
                    .as_ref()
                    .and_then(|effects| effects.get(key))
            })
            .or_else(|| base_assets.particle_effects.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_image(
        key: &str,
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<Image>, AssetError> {
        mod_assets
            .images
            .as_ref()
            .and_then(|images| images.get(key))
            .or_else(|| game_assets.images.as_ref().and_then(|images| images.get(key)))
            .or_else(|| base_assets.images.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_sprite(
        key: &str,
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<Aseprite>, AssetError> {
        mod_assets
            .sprites
            .as_ref()
            .and_then(|sprites| sprites.get(key))
            .or_else(|| {
                game_assets
                    .sprites
                    .as_ref()
                    .and_then(|sprites| sprites.get(key))
            })
            .or_else(|| base_assets.sprites.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_ui_font(
        key: &str,
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<Font>, AssetError> {
        mod_assets
            .fonts
            .as_ref()
            .and_then(|fonts| fonts.get(key))
            .or_else(|| game_assets.fonts.as_ref().and_then(|fonts| fonts.get(key)))
            .or_else(|| base_assets.fonts.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }

    pub fn get_random_button_press_effect(
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        // Check mods first
        if let Some(effects) = &mod_assets.menu_button_select_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Then game assets
        if let Some(effects) = &game_assets.menu_button_select_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Finally base assets
        if !base_assets.menu_button_select_effects.is_empty() {
            let idx = rand::rng().random_range(0..base_assets.menu_button_select_effects.len());
            Ok(base_assets.menu_button_select_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button press effects".to_string(),
            ))
        }
    }

    pub fn get_random_button_release_effect(
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        // Check mods first
        if let Some(effects) = &mod_assets.menu_button_release_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Then game assets
        if let Some(effects) = &game_assets.menu_button_release_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Finally base assets
        if !base_assets.menu_button_release_effects.is_empty() {
            let idx = rand::rng().random_range(0..base_assets.menu_button_release_effects.len());
            Ok(base_assets.menu_button_release_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button release effects".to_string(),
            ))
        }
    }

    pub fn get_random_button_confirm_effect(
        mod_assets: &ModUiAssets,
        game_assets: &ExtendedUiAssets,
        base_assets: &UiAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        // Check mods first
        if let Some(effects) = &mod_assets.menu_button_confirm_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Then game assets
        if let Some(effects) = &game_assets.menu_button_confirm_effects
            && !effects.is_empty()
        {
            let idx = rand::rng().random_range(0..effects.len());
            return Ok(effects[idx].clone());
        }
        // Finally base assets
        if !base_assets.menu_button_confirm_effects.is_empty() {
            let idx = rand::rng().random_range(0..base_assets.menu_button_confirm_effects.len());
            Ok(base_assets.menu_button_confirm_effects[idx].clone())
        } else {
            Err(AssetError::EmptyCollections(
                "button confirm effects".to_string(),
            ))
        }
    }

    pub fn get_random_space_bg(
        mod_assets: &ModBackgroundAssets,
        game_assets: &ExtendedBackgroundAssets,
        base_assets: &BackgroundAssets,
    ) -> Result<Handle<Image>, AssetError> {
        let mut all_backgrounds = Vec::new();
        all_backgrounds.extend(base_assets.space_backgrounds.iter().cloned());
        if let Some(game_backgrounds) = &game_assets.space_backgrounds {
            all_backgrounds.extend(game_backgrounds.iter().cloned());
        }
        if let Some(mod_backgrounds) = &mod_assets.space_backgrounds {
            all_backgrounds.extend(mod_backgrounds.iter().cloned());
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
        mod_assets: &ModBackgroundAssets,
        game_assets: &ExtendedBackgroundAssets,
        base_assets: &BackgroundAssets,
    ) -> Result<Handle<Scene>, AssetError> {
        let mut all_planets = Vec::new();
        all_planets.extend(base_assets.planets.iter().cloned());
        if let Some(game_planets) = &game_assets.planets {
            all_planets.extend(game_planets.iter().cloned());
        }
        if let Some(mod_planets) = &mod_assets.planets {
            all_planets.extend(mod_planets.iter().cloned());
        }
        if all_planets.is_empty() {
            return Err(AssetError::EmptyCollections("planets".to_string()));
        }
        let idx = rand::rng().random_range(0..all_planets.len());
        Ok(all_planets[idx].clone())
    }

    pub fn get_music(
        key: &str,
        mod_assets: &ModMusicAssets,
        game_assets: &ExtendedMusicAssets,
        base_assets: &MusicAssets,
    ) -> Result<Handle<AudioSource>, AssetError> {
        mod_assets
            .music
            .as_ref()
            .and_then(|music| music.get(key))
            .or_else(|| game_assets.music.as_ref().and_then(|music| music.get(key)))
            .or_else(|| base_assets.music.get(key))
            .cloned()
            .ok_or_else(|| AssetError::NotFound(key.to_string()))
    }
}
