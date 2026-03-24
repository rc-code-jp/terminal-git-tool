use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Rect;

use git_pocket::app::Mode;
use git_pocket::event::{map_event, Action, ButtonAction, ClickAreas};

fn key_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_event_with_mod(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn mouse_click(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn empty_click_areas() -> ClickAreas {
    ClickAreas::new()
}

// =============================================================================
// Normalモード: キーボード
// =============================================================================

#[test]
fn normal_q_quits() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('q'));
    let action = map_event(&ev, &Mode::Normal, &areas);
    assert_eq!(action, Some(Action::Quit));
}

#[test]
fn normal_esc_quits() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Esc);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Quit));
}

#[test]
fn normal_ctrl_c_quits() {
    let areas = empty_click_areas();
    let ev = key_event_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Quit));
}

#[test]
fn normal_j_moves_down() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('j'));
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::MoveDown));
}

#[test]
fn normal_k_moves_up() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('k'));
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::MoveUp));
}

#[test]
fn normal_arrow_down() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Down);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::MoveDown));
}

#[test]
fn normal_arrow_up() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Up);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::MoveUp));
}

#[test]
fn normal_s_stages() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('s'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ToggleStage)
    );
}

#[test]
fn normal_enter_stages() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Enter);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ToggleStage)
    );
}

#[test]
fn normal_shift_a_stage_all() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('A'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::StageAll)
    );
}

#[test]
fn normal_shift_u_unstage_all() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('U'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::UnstageAll)
    );
}

#[test]
fn normal_c_enters_commit_mode() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('c'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::EnterCommitMode)
    );
}

#[test]
fn normal_p_pushes() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('p'));
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Push));
}

#[test]
fn normal_r_refreshes() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('r'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::Refresh)
    );
}

#[test]
fn normal_unknown_key_returns_none() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('z'));
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), None);
}

// =============================================================================
// CommitInputモード: キーボード
// =============================================================================

#[test]
fn commit_enter_confirms() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Enter);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::ConfirmCommit)
    );
}

#[test]
fn commit_esc_cancels() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Esc);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CancelCommit)
    );
}

#[test]
fn commit_ctrl_c_cancels() {
    let areas = empty_click_areas();
    let ev = key_event_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CancelCommit)
    );
}

#[test]
fn commit_backspace() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Backspace);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CommitInputBackspace)
    );
}

#[test]
fn commit_char_input() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('x'));
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CommitInputChar('x'))
    );
}

#[test]
fn commit_j_is_char_input_not_move() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('j'));
    // In commit mode, 'j' should be treated as character input, not move down
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CommitInputChar('j'))
    );
}

// =============================================================================
// マウス: ボタンクリック
// =============================================================================

#[test]
fn mouse_click_stage_all_button() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(1, 10, 12, 1),
        ButtonAction::StageAll,
    ));
    let ev = mouse_click(5, 10); // within button rect
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::StageAll)
    );
}

#[test]
fn mouse_click_commit_button() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(14, 10, 10, 1),
        ButtonAction::Commit,
    ));
    let ev = mouse_click(16, 10);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::EnterCommitMode)
    );
}

#[test]
fn mouse_click_push_button() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(25, 10, 8, 1),
        ButtonAction::Push,
    ));
    let ev = mouse_click(28, 10);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Push));
}

#[test]
fn mouse_click_outside_buttons() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(1, 10, 12, 1),
        ButtonAction::StageAll,
    ));
    let ev = mouse_click(50, 10); // outside button
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), None);
}

// =============================================================================
// マウス: ファイル行クリック
// =============================================================================

#[test]
fn mouse_click_file_row() {
    let mut areas = ClickAreas::new();
    areas.file_rows.push((Rect::new(0, 2, 80, 1), 0));
    areas.file_rows.push((Rect::new(0, 3, 80, 1), 1));
    areas.file_rows.push((Rect::new(0, 4, 80, 1), 2));

    let ev = mouse_click(10, 3); // second row
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::SelectIndex(1))
    );
}

#[test]
fn mouse_click_file_row_first() {
    let mut areas = ClickAreas::new();
    areas.file_rows.push((Rect::new(0, 2, 80, 1), 0));
    let ev = mouse_click(5, 2);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::SelectIndex(0))
    );
}

// =============================================================================
// マウス: CommitInputモードのボタン
// =============================================================================

#[test]
fn mouse_click_confirm_commit_button() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(1, 10, 10, 1),
        ButtonAction::ConfirmCommit,
    ));
    let ev = mouse_click(5, 10);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::ConfirmCommit)
    );
}

#[test]
fn mouse_click_cancel_commit_button() {
    let mut areas = ClickAreas::new();
    areas.buttons.push((
        Rect::new(12, 10, 10, 1),
        ButtonAction::CancelCommit,
    ));
    let ev = mouse_click(15, 10);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::CancelCommit)
    );
}

// =============================================================================
// ボタン優先順位: ボタンがファイル行より先にチェックされる
// =============================================================================

#[test]
fn button_takes_priority_over_file_row() {
    let mut areas = ClickAreas::new();
    // Both occupy the same row
    areas.buttons.push((
        Rect::new(0, 5, 10, 1),
        ButtonAction::StageAll,
    ));
    areas.file_rows.push((Rect::new(0, 5, 80, 1), 0));

    let ev = mouse_click(5, 5);
    // Button should win
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::StageAll)
    );
}

// =============================================================================
// マウス: スクロール
// =============================================================================

fn mouse_scroll_up(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_scroll_down(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn mouse_scroll_up_normal() {
    let areas = empty_click_areas();
    let ev = mouse_scroll_up(10, 5);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::ScrollUp));
}

#[test]
fn mouse_scroll_down_normal() {
    let areas = empty_click_areas();
    let ev = mouse_scroll_down(10, 5);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ScrollDown)
    );
}

#[test]
fn mouse_scroll_up_commit_mode() {
    let areas = empty_click_areas();
    let ev = mouse_scroll_up(10, 5);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::ScrollUp)
    );
}

#[test]
fn mouse_scroll_down_commit_mode() {
    let areas = empty_click_areas();
    let ev = mouse_scroll_down(10, 5);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::ScrollDown)
    );
}
