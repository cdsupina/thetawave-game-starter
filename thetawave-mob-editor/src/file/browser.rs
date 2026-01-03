//! File browser tree structure and scanning.
//!
//! Contains [`FileTreeState`] for managing the file browser panel,
//! including directory scanning and expansion state.

use std::collections::HashSet;
use std::path::PathBuf;

use bevy::prelude::Resource;

/// Represents a node in the file tree
#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

impl FileNode {
    pub fn new_file(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            is_directory: false,
            children: Vec::new(),
            expanded: false,
        }
    }

    pub fn new_directory(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            is_directory: true,
            children: Vec::new(),
            expanded: false,
        }
    }
}

/// Resource storing the file tree state
#[derive(Resource, Default)]
pub struct FileTreeState {
    /// Root nodes of the file tree (base and extended directories)
    pub roots: Vec<FileNode>,

    /// Currently selected file path
    pub selected: Option<PathBuf>,

    /// Whether the tree needs to be rescanned
    pub needs_refresh: bool,
}

impl FileTreeState {
    /// Scan the mobs directories and build the file tree
    pub fn scan_directories(
        &mut self,
        base_dir: &PathBuf,
        extended_dir: Option<&PathBuf>,
        show_base_mobs: bool,
    ) {
        self.roots.clear();

        // Scan base assets directory (only if show_base_mobs is true)
        if show_base_mobs && base_dir.exists() {
            let mut base_root = FileNode::new_directory("base".to_string(), base_dir.clone());
            base_root.expanded = true;
            Self::scan_directory(&mut base_root, base_dir);
            if !base_root.children.is_empty() {
                self.roots.push(base_root);
            }
        }

        // Scan extended assets directory (create if needed, always show so users can create mobs)
        if let Some(ext_dir) = extended_dir {
            // Create the directory if it doesn't exist
            if !ext_dir.exists()
                && let Err(e) = std::fs::create_dir_all(ext_dir)
            {
                bevy::log::warn!(
                    "Failed to create extended assets directory {:?}: {}",
                    ext_dir,
                    e
                );
            }

            if ext_dir.exists() {
                let mut ext_root = FileNode::new_directory("extended".to_string(), ext_dir.clone());
                ext_root.expanded = true;
                Self::scan_directory(&mut ext_root, ext_dir);
                self.roots.push(ext_root);
            }
        }

        self.needs_refresh = false;
    }

    /// Maximum recursion depth for directory scanning to prevent infinite loops
    const MAX_SCAN_DEPTH: usize = 10;

    fn scan_directory(node: &mut FileNode, path: &PathBuf) {
        Self::scan_directory_with_depth(node, path, 0);
    }

    fn scan_directory_with_depth(node: &mut FileNode, path: &PathBuf, depth: usize) {
        if depth >= Self::MAX_SCAN_DEPTH {
            bevy::log::warn!(
                "Max directory scan depth {} reached at {:?}, stopping recursion",
                Self::MAX_SCAN_DEPTH,
                path
            );
            return;
        }

        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => {
                bevy::log::warn!(
                    "Failed to read directory {:?}: {}. Contents will not be shown.",
                    path,
                    e
                );
                return;
            }
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    bevy::log::debug!("Skipping inaccessible entry in {:?}: {}", path, e);
                    continue;
                }
            };
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories
            if name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                let mut dir_node = FileNode::new_directory(name, entry_path.clone());
                Self::scan_directory_with_depth(&mut dir_node, &entry_path, depth + 1);
                dirs.push(dir_node);
            } else if let Some(ext) = entry_path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "mob" || ext_str == "mobpatch" {
                    files.push(FileNode::new_file(name, entry_path));
                }
            }
        }

        // Sort directories and files alphabetically
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        // Directories first, then files
        node.children.extend(dirs);
        node.children.extend(files);
    }

    /// Toggle expansion state of a directory
    pub fn toggle_expanded(&mut self, path: &PathBuf) {
        Self::toggle_expanded_recursive(&mut self.roots, path);
    }

    fn toggle_expanded_recursive(nodes: &mut [FileNode], path: &PathBuf) {
        for node in nodes.iter_mut() {
            if &node.path == path && node.is_directory {
                node.expanded = !node.expanded;
                return;
            }
            if node.is_directory {
                Self::toggle_expanded_recursive(&mut node.children, path);
            }
        }
    }

    /// Get categorized mob refs - returns (base_refs, extended_refs)
    /// where each ref does NOT include the "base/" or "extended/" prefix
    pub fn get_categorized_mob_refs(&self) -> (Vec<String>, Vec<String>) {
        let mut base_refs = Vec::new();
        let mut extended_refs = Vec::new();

        for root in &self.roots {
            if root.name == "base" {
                for child in &root.children {
                    Self::collect_mob_refs_no_root(child, "", &mut base_refs);
                }
            } else if root.name == "extended" {
                for child in &root.children {
                    Self::collect_mob_refs_no_root(child, "", &mut extended_refs);
                }
            }
        }

        (base_refs, extended_refs)
    }

    /// Get mob refs from only the "base" root (mobs that can be patched)
    /// Returns refs without the "base/" prefix (e.g., "xhitara/spitter")
    pub fn get_base_mob_refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        for root in &self.roots {
            if root.name == "base" {
                // Collect from children of "base" root, skipping the root name
                for child in &root.children {
                    Self::collect_mob_refs_no_root(child, "", &mut refs);
                }
            }
        }
        refs
    }

    fn collect_mob_refs_no_root(node: &FileNode, prefix: &str, refs: &mut Vec<String>) {
        if node.is_directory {
            let new_prefix = if prefix.is_empty() {
                node.name.clone()
            } else {
                format!("{}/{}", prefix, node.name)
            };
            for child in &node.children {
                Self::collect_mob_refs_no_root(child, &new_prefix, refs);
            }
        } else if node.name.ends_with(".mob") {
            let stem = node.name.strip_suffix(".mob").unwrap_or(&node.name);
            let mob_ref = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{}/{}", prefix, stem)
            };
            refs.push(mob_ref);
        }
    }

    /// Get existing patch refs from "extended" root
    /// Returns refs without the "extended/" prefix (e.g., "xhitara/spitter")
    pub fn get_existing_patch_refs(&self) -> HashSet<String> {
        let mut refs = HashSet::new();
        for root in &self.roots {
            if root.name == "extended" {
                // Collect from children of "extended" root, skipping the root name
                for child in &root.children {
                    Self::collect_patch_refs_no_root(child, "", &mut refs);
                }
            }
        }
        refs
    }

    fn collect_patch_refs_no_root(node: &FileNode, prefix: &str, refs: &mut HashSet<String>) {
        if node.is_directory {
            let new_prefix = if prefix.is_empty() {
                node.name.clone()
            } else {
                format!("{}/{}", prefix, node.name)
            };
            for child in &node.children {
                Self::collect_patch_refs_no_root(child, &new_prefix, refs);
            }
        } else if node.name.ends_with(".mobpatch") {
            let stem = node.name.strip_suffix(".mobpatch").unwrap_or(&node.name);
            let mob_ref = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{}/{}", prefix, stem)
            };
            refs.insert(mob_ref);
        }
    }
}
