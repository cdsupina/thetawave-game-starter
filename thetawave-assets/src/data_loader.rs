use serde::de::DeserializeOwned;
use std::path::Path;
use toml::Value;

/// Load a resource with optional extended data field-level merging
/// Extended data is looked for relative to the binary: "assets/data/{filename}"
/// 
/// Field-level merging strategy:
/// - If entry doesn't exist in base: add the entire entry from extended
/// - If entry exists in base: merge at field level, extended fields override base fields
/// - Fields not specified in extended retain their base values
/// 
/// # Arguments
/// * `base_bytes` - The embedded base data as bytes  
/// * `extended_filename` - The filename to look for in assets/data/
/// 
/// # Returns
/// The merged resource with base data + field-level extended overrides/additions
pub fn load_with_extended<T>(
    base_bytes: &[u8],
    extended_filename: &str,
) -> T 
where
    T: DeserializeOwned
{
    // Parse base embedded data as TOML value for merging
    let mut base_value: Value = toml::from_slice(base_bytes)
        .expect("Failed to parse base data as TOML");
    
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
            if let Ok(extended_value) = toml::from_slice::<Value>(&extended_bytes) {
                // Perform field-level TOML merging
                base_value = serde_toml_merge::merge(base_value, extended_value)
                    .expect("Failed to merge TOML values");
                log::info!("Loaded and merged extended data from: {:?}", extended_path);
            } else {
                log::warn!("Failed to parse extended data as TOML from: {:?}", extended_path);
            }
        } else {
            log::warn!("Failed to read extended data file: {:?}", extended_path);
        }
    }
    
    // Deserialize the merged TOML value to the target type
    base_value.try_into()
        .expect("Failed to deserialize merged TOML data")
}