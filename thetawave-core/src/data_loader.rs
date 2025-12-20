use bevy::log::info;
use serde::de::DeserializeOwned;
use toml::Value;

/// Recursively merge TOML values, with extended values taking precedence over base values.
/// This handles tables, arrays, and primitive values correctly.
pub fn merge_toml_values(base: &mut Value, extended: Value) {
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
pub fn load_with_extended<T>(base_bytes: &[u8], extended_filename: &str) -> T
where
    T: DeserializeOwned,
{
    // Parse base embedded data as TOML value for merging
    let mut base_value: Value =
        toml::from_slice(base_bytes).expect("Failed to parse base data as TOML");

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::path::Path;

        // Native: Use filesystem access
        // Check multiple locations for extended data files:
        // 1. assets/data/ - works from game crate directory or shared workspace
        // 2. thetawave-test-game/assets/data/ - when running from workspace root
        // Later paths take precedence over earlier ones

        let paths_to_check = [
            Path::new("assets/data").join(extended_filename),
            Path::new("thetawave-test-game/assets/data").join(extended_filename),
        ];

        for path in &paths_to_check {
            if path.exists()
                && let Ok(extended_bytes) = std::fs::read(path)
                && let Ok(extended_value) = toml::from_slice::<Value>(&extended_bytes)
            {
                merge_toml_values(&mut base_value, extended_value);
                info!("Loaded and merged extended data from: {:?}", path);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: Use HTTP request to fetch extended data
        use web_sys::XmlHttpRequest;

        // Helper to try loading from a URL
        let try_load_url = |url: &str| -> Option<Value> {
            let xhr = XmlHttpRequest::new().ok()?;
            xhr.open_with_async("GET", url, false).ok()?;
            xhr.send().ok()?;
            if xhr.status().ok()? == 200 {
                let text = xhr.response_text().ok()??;
                toml::from_str::<Value>(&text).ok()
            } else {
                None
            }
        };

        // WASM serves from a single assets folder
        let url = format!("assets/data/{}", extended_filename);
        if let Some(extended_value) = try_load_url(&url) {
            merge_toml_values(&mut base_value, extended_value);
            info!("Loaded and merged extended data from: {}", url);
        }
    }

    // Deserialize the merged TOML value to the target type
    base_value
        .try_into()
        .expect("Failed to deserialize merged TOML data")
}
