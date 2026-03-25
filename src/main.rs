mod app;
mod event;
mod git;
mod ui;

use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use event::{Action, ClickAreas};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

fn main() -> Result<()> {
    if !git::is_git_repo() {
        eprintln!("Error: not a git repository");
        std::process::exit(1);
    }

    // Fetch remote once at startup to detect unpulled commits
    git::fetch();

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to setup terminal")?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut click_areas = ClickAreas::new();

    // Setup file system watcher
    let (fs_tx, fs_rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |_res| {
        let _ = fs_tx.send(());
    })
    .context("Failed to create file watcher")?;
    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .context("Failed to watch directory")?;

    // Debounce: ignore rapid-fire events, refresh at most once per this interval
    let debounce_interval = Duration::from_millis(500);
    let mut fs_changed = false;
    let mut last_refresh = Instant::now();

    loop {
        // Drain all pending fs events (non-blocking)
        while fs_rx.try_recv().is_ok() {
            fs_changed = true;
        }

        // Debounced refresh on file system change
        if fs_changed && last_refresh.elapsed() >= debounce_interval {
            app.refresh();
            last_refresh = Instant::now();
            fs_changed = false;
        }

        if app.dirty {
            let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
            app.adjust_scroll(visible_height);

            terminal.draw(|frame| {
                ui::render(frame, &app, &mut click_areas);
            })?;

            app.dirty = false;
        }

        if let Some(ev) = event::poll_event(Duration::from_millis(250))? {
            let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
            if let Some(action) = event::map_event(&ev, &app.mode, &click_areas) {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => app.move_up(),
                    Action::MoveDown => app.move_down(),
                    Action::ToggleStage => app.toggle_stage_selected(),
                    Action::StageAll => app.stage_all(),
                    Action::UnstageAll => app.unstage_all(),
                    Action::EnterCommitMode => app.enter_commit_mode(),
                    Action::ConfirmCommit => app.confirm_commit(),
                    Action::CancelCommit => app.cancel_commit(),
                    Action::Pull => app.pull(),
                    Action::Push => app.push(),
                    Action::ShowHelp => app.show_help(),
                    Action::CloseHelp => app.close_help(),
                    Action::Refresh => {
                        app.refresh();
                        last_refresh = Instant::now();
                        fs_changed = false;
                    }
                    Action::CommitInputChar(c) => {
                        app.commit_message.push(c);
                        app.dirty = true;
                    }
                    Action::CommitInputBackspace => {
                        app.commit_message.pop();
                        app.dirty = true;
                    }
                    Action::Resize => app.dirty = true,
                    Action::ScrollUp => app.scroll_up(),
                    Action::ScrollDown => app.scroll_down(visible_height),
                    Action::SelectIndex(idx) => {
                        if idx < app.repo.files.len() {
                            app.selected_index = idx;
                            app.toggle_stage_selected();
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
