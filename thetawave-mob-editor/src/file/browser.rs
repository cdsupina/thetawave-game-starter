use std::path::PathBuf;

use bevy::prelude::*;

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
    pub fn scan_directories(&mut self, base_dir: &PathBuf, extended_dir: Option<&PathBuf>) {
        self.roots.clear();

        // Scan base assets directory
        if base_dir.exists() {
            let mut base_root = FileNode::new_directory("base".to_string(), base_dir.clone());
            base_root.expanded = true;
            Self::scan_directory(&mut base_root, base_dir);
            if !base_root.children.is_empty() {
                self.roots.push(base_root);
            }
        }

        // Scan extended assets directory
        if let Some(ext_dir) = extended_dir {
            if ext_dir.exists() {
                let mut ext_root = FileNode::new_directory("extended".to_string(), ext_dir.clone());
                ext_root.expanded = true;
                Self::scan_directory(&mut ext_root, ext_dir);
                if !ext_root.children.is_empty() {
                    self.roots.push(ext_root);
                }
            }
        }

        self.needs_refresh = false;
    }

    fn scan_directory(node: &mut FileNode, path: &PathBuf) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories
            if name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                let mut dir_node = FileNode::new_directory(name, entry_path.clone());
                Self::scan_directory(&mut dir_node, &entry_path);
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

    /// Get all mob references (paths without extension) for dropdown selection
    pub fn get_mob_refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        for root in &self.roots {
            Self::collect_mob_refs(root, "", &mut refs);
        }
        refs
    }

    fn collect_mob_refs(node: &FileNode, prefix: &str, refs: &mut Vec<String>) {
        if node.is_directory {
            // Skip root level directory names (base/extended)
            let new_prefix = if prefix.is_empty() {
                String::new()
            } else {
                format!("{}/{}", prefix, node.name)
            };
            for child in &node.children {
                // For first-level directories under root, use the directory name directly
                let child_prefix = if prefix.is_empty() {
                    node.name.clone()
                } else {
                    new_prefix.clone()
                };
                Self::collect_mob_refs(child, &child_prefix, refs);
            }
        } else if node.name.ends_with(".mob") {
            // Convert to mob_ref format (path without extension)
            let stem = node.name.strip_suffix(".mob").unwrap_or(&node.name);
            let mob_ref = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{}/{}", prefix, stem)
            };
            refs.push(mob_ref);
        }
    }
}
