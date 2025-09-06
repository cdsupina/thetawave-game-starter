use serde::de::DeserializeOwned;
use std::path::Path;
use toml::Value;

/// Recursively merge TOML values, with extended values taking precedence over base values.
/// This handles tables, arrays, and primitive values correctly.
fn merge_toml_values(base: &mut Value, extended: Value) {
    match (base, extended) {
        // Both are tables - merge recursively
        (Value::Table(base_table), Value::Table(extended_table)) => {
            for (key, extended_value) in extended_table {
                match base_table.get_mut(&key) {
                    Some(base_value) => {
                        // Key exists in base - merge recursively
                        merge_toml_values(base_value, extended_value);
                    }
                    None => {
                        // Key doesn't exist in base - add it
                        base_table.insert(key, extended_value);
                    }
                }
            }
        }
        // For non-table values, extended completely replaces base
        (base, extended) => {
            *base = extended;
        }
    }
}

/// Load a resource with optional extended data field-level merging
/// Extended data is looked for relative to the working directory: "assets/data/{filename}"
/// This aligns with how media assets work via the "extended" asset source.
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
    
    // Look for extended data in assets/data/ relative to current working directory
    // This matches how the "extended" asset source works for media files
    let extended_path = Path::new("assets/data").join(extended_filename);
    
    log::info!("Looking for extended data at: {:?}", extended_path);
    
    if extended_path.exists() {
        if let Ok(extended_bytes) = std::fs::read(&extended_path) {
            if let Ok(extended_value) = toml::from_slice::<Value>(&extended_bytes) {
                // Perform careful TOML merging that preserves arrays correctly
                merge_toml_values(&mut base_value, extended_value);
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