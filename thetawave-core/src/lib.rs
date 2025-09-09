mod data;
mod data_loader;
mod plugin;

pub use data::{CollisionDamage, Faction, HealthComponent};
pub use data_loader::load_with_extended;
pub use plugin::ThetawaveCorePlugin;
