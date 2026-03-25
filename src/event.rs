use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::time::Duration;

use crate::app::Mode;

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    ToggleStage,
    StageAll,
    UnstageAll,
    EnterCommitMode,
    ConfirmCommit,
    CancelCommit,
    Pull,
    Push,
    Refresh,
    CommitInputChar(char),
    CommitInputBackspace,
    SelectIndex(usize),
    ScrollUp,
    ScrollDown,
    Resize,
}

pub struct ClickAreas {
    pub file_rows: Vec<(Rect, usize)>,
    pub buttons: Vec<(Rect, ButtonAction)>,
}

#[derive(Clone, Debug)]
pub enum ButtonAction {
    StageAll,
    Commit,
    Pull,
    Push,
    ConfirmCommit,
    CancelCommit,
}

impl ClickAreas {
    pub fn new() -> Self {
        Self {
            file_rows: Vec::new(),
            buttons: Vec::new(),
        }
    }
}

pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn map_event(
    event: &Event,
    mode: &Mode,
    click_areas: &ClickAreas,
) -> Option<Action> {
    match mode {
        Mode::Normal => map_normal_event(event, click_areas),
        Mode::CommitInput => map_commit_event(event, click_areas),
    }
}

fn map_normal_event(event: &Event, click_areas: &ClickAreas) -> Option<Action> {
    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => {
            if *modifiers == KeyModifiers::CONTROL && *code == KeyCode::Char('c') {
                return Some(Action::Quit);
            }
            match code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
                KeyCode::Char('s') | KeyCode::Enter => Some(Action::ToggleStage),
                KeyCode::Char('A') => Some(Action::StageAll),
                KeyCode::Char('U') => Some(Action::UnstageAll),
                KeyCode::Char('c') => Some(Action::EnterCommitMode),
                KeyCode::Char('p') => Some(Action::Push),
                KeyCode::Char('P') => Some(Action::Pull),
                KeyCode::Char('r') => Some(Action::Refresh),
                _ => None,
            }
        }
        Event::Mouse(m) => match m.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let col = m.column;
                let row = m.row;
                // Check button clicks
                for (rect, btn) in &click_areas.buttons {
                    if col >= rect.x
                        && col < rect.x + rect.width
                        && row >= rect.y
                        && row < rect.y + rect.height
                    {
                        return match btn {
                            ButtonAction::StageAll => Some(Action::StageAll),
                            ButtonAction::Commit => Some(Action::EnterCommitMode),
                            ButtonAction::Pull => Some(Action::Pull),
                            ButtonAction::Push => Some(Action::Push),
                            _ => None,
                        };
                    }
                }
                // Check file row clicks
                for (rect, idx) in &click_areas.file_rows {
                    if col >= rect.x
                        && col < rect.x + rect.width
                        && row >= rect.y
                        && row < rect.y + rect.height
                    {
                        return Some(Action::SelectIndex(*idx));
                    }
                }
                None
            }
            MouseEventKind::ScrollUp => Some(Action::ScrollUp),
            MouseEventKind::ScrollDown => Some(Action::ScrollDown),
            _ => None,
        },
        Event::Resize(_, _) => Some(Action::Resize),
        _ => None,
    }
}

fn map_commit_event(event: &Event, click_areas: &ClickAreas) -> Option<Action> {
    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => {
            if *modifiers == KeyModifiers::CONTROL && *code == KeyCode::Char('c') {
                return Some(Action::CancelCommit);
            }
            match code {
                KeyCode::Enter => Some(Action::ConfirmCommit),
                KeyCode::Esc => Some(Action::CancelCommit),
                KeyCode::Backspace => Some(Action::CommitInputBackspace),
                KeyCode::Char(c) => Some(Action::CommitInputChar(*c)),
                _ => None,
            }
        }
        Event::Mouse(m) => match m.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let col = m.column;
                let row = m.row;
                for (rect, btn) in &click_areas.buttons {
                    if col >= rect.x
                        && col < rect.x + rect.width
                        && row >= rect.y
                        && row < rect.y + rect.height
                    {
                        return match btn {
                            ButtonAction::ConfirmCommit => Some(Action::ConfirmCommit),
                            ButtonAction::CancelCommit => Some(Action::CancelCommit),
                            _ => None,
                        };
                    }
                }
                None
            }
            MouseEventKind::ScrollUp => Some(Action::ScrollUp),
            MouseEventKind::ScrollDown => Some(Action::ScrollDown),
            _ => None,
        },
        Event::Resize(_, _) => Some(Action::Resize),
        _ => None,
    }
}
