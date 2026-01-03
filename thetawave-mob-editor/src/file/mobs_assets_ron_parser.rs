//! Parser for `mobs.assets.ron` mob registration files.
//!
//! Handles reading and appending mob/patch entries to the RON-format
//! asset registration files used by bevy_asset_loader.

use std::fs;
use std::path::PathBuf;

/// Parsed content from a mobs.assets.ron file
#[derive(Debug, Default)]
pub struct ParsedMobsAssets {
    /// Mob paths from "mobs" or "extended_mobs" sections
    pub mobs: Vec<String>,
    /// Patch paths from "extended_mob_patches" section
    pub patches: Vec<String>,
}

/// Parse mob and patch paths from a mobs.assets.ron file
///
/// Returns paths without the extended:// prefix (normalized).
///
/// # Format
///
/// Base mobs.assets.ron:
/// ```ron
/// ({
///     "mobs": Files(
///         paths: [
///             "mobs/xhitara/grunt.mob",
///         ]
///     ),
/// })
/// ```
///
/// Extended mobs.assets.ron:
/// ```ron
/// ({
///     "extended_mobs": Files(
///         paths: [
///             "extended://mobs/custom/enemy.mob",
///         ]
///     ),
///     "extended_mob_patches": Files(
///         paths: [
///             "extended://mobs/xhitara/grunt.mobpatch",
///         ]
///     ),
/// })
/// ```
pub fn parse_mobs_assets_ron(path: &PathBuf) -> Result<ParsedMobsAssets, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut result = ParsedMobsAssets::default();

    #[derive(Clone, Copy, PartialEq)]
    enum Section {
        None,
        Mobs,         // "mobs" or "extended_mobs"
        MobPatches,   // "extended_mob_patches"
    }

    let mut current_section = Section::None;
    let mut in_paths_array = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Check for section starts (before tracking braces)
        if trimmed.contains("\"mobs\"") || trimmed.contains("\"extended_mobs\"") {
            current_section = Section::Mobs;
            in_paths_array = false;
            brace_depth = 0;
            continue;
        }
        if trimmed.contains("\"extended_mob_patches\"") {
            current_section = Section::MobPatches;
            in_paths_array = false;
            brace_depth = 0;
            continue;
        }

        if current_section != Section::None {
            // Track bracket depth
            for ch in trimmed.chars() {
                match ch {
                    '(' | '[' => brace_depth += 1,
                    ')' | ']' => brace_depth -= 1,
                    _ => {}
                }
            }

            // Check for paths array
            if trimmed.contains("paths:") {
                in_paths_array = true;
            }

            // Extract quoted strings ending in .mob or .mobpatch
            if in_paths_array {
                if let Some(path_str) = extract_quoted_path(trimmed) {
                    if path_str.ends_with(".mob") || path_str.ends_with(".mobpatch") {
                        // Strip extended:// prefix if present
                        let clean_path = path_str
                            .strip_prefix("extended://")
                            .unwrap_or(&path_str)
                            .to_string();

                        match current_section {
                            Section::Mobs => result.mobs.push(clean_path),
                            Section::MobPatches => result.patches.push(clean_path),
                            Section::None => {}
                        }
                    }
                }
            }

            // End of section when we close back to depth 0
            if brace_depth <= 0 && in_paths_array {
                current_section = Section::None;
                in_paths_array = false;
            }
        }
    }

    Ok(result)
}

