use bevy::{
    asset::Handle,
    image::Image,
    prelude::{Res, Resource},
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_hui::prelude::HtmlTemplate;
use rand::Rng;

// Main menu assets including HTML templates for various UI components
#[derive(AssetCollection, Resource)]
pub(crate) struct MainMenuAssets {
    // HTML template for main menu layout
    #[asset(path = "ui/menus/main_menu.html")]
    pub main_menu_html: Handle<HtmlTemplate>,
    // HTML template for menu buttons
    #[asset(path = "ui/components/menu_button.html")]
    pub menu_button_html: Handle<HtmlTemplate>,
    // HTML template for website footer button
    #[asset(path = "ui/components/website_footer_button.html")]
    pub website_footer_button_html: Handle<HtmlTemplate>,
}

// Assets for background images
#[derive(AssetCollection, Resource)]
pub(crate) struct BackgroundAssets {
    // all space backgrounds
    #[asset(path = "media/images/backgrounds", collection(typed))]
    pub space_backgrounds: Vec<Handle<Image>>,
}

impl BackgroundAssets {
    pub(crate) fn get_random_space_bg(&self) -> Handle<Image> {
        self.space_backgrounds[rand::thread_rng().gen_range(0..self.space_backgrounds.len())]
            .clone()
    }
}
