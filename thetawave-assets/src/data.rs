use bevy::{
    asset::Handle,
    image::Image,
    platform::collections::HashMap,
    prelude::{Event, Res, Resource},
    scene::Scene,
    text::Font,
};
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileStem};
use bevy_enoki::{Particle2dEffect, prelude::ColorParticle2dMaterial};
use bevy_kira_audio::AudioSource;
use rand::Rng;
use thetawave_core::Faction;

/// Assets used in the game state
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "game_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "game_particle_effects", collection(typed, mapped))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Additional assets used in the game state that are not built in to thetawave-assets
#[derive(AssetCollection, Resource, Default, Clone)]
pub struct ExtendedGameAssets {
    #[asset(key = "extended_game_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "extended_game_particle_effects", collection(typed, mapped))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Resource for storing faction-based particle materials
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
}

/// Utility for resolving assets with ExtendedGameAssets priority and GameAssets fallback
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
        game_assets: &GameAssets,
    ) -> Handle<Aseprite> {
        extended_assets
            .sprites
            .get(key)
            .or_else(|| game_assets.sprites.get(key))
            .cloned()
            .unwrap_or_else(|| panic!("Missing sprite asset for key: {:?}", key))
    }

    /// Get an Particle2DEffect handle by key, checking ExtendedGameAssets first, then GameAssets
    pub fn get_game_particle_effect(
        key: &str,
        extended_assets: &ExtendedGameAssets,
        game_assets: &GameAssets,
    ) -> Handle<Particle2dEffect> {
        extended_assets
            .particle_effects
            .get(key)
            .or_else(|| game_assets.particle_effects.get(key))
            .cloned()
            .unwrap_or_else(|| panic!("Missing particle asset for key: {:?}", key))
    }

    pub fn get_ui_image(key: &str, ui_assets: &UiAssets) -> Handle<Image> {
        ui_assets
            .images
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing image asset for key: {:?}", key))
    }

    pub fn get_ui_sprite(key: &str, ui_assets: &UiAssets) -> Handle<Aseprite> {
        ui_assets
            .sprites
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing sprite asset for key: {:?}", key))
    }

    pub fn get_ui_font(key: &str, ui_assets: &UiAssets) -> Handle<Font> {
        ui_assets
            .fonts
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing font asset for key: {:?}", key))
    }
}

/// Audio assets used throughout all states of the app
#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(key = "main_menu_theme")]
    pub main_menu_theme: Handle<AudioSource>,
    #[asset(key = "game_theme")]
    pub game_theme: Handle<AudioSource>,
}

// Assets for Bevy ui
#[derive(AssetCollection, Resource)]
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

impl UiAssets {
    pub fn get_random_button_press_effect(&self) -> Handle<AudioSource> {
        self.menu_button_select_effects
            [rand::rng().random_range(0..self.menu_button_select_effects.len())]
        .clone()
    }

    pub fn get_random_button_release_effect(&self) -> Handle<AudioSource> {
        self.menu_button_release_effects
            [rand::rng().random_range(0..self.menu_button_release_effects.len())]
        .clone()
    }

    pub fn get_random_button_confirm_effect(&self) -> Handle<AudioSource> {
        self.menu_button_confirm_effects
            [rand::rng().random_range(0..self.menu_button_confirm_effects.len())]
        .clone()
    }
}

// Assets for background images
#[derive(AssetCollection, Resource)]
pub struct BackgroundAssets {
    // all space backgrounds
    #[asset(key = "space_backgrounds", collection(typed))]
    pub space_backgrounds: Vec<Handle<Image>>,
    // all planets
    #[asset(key = "planets", collection(typed))]
    pub planets: Vec<Handle<Scene>>,
}

impl BackgroundAssets {
    pub fn get_random_space_bg(&self) -> Handle<Image> {
        self.space_backgrounds[rand::rng().random_range(0..self.space_backgrounds.len())].clone()
    }

    pub fn get_random_planet(&self) -> Handle<Scene> {
        self.planets[rand::rng().random_range(0..self.planets.len())].clone()
    }
}

/// Event for sending percentage of loading progress
#[derive(Event)]
pub struct LoadingProgressEvent(pub f32);