/// Extract a quoted path from a line like `"mobs/xhitara/grunt.mob",`
fn extract_quoted_path(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Create a template extended mobs.assets.ron file
fn create_extended_mobs_assets_ron_template() -> String {
    r#"// Extended mobs - add your custom mob definitions here
({
    // Extended complete mobs - these add new mobs to the game
    // Use .mob extension for complete mob definitions
    "extended_mobs": Files(
        paths: [
        ]
    ),
    // Extended mob patches - these merge with base mobs
    // Use .mobpatch extension for partial field overrides
    "extended_mob_patches": Files(
        paths: [
        ]
    ),
})
"#
    .to_string()
}

/// Append a mob or patch path to a mobs.assets.ron file
///
/// For extended files, the path will be prefixed with "extended://".
/// For base files, only mobs can be added (not patches).
///
/// Creates the file with a template if it doesn't exist (extended only).
pub fn append_to_mobs_assets_ron(
    path: &PathBuf,
    mob_path: &str,
    is_patch: bool,
    is_extended: bool,
) -> Result<(), String> {
    // Base mobs.assets.ron should never be created by the editor
    if !is_extended && !path.exists() {
        return Err(format!(
            "Base mobs.assets.ron does not exist at {}. Base mobs should be added to the game's asset files.",
            path.display()
        ));
    }

    // Create extended file if it doesn't exist
    if is_extended && !path.exists() {
        let template = create_extended_mobs_assets_ron_template();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }
        fs::write(path, &template)
            .map_err(|e| format!("Failed to create {}: {}", path.display(), e))?;
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Determine which section to add to
    let section_name = if is_extended {
        if is_patch {
            "\"extended_mob_patches\""
        } else {
            "\"extended_mobs\""
        }
    } else {
        "\"mobs\""
    };

    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut in_section = false;
    let mut in_paths = false;
    let mut insert_index = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.contains(section_name) {
            in_section = true;
            continue;
        }

        if in_section && trimmed.contains("paths:") {
            in_paths = true;
            continue;
        }

        // Find the closing bracket of the paths array
        if in_paths && trimmed.starts_with(']') {
            insert_index = Some(i);
            break;
        }
    }

    if let Some(idx) = insert_index {
        // Format the new line with proper indentation
        let new_line = if is_extended {
            format!("            \"extended://{}\",", mob_path)
        } else {
            format!("            \"{}\",", mob_path)
        };
        lines.insert(idx, new_line);

        let new_content = lines.join("\n");
        fs::write(path, new_content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        Ok(())
    } else {
        Err(format!(
            "Could not find {} section in {}",
            section_name,
            path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quoted_path() {
        assert_eq!(
            extract_quoted_path(r#"            "mobs/xhitara/grunt.mob","#),
            Some("mobs/xhitara/grunt.mob".to_string())
        );
        assert_eq!(
            extract_quoted_path(r#"            "extended://mobs/custom/enemy.mob","#),
            Some("extended://mobs/custom/enemy.mob".to_string())
        );
        assert_eq!(extract_quoted_path("no quotes here"), None);
    }

    #[test]
    fn test_parse_base_mobs_assets_ron() {
        let content = r#"({
    "mobs": Files(
        paths: [
            "mobs/xhitara/grunt.mob",
            "mobs/xhitara/spitter.mob",
        ]
    ),
})"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_base_mobs.assets.ron");
        fs::write(&temp_file, content).unwrap();

        let result = parse_mobs_assets_ron(&temp_file).unwrap();

        assert_eq!(result.mobs.len(), 2);
        assert_eq!(result.mobs[0], "mobs/xhitara/grunt.mob");
        assert_eq!(result.mobs[1], "mobs/xhitara/spitter.mob");
        assert!(result.patches.is_empty());

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_extended_mobs_assets_ron() {
        let content = r#"({
    "extended_mobs": Files(
        paths: [
            "extended://mobs/custom/enemy.mob",
        ]
    ),
    "extended_mob_patches": Files(
        paths: [
            "extended://mobs/xhitara/grunt.mobpatch",
        ]
    ),
})"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_extended_mobs.assets.ron");
        fs::write(&temp_file, content).unwrap();

        let result = parse_mobs_assets_ron(&temp_file).unwrap();

        assert_eq!(result.mobs.len(), 1);
        assert_eq!(result.mobs[0], "mobs/custom/enemy.mob"); // extended:// stripped
        assert_eq!(result.patches.len(), 1);
        assert_eq!(result.patches[0], "mobs/xhitara/grunt.mobpatch"); // extended:// stripped

        fs::remove_file(&temp_file).ok();
    }
}
