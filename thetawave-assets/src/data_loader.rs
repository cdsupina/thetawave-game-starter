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
    
    // Try to load extended data from assets/data/ relative to the binary's location
    // This works for all scenarios: development, library usage, and release builds
    let extended_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("assets/data").join(extended_filename)
        } else {
            // Fallback to current working directory if parent() fails
            Path::new("assets/data").join(extended_filename)
        }
    } else {
        // Fallback to current working directory if current_exe() fails
        Path::new("assets/data").join(extended_filename)
    };
    
    log::info!("Looking for extended data at: {:?}", extended_path);
    
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