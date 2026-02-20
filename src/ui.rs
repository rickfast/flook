use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};
use crate::viewer::{DiffLineType, ViewMode};

/// Render the full UI.
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(1)])
        .split(frame.area());

    draw_tree(frame, app, chunks[0]);
    draw_viewer(frame, app, chunks[1]);
}

fn border_style(app: &App, panel: Panel) -> Style {
    if app.focus == panel {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn draw_tree(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Files ")
        .border_style(border_style(app, Panel::Tree));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Determine visible range with scrolling to keep selected item visible.
    let height = inner.height as usize;
    if height == 0 || app.tree.entries.is_empty() {
        return;
    }

    let total = app.tree.entries.len();
    let selected = app.tree.selected;

    // Calculate scroll offset to keep selected visible.
    let scroll_offset = if selected < height / 2 {
        0
    } else if selected + height / 2 >= total {
        total.saturating_sub(height)
    } else {
        selected.saturating_sub(height / 2)
    };

    let visible_end = (scroll_offset + height).min(total);

    let lines: Vec<Line> = app.tree.entries[scroll_offset..visible_end]
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let global_idx = scroll_offset + i;
            let indent = "  ".repeat(entry.depth);

            let icon = if entry.is_dir {
                if entry.is_expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let text = format!("{}{}{}", indent, icon, entry.name);

            let style = if global_idx == selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(text, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_viewer(frame: &mut Frame, app: &App, area: Rect) {
    let mode_indicator = match app.viewer.view_mode {
        ViewMode::Normal => "",
        ViewMode::GitDiff => " [DIFF] ",
    };

    let title = match &app.viewer.file_path {
        Some(p) => format!(" {}{} ", p.display(), mode_indicator),
        None => " Viewer ".to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style(app, Panel::Viewer));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // If there's a message (error / binary / placeholder), show it.
    if let Some(msg) = &app.viewer.message {
        let paragraph = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, inner);
        return;
    }

    let height = inner.height as usize;
    if height == 0 {
        return;
    }

    match app.viewer.view_mode {
        ViewMode::Normal => draw_normal_view(frame, app, inner),
        ViewMode::GitDiff => draw_diff_view(frame, app, inner),
    }
}

fn draw_normal_view(frame: &mut Frame, app: &App, area: Rect) {
    let height = area.height as usize;
    let scroll = app.viewer.scroll;
    let end = (scroll + height).min(app.viewer.lines.len());

    // Line number gutter width.
    let total_lines = app.viewer.lines.len();
    let gutter_width = format!("{}", total_lines).len() + 1; // +1 for space

    let lines: Vec<Line> = app.viewer.lines[scroll..end]
        .iter()
        .enumerate()
        .map(|(i, segments)| {
            let line_num = scroll + i + 1;
            let gutter = format!("{:>width$} ", line_num, width = gutter_width - 1);

            let mut spans = vec![Span::styled(
                gutter,
                Style::default().fg(Color::DarkGray),
            )];

            for seg in segments {
                spans.push(Span::styled(seg.text.clone(), seg.style));
            }

            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_diff_view(frame: &mut Frame, app: &App, area: Rect) {
    let height = area.height as usize;
    let scroll = app.viewer.scroll;
    let end = (scroll + height).min(app.viewer.diff_lines.len());

    // Line number gutter width (find max line number).
    let max_line_num = app.viewer.diff_lines
        .iter()
        .filter_map(|d| d.line_number)
        .max()
        .unwrap_or(0);
    let gutter_width = format!("{}", max_line_num).len() + 1;

    let lines: Vec<Line> = app.viewer.diff_lines[scroll..end]
        .iter()
        .map(|diff_line| {
            // Determine line background and prefix based on type.
            let (bg_color, prefix, prefix_color) = match diff_line.line_type {
                DiffLineType::Added => (Color::Rgb(0, 64, 0), "+ ", Color::Green),
                DiffLineType::Removed => (Color::Rgb(64, 0, 0), "- ", Color::Red),
                DiffLineType::Context => (Color::Reset, "  ", Color::Reset),
            };

            // Format line number or leave blank for removed lines.
            let line_num_str = match diff_line.line_number {
                Some(num) => format!("{:>width$} ", num, width = gutter_width - 1),
                None => format!("{:>width$} ", "", width = gutter_width - 1),
            };

            let mut spans = vec![
                Span::styled(
                    line_num_str,
                    Style::default().fg(Color::DarkGray).bg(bg_color),
                ),
                Span::styled(
                    prefix,
                    Style::default().fg(prefix_color).bg(bg_color),
                ),
            ];

            // Apply background color to all content spans.
            for seg in &diff_line.segments {
                spans.push(Span::styled(
                    seg.text.clone(),
                    seg.style.bg(bg_color),
                ));
            }

            Line::from(spans).style(Style::default().bg(bg_color))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
