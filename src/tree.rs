use std::fs;
use std::path::{Path, PathBuf};

/// A single entry in the flattened file tree.
#[derive(Clone, Debug)]
pub struct TreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
}

/// Flat file tree model with expand/collapse support.
pub struct FileTree {
    pub entries: Vec<TreeEntry>,
    pub selected: usize,
}

impl FileTree {
    /// Build a new tree rooted at `root`. The root directory's immediate
    /// children are loaded (expanded) by default.
    pub fn new(root: &Path) -> Self {
        let mut entries = Vec::new();

        // Add root entry as expanded.
        entries.push(TreeEntry {
            path: root.to_path_buf(),
            name: root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.to_string_lossy().into_owned()),
            depth: 0,
            is_dir: true,
            is_expanded: true,
        });

        // Load root children.
        let children = read_children(root, 1);
        entries.extend(children);

        Self {
            entries,
            selected: 0,
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Toggle expand/collapse for a directory, or return the path if it's a file.
    pub fn toggle_selected(&mut self) -> Option<PathBuf> {
        let entry = &self.entries[self.selected];

        if !entry.is_dir {
            return Some(entry.path.clone());
        }

        let idx = self.selected;
        let entry = &self.entries[idx];

        if entry.is_expanded {
            self.collapse(idx);
        } else {
            self.expand(idx);
        }

        None
    }

    fn expand(&mut self, idx: usize) {
        let entry = &mut self.entries[idx];
        entry.is_expanded = true;
        let depth = entry.depth + 1;
        let path = entry.path.clone();

        let children = read_children(&path, depth);
        // Insert children right after the current entry.
        let insert_pos = idx + 1;
        self.entries.splice(insert_pos..insert_pos, children);
    }

    fn collapse(&mut self, idx: usize) {
        let parent_depth = self.entries[idx].depth;
        self.entries[idx].is_expanded = false;

        // Remove all entries after idx that have depth > parent_depth.
        let mut end = idx + 1;
        while end < self.entries.len() && self.entries[end].depth > parent_depth {
            end += 1;
        }
        self.entries.drain((idx + 1)..end);
    }

    /// Select the entry matching the given path (if any).
    pub fn select_path(&mut self, target: &Path) {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.path == target {
                self.selected = i;
                return;
            }
        }
    }
}

/// Read direct children of a directory, sorted: directories first, then files,
/// each group alphabetically. Silently skips entries on error.
fn read_children(dir: &Path, depth: usize) -> Vec<TreeEntry> {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip hidden files/directories.
        if name.starts_with('.') {
            continue;
        }

        let is_dir = path.is_dir();

        let tree_entry = TreeEntry {
            path,
            name,
            depth,
            is_dir,
            is_expanded: false,
        };

        if is_dir {
            dirs.push(tree_entry);
        } else {
            files.push(tree_entry);
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    dirs.extend(files);
    dirs
}
