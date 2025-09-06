use serde::de::DeserializeOwned;
use std::path::Path;

/// Trait for resources that can be merged with extended data
/// Extended data takes priority and can add new entries
pub trait MergeableResource: DeserializeOwned {
    /// Merge another resource of the same type into this one
    /// Extended data should override existing entries and add new ones
    fn merge(&mut self, other: Self);
}

/// Load a resource with optional extended data override/merge
/// Extended data is looked for relative to the binary: "assets/data/{filename}"
/// 
/// # Arguments
/// * `base_bytes` - The embedded base data as bytes
/// * `extended_filename` - The filename to look for in assets/data/
/// 
/// # Returns
/// The merged resource with base data + any extended overrides/additions
pub fn load_with_extended<T>(
    base_bytes: &[u8],
    extended_filename: &str,
) -> T 
where
    T: DeserializeOwned + MergeableResource
{
    // Parse base embedded data
    let mut base = toml::from_slice::<T>(base_bytes)
        .expect("Failed to parse base data");
    
    // Try to load extended data from assets/data/ relative to working directory
    // Check if we're in development workspace (has Cargo.toml) vs runtime directory
    let in_development = Path::new("Cargo.toml").exists();
    
    let extended_path = if in_development {
        // We're in development workspace - look in test-game subdirectory
        Path::new("thetawave-test-game/assets/data").join(extended_filename)
    } else {
        // We're in runtime directory - look in local assets
        Path::new("assets/data").join(extended_filename)
    };
    
    log::info!("In development: {}, Looking for extended data at: {:?}", in_development, extended_path);
    
    if extended_path.exists() {
        if let Ok(extended_bytes) = std::fs::read(&extended_path) {
            if let Ok(extended) = toml::from_slice::<T>(&extended_bytes) {
                base.merge(extended);
                log::info!("Loaded extended data from: {:?}", extended_path);
            } else {
                log::warn!("Failed to parse extended data from: {:?}", extended_path);
            }
        } else {
            log::warn!("Failed to read extended data file: {:?}", extended_path);
        }
    }
    
    base
}