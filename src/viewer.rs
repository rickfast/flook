use std::fs;
use std::path::{Path, PathBuf};

use crate::highlight::{Highlighter, StyledSegment};

/// The state of the code viewer panel.
pub struct CodeViewer {
    /// Currently loaded file path (if any).
    pub file_path: Option<PathBuf>,
    /// Pre-highlighted lines ready for rendering.
    pub lines: Vec<Vec<StyledSegment>>,
    /// Current scroll offset (first visible line).
    pub scroll: usize,
    /// Status/error message to display instead of content.
    pub message: Option<String>,
}

impl CodeViewer {
    pub fn new() -> Self {
        Self {
            file_path: None,
            lines: Vec::new(),
            scroll: 0,
            message: Some("Select a file to view".into()),
        }
    }

    /// Load and highlight a file.
    pub fn load(&mut self, path: &Path, highlighter: &Highlighter) {
        self.file_path = Some(path.to_path_buf());
        self.scroll = 0;
        self.message = None;
        self.lines.clear();

        // Read raw bytes first for binary detection.
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
                return;
            }
        };

        // Binary detection: check first 8KB for null bytes.
        let check_len = bytes.len().min(8192);
        if bytes[..check_len].contains(&0) {
            self.message = Some("Binary file".into());
            return;
        }

        // Try to interpret as UTF-8.
        let source = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                self.message = Some("Cannot display: file is not valid UTF-8".into());
                return;
            }
        };

        // Determine extension for syntax detection.
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        self.lines = highlighter.highlight(&source, ext);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max = self.max_scroll();
        self.scroll = (self.scroll + amount).min(max);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll();
    }

    fn max_scroll(&self) -> usize {
        if self.lines.is_empty() {
            0
        } else {
            self.lines.len().saturating_sub(1)
        }
    }
}
