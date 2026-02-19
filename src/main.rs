mod app;
mod highlight;
mod tree;
mod ui;
mod viewer;

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

/// flook: Terminal code browser with syntax highlighting
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Directory or file to browse (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let path = cli.path.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: {}", cli.path.display(), e),
        )
    })?;

    // Determine root directory and optional file to pre-load.
    let (root, file_to_open) = if path.is_file() {
        let parent = path.parent().unwrap_or(&path);
        (parent.to_path_buf(), Some(path.clone()))
    } else if path.is_dir() {
        (path, None)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: not a file or directory", cli.path.display()),
        ));
    };

    // Terminal setup.
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&root);

    // Pre-load file if CLI arg was a file.
    if let Some(ref file) = file_to_open {
        app.tree.select_path(file);
        app.load_file(file);
    }

    // Main event loop.
    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }

        if app.quit {
            break;
        }
    }

    // Terminal restore.
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
