mod app;
mod event;
mod git;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
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

    loop {
        // Adjust scroll before rendering
        let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
        app.adjust_scroll(visible_height);

        terminal.draw(|frame| {
            ui::render(frame, &app, &mut click_areas);
        })?;

        if let Some(ev) = event::poll_event(Duration::from_millis(250))? {
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
                    Action::Push => app.push(),
                    Action::Refresh => app.refresh(),
                    Action::CommitInputChar(c) => app.commit_message.push(c),
                    Action::CommitInputBackspace => {
                        app.commit_message.pop();
                    }
                    Action::ScrollUp => app.scroll_up(),
                    Action::ScrollDown => app.scroll_down(visible_height),
                    Action::SelectIndex(idx) => {
                        if idx < app.repo.files.len() {
                            if app.selected_index == idx {
                                app.toggle_stage_selected();
                            } else {
                                app.selected_index = idx;
                            }
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
