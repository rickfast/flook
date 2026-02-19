# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`flook` is a terminal-based code browser with syntax highlighting, built in Rust using the ratatui TUI framework. It provides a two-panel interface: a file tree navigator and a code viewer with line numbers and syntax highlighting.

## Development Commands

### Build and Run
```bash
cargo build               # Build the project
cargo run                 # Run with current directory
cargo run -- <path>       # Browse specific directory or file
cargo run --release       # Build and run optimized binary
```

### Testing and Quality
```bash
cargo test               # Run tests
cargo clippy             # Run linter
cargo fmt                # Format code
cargo check              # Fast compile check without building binary
```

## Architecture

### Module Structure

The codebase follows a clear separation of concerns:

- **main.rs**: Entry point that handles CLI parsing (via clap), terminal initialization (raw mode, alternate screen), and the main event loop
- **app.rs**: Central application state and key event dispatcher. Routes events to tree or viewer based on focus
- **tree.rs**: File tree model using a **flat list structure** (not nested/recursive) for efficient rendering and scrolling
- **viewer.rs**: Code viewer that handles file loading, binary detection, UTF-8 validation, and scroll state
- **highlight.rs**: Bridge between syntect's syntax highlighting and ratatui's styling system
- **ui.rs**: All rendering logic, including two-panel layout, borders, focus indication, and scroll calculations

### Key Design Patterns

**Flat Tree Model**: The file tree is stored as a flat `Vec<TreeEntry>` rather than nested structs. Expand/collapse operations insert or remove entries from the vector. This enables efficient rendering without recursive traversal and keeps the selected index simple.

**Pre-highlighting**: When a file is loaded, all lines are immediately highlighted and converted to `Vec<Vec<StyledSegment>>`. This trades memory for speed - scrolling is instant since no highlighting happens during render.

**Panel Focus System**: The `App` struct maintains a `focus: Panel` enum (Tree or Viewer). The `handle_key` method dispatches to `handle_tree_key` or `handle_viewer_key` based on focus. Tab switches focus, q/Ctrl-C always quits.

**Scroll Management**: Both tree and viewer implement smart scrolling:
- Tree scroll keeps selected item vertically centered when possible
- Viewer scroll uses simple offset with bounds checking
- Both calculate visible ranges based on terminal height

**Binary/UTF-8 Detection**: The viewer reads files as bytes first, checks for null bytes in the first 8KB to detect binaries, then validates UTF-8 before attempting to display.

### Key Event Handling

- **Tab**: Switch focus between tree and viewer
- **j/k or Up/Down**: Navigate (vim-style)
- **Enter**: In tree, toggle directory or load file
- **PageUp/PageDown**: Scroll viewer by 20 lines
- **g/G or Home/End**: Scroll viewer to top/bottom
- **q or Ctrl-C**: Quit

### Adding New Features

**New syntax theme**: Modify `Highlighter::new()` in highlight.rs to select a different theme from ThemeSet

**New key bindings**: Add cases to `handle_tree_key()` or `handle_viewer_key()` in app.rs

**New panels**: Add variant to `Panel` enum in app.rs, update UI layout in ui.rs, add handler methods

**File filtering**: Modify `read_children()` in tree.rs (currently skips dotfiles)
