mod app;
mod event;
mod git;
mod ui;

use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppCommand, BusyAction, CommandRequest};
use event::{map_busy_event, Action, ClickAreas};

struct WorkerResponse {
    action: BusyAction,
    result: Result<String, String>,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

fn run_app_command(command: AppCommand) -> Result<String, String> {
    match command {
        AppCommand::StageAll => git::stage_all()
            .map(|()| String::from("Staged all"))
            .map_err(|e| e.to_string()),
        AppCommand::UnstageAll => git::unstage_all()
            .map(|()| String::from("Unstaged all"))
            .map_err(|e| e.to_string()),
        AppCommand::Commit { message } => git::commit(&message).map_err(|e| e.to_string()),
        AppCommand::Pull => git::pull().map_err(|e| e.to_string()),
        AppCommand::Push => git::push().map_err(|e| e.to_string()),
        AppCommand::CheckoutBranch { name } => {
            git::checkout_branch(&name).map_err(|e| e.to_string())
        }
        AppCommand::CreateBranch { name } => {
            git::create_and_checkout_branch(&name).map_err(|e| e.to_string())
        }
    }
}

fn spawn_worker(command_rx: Receiver<CommandRequest>, result_tx: Sender<WorkerResponse>) {
    thread::spawn(move || {
        while let Ok(request) = command_rx.recv() {
            let result = run_app_command(request.command);
            let _ = result_tx.send(WorkerResponse {
                action: request.busy_action,
                result,
            });
        }
    });
}

fn enqueue_command(
    app: &mut App,
    command_tx: &Sender<CommandRequest>,
    request: Option<CommandRequest>,
) {
    let Some(request) = request else {
        return;
    };

    let busy_action = request.busy_action.clone();
    app.begin_busy(busy_action.clone());
    if let Err(err) = command_tx.send(request) {
        app.finish_busy_error(busy_action, err.to_string());
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
    let (command_tx, command_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    spawn_worker(command_rx, result_tx);

    // Setup file system watcher
    let (fs_tx, fs_rx) = mpsc::channel();
    let startup_refresh_tx = fs_tx.clone();
    thread::spawn(move || {
        git::fetch();
        let _ = startup_refresh_tx.send(());
    });
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

        while let Ok(response) = result_rx.try_recv() {
            match response.result {
                Ok(message) => {
                    app.finish_busy_success(response.action, message);
                    last_refresh = Instant::now();
                    fs_changed = false;
                }
                Err(error) => app.finish_busy_error(response.action, error),
            }
        }

        // Debounced refresh on file system change
        if !app.is_busy() && fs_changed && last_refresh.elapsed() >= debounce_interval {
            app.refresh();
            last_refresh = Instant::now();
            fs_changed = false;
        }

        if app.dirty {
            let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
            app.adjust_scroll(visible_height);
            app.adjust_branch_scroll(visible_height);

            terminal.draw(|frame| {
                ui::render(frame, &app, &mut click_areas);
            })?;

            app.dirty = false;
        }

        if let Some(ev) = event::poll_event(Duration::from_millis(250))? {
            let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
            let action = if app.is_busy() {
                map_busy_event(&ev)
            } else {
                event::map_event(&ev, &app.mode, &click_areas)
            };

            if let Some(action) = action {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => app.move_up(),
                    Action::MoveDown => app.move_down(),
                    Action::ToggleStage => app.toggle_stage_selected(),
                    Action::StageAll => {
                        let request = app.stage_all();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::UnstageAll => {
                        let request = app.unstage_all();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::EnterCommitMode => app.enter_commit_mode(),
                    Action::ConfirmCommit => {
                        let request = app.confirm_commit();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::CancelCommit => app.cancel_commit(),
                    Action::Pull => {
                        let request = app.pull();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::Push => {
                        let request = app.push();
                        enqueue_command(&mut app, &command_tx, request);
                    }
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
                    Action::ShowBranchList => app.show_branch_list(),
                    Action::CloseBranchList => app.close_branch_list(),
                    Action::BranchListMoveUp => app.branch_list_move_up(),
                    Action::BranchListMoveDown => app.branch_list_move_down(),
                    Action::BranchListSelect => {
                        let request = app.confirm_branch_switch();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::EnterBranchCreate => app.enter_branch_create(),
                    Action::ConfirmBranchCreate => {
                        let request = app.confirm_branch_create();
                        enqueue_command(&mut app, &command_tx, request);
                    }
                    Action::CancelBranchCreate => app.cancel_branch_create(),
                    Action::BranchInputChar(c) => {
                        app.branch_name_input.push(c);
                        app.dirty = true;
                    }
                    Action::BranchInputBackspace => {
                        app.branch_name_input.pop();
                        app.dirty = true;
                    }
                    Action::BranchSelectIndex(idx) => {
                        if idx < app.branches.len() {
                            app.branch_selected = idx;
                            let request = app.confirm_branch_switch();
                            enqueue_command(&mut app, &command_tx, request);
                        }
                    }
                    Action::ScrollUp => {
                        if app.mode == app::Mode::Help {
                            app.help_scroll_up();
                        } else if app.mode == app::Mode::BranchList {
                            app.branch_list_scroll_up();
                        } else {
                            app.scroll_up();
                        }
                    }
                    Action::ScrollDown => {
                        if app.mode == app::Mode::Help {
                            app.help_scroll_down(ui::HELP_LINE_COUNT, visible_height);
                        } else if app.mode == app::Mode::BranchList {
                            app.branch_list_scroll_down(visible_height);
                        } else {
                            app.scroll_down(visible_height);
                        }
                    }
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
