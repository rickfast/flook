use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};

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
    let title = match &app.viewer.file_path {
        Some(p) => format!(" {} ", p.display()),
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
    frame.render_widget(paragraph, inner);
}
