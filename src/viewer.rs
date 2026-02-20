use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::highlight::{Highlighter, StyledSegment};

/// View mode for the code viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Normal,
    GitDiff,
}

/// Type of diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}

/// A line in diff view with its type and highlighted segments.
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub segments: Vec<StyledSegment>,
    pub line_number: Option<usize>,
}

/// The state of the code viewer panel.
pub struct CodeViewer {
    /// Currently loaded file path (if any).
    pub file_path: Option<PathBuf>,
    /// Pre-highlighted lines ready for rendering (normal mode).
    pub lines: Vec<Vec<StyledSegment>>,
    /// Diff lines (git diff mode).
    pub diff_lines: Vec<DiffLine>,
    /// Current view mode.
    pub view_mode: ViewMode,
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
            diff_lines: Vec::new(),
            view_mode: ViewMode::Normal,
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

    /// Toggle between normal and git diff view.
    pub fn toggle_view_mode(&mut self, highlighter: &Highlighter) {
        self.view_mode = match self.view_mode {
            ViewMode::Normal => ViewMode::GitDiff,
            ViewMode::GitDiff => ViewMode::Normal,
        };
        self.scroll = 0;

        // Load diff if switching to diff mode and file is loaded.
        if self.view_mode == ViewMode::GitDiff {
            if let Some(path) = self.file_path.clone() {
                self.load_git_diff(&path, highlighter);
            }
        }
    }

    /// Load git diff for the current file.
    fn load_git_diff(&mut self, path: &Path, highlighter: &Highlighter) {
        self.diff_lines.clear();
        self.message = None;

        // Run git diff to get changes.
        let output = Command::new("git")
            .arg("diff")
            .arg("HEAD")
            .arg(path)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                self.message = Some(format!("Error running git: {}", e));
                return;
            }
        };

        if !output.status.success() {
            self.message = Some("Not a git repository or file not tracked".into());
            return;
        }

        let diff_text = String::from_utf8_lossy(&output.stdout);

        if diff_text.trim().is_empty() {
            self.message = Some("No changes in working directory".into());
            return;
        }

        // Parse diff output.
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut line_num = 0;
        for line in diff_text.lines() {
            // Skip diff headers.
            if line.starts_with("diff --git")
                || line.starts_with("index ")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
            {
                continue;
            }

            // Parse hunk headers to track line numbers.
            if line.starts_with("@@") {
                // Extract line number from hunk header.
                if let Some(parts) = line.split("@@").nth(1) {
                    if let Some(new_line_info) = parts.trim().split_whitespace().nth(1) {
                        if let Some(num_str) = new_line_info.strip_prefix('+') {
                            if let Some(num) = num_str.split(',').next() {
                                line_num = num.parse::<usize>().unwrap_or(0);
                            }
                        }
                    }
                }
                continue;
            }

            let (line_type, content) = if let Some(rest) = line.strip_prefix('+') {
                line_num += 1;
                (DiffLineType::Added, rest)
            } else if let Some(rest) = line.strip_prefix('-') {
                (DiffLineType::Removed, rest)
            } else if let Some(rest) = line.strip_prefix(' ') {
                line_num += 1;
                (DiffLineType::Context, rest)
            } else {
                continue;
            };

            // Highlight the content.
            let segments = if content.is_empty() {
                vec![StyledSegment {
                    text: String::new(),
                    style: ratatui::style::Style::default(),
                }]
            } else {
                highlighter.highlight(content, ext)
                    .into_iter()
                    .flatten()
                    .collect()
            };

            self.diff_lines.push(DiffLine {
                line_type,
                segments,
                line_number: match line_type {
                    DiffLineType::Removed => None,
                    _ => Some(line_num),
                },
            });
        }

        if self.diff_lines.is_empty() {
            self.message = Some("No diff output".into());
        }
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
        let total = match self.view_mode {
            ViewMode::Normal => self.lines.len(),
            ViewMode::GitDiff => self.diff_lines.len(),
        };

        if total == 0 {
            0
        } else {
            total.saturating_sub(1)
        }
    }

    /// Jump to the next change (added or removed line) in diff mode.
    pub fn jump_to_next_change(&mut self) {
        if self.view_mode != ViewMode::GitDiff {
            return;
        }

        // Start searching from the line after current scroll position.
        let start = self.scroll + 1;
        for (idx, line) in self.diff_lines.iter().enumerate().skip(start) {
            if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Removed {
                self.scroll = idx;
                return;
            }
        }

        // If no change found after current position, wrap to beginning.
        for (idx, line) in self.diff_lines.iter().enumerate().take(start) {
            if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Removed {
                self.scroll = idx;
                return;
            }
        }
    }

    /// Jump to the previous change (added or removed line) in diff mode.
    pub fn jump_to_prev_change(&mut self) {
        if self.view_mode != ViewMode::GitDiff {
            return;
        }

        // Search backwards from current scroll position.
        for idx in (0..self.scroll).rev() {
            if let Some(line) = self.diff_lines.get(idx) {
                if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Removed {
                    self.scroll = idx;
                    return;
                }
            }
        }

        // If no change found before current position, wrap to end.
        for idx in (self.scroll + 1..self.diff_lines.len()).rev() {
            if let Some(line) = self.diff_lines.get(idx) {
                if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Removed {
                    self.scroll = idx;
                    return;
                }
            }
        }
    }
}
