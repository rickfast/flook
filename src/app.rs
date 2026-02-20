use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::highlight::Highlighter;
use crate::tree::FileTree;
use crate::viewer::CodeViewer;

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Tree,
    Viewer,
}

/// Top-level application state.
pub struct App {
    pub tree: FileTree,
    pub viewer: CodeViewer,
    pub highlighter: Highlighter,
    pub focus: Panel,
    pub quit: bool,
}

impl App {
    pub fn new(root: &Path) -> Self {
        Self {
            tree: FileTree::new(root),
            viewer: CodeViewer::new(),
            highlighter: Highlighter::new(),
            focus: Panel::Tree,
            quit: false,
        }
    }

    /// Pre-load a file (used when CLI argument is a file path).
    pub fn load_file(&mut self, path: &Path) {
        self.viewer.load(path, &self.highlighter);
    }

    /// Dispatch a key event to the appropriate handler.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global keys.
        match key.code {
            KeyCode::Char('q') => {
                self.quit = true;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true;
                return;
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panel::Tree => Panel::Viewer,
                    Panel::Viewer => Panel::Tree,
                };
                return;
            }
            _ => {}
        }

        match self.focus {
            Panel::Tree => self.handle_tree_key(key),
            Panel::Viewer => self.handle_viewer_key(key),
        }
    }

    fn handle_tree_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.tree.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.tree.move_up(),
            KeyCode::Enter => {
                if let Some(file_path) = self.tree.toggle_selected() {
                    self.viewer.load(&file_path, &self.highlighter);
                }
            }
            _ => {}
        }
    }

    fn handle_viewer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.viewer.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.viewer.scroll_up(1),
            KeyCode::PageDown => self.viewer.scroll_down(20),
            KeyCode::PageUp => self.viewer.scroll_up(20),
            KeyCode::Char('g') | KeyCode::Home => self.viewer.scroll_to_top(),
            KeyCode::Char('G') | KeyCode::End => self.viewer.scroll_to_bottom(),
            KeyCode::Char('d') => self.viewer.toggle_view_mode(&self.highlighter),
            KeyCode::Char('n') | KeyCode::Right => self.viewer.jump_to_next_change(),
            KeyCode::Char('N') | KeyCode::Left => self.viewer.jump_to_prev_change(),
            _ => {}
        }
    }
}
