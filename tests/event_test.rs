use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Rect;

use pocogit::app::Mode;
use pocogit::event::{map_busy_event, map_event, Action, BranchRow, ButtonAction, ClickAreas};

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
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::MoveDown)
    );
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
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::MoveDown)
    );
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
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Refresh));
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
    areas
        .buttons
        .push((Rect::new(1, 10, 12, 1), ButtonAction::StageAll));
    let ev = mouse_click(5, 10); // within button rect
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::StageAll)
    );
}

#[test]
fn mouse_click_commit_button() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(14, 10, 10, 1), ButtonAction::Commit));
    let ev = mouse_click(16, 10);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::EnterCommitMode)
    );
}

#[test]
fn mouse_click_push_button() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(25, 10, 8, 1), ButtonAction::Push));
    let ev = mouse_click(28, 10);
    assert_eq!(map_event(&ev, &Mode::Normal, &areas), Some(Action::Push));
}

#[test]
fn mouse_click_outside_buttons() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(1, 10, 12, 1), ButtonAction::StageAll));
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
    areas
        .buttons
        .push((Rect::new(1, 10, 10, 1), ButtonAction::ConfirmCommit));
    let ev = mouse_click(5, 10);
    assert_eq!(
        map_event(&ev, &Mode::CommitInput, &areas),
        Some(Action::ConfirmCommit)
    );
}

#[test]
fn mouse_click_cancel_commit_button() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(12, 10, 10, 1), ButtonAction::CancelCommit));
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
    areas
        .buttons
        .push((Rect::new(0, 5, 10, 1), ButtonAction::StageAll));
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
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ScrollUp)
    );
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

// =============================================================================
// BranchList mode
// =============================================================================

#[test]
fn branch_list_q_closes() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('q'));
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::CloseBranchList)
    );
}

#[test]
fn branch_list_esc_closes() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Esc);
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::CloseBranchList)
    );
}

#[test]
fn branch_list_j_moves_down() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('j'));
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::BranchListMoveDown)
    );
}

#[test]
fn branch_list_k_moves_up() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('k'));
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::BranchListMoveUp)
    );
}

#[test]
fn branch_list_enter_selects() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Enter);
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::BranchListSelect)
    );
}

#[test]
fn branch_list_n_creates() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('n'));
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::EnterBranchCreate)
    );
}

#[test]
fn branch_list_ctrl_c_quits() {
    let areas = empty_click_areas();
    let ev = key_event_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::Quit)
    );
}

#[test]
fn normal_b_shows_branch_list() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('b'));
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ShowBranchList)
    );
}

// =============================================================================
// BranchCreate mode
// =============================================================================

#[test]
fn branch_create_enter_confirms() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Enter);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::ConfirmBranchCreate)
    );
}

#[test]
fn branch_create_esc_cancels() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Esc);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::CancelBranchCreate)
    );
}

#[test]
fn branch_create_ctrl_c_cancels() {
    let areas = empty_click_areas();
    let ev = key_event_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::CancelBranchCreate)
    );
}

#[test]
fn branch_create_char_input() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Char('f'));
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::BranchInputChar('f'))
    );
}

#[test]
fn branch_create_backspace() {
    let areas = empty_click_areas();
    let ev = key_event(KeyCode::Backspace);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::BranchInputBackspace)
    );
}

// =============================================================================
// BranchList mouse
// =============================================================================

#[test]
fn branch_list_mouse_click_selects_row() {
    let mut areas = ClickAreas::new();
    areas.branch_rows.push(BranchRow {
        rect: Rect::new(0, 2, 40, 1),
        index: 0,
    });
    areas.branch_rows.push(BranchRow {
        rect: Rect::new(0, 3, 40, 1),
        index: 1,
    });
    let ev = mouse_click(5, 3);
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::BranchSelectIndex(1))
    );
}

#[test]
fn branch_list_mouse_click_outside_returns_none() {
    let mut areas = ClickAreas::new();
    areas.branch_rows.push(BranchRow {
        rect: Rect::new(0, 2, 40, 1),
        index: 0,
    });
    let ev = mouse_click(5, 10);
    assert_eq!(map_event(&ev, &Mode::BranchList, &areas), None);
}

#[test]
fn branch_list_mouse_scroll() {
    let areas = empty_click_areas();
    let ev_up = mouse_scroll_up(10, 5);
    assert_eq!(
        map_event(&ev_up, &Mode::BranchList, &areas),
        Some(Action::ScrollUp)
    );
    let ev_down = mouse_scroll_down(10, 5);
    assert_eq!(
        map_event(&ev_down, &Mode::BranchList, &areas),
        Some(Action::ScrollDown)
    );
}

// =============================================================================
// Header branch name click
// =============================================================================

#[test]
fn header_branch_click_shows_branch_list() {
    let mut areas = ClickAreas::new();
    // ブランチ名がヘッダー左端にある想定
    areas
        .buttons
        .push((Rect::new(0, 0, 4, 1), ButtonAction::ShowBranchList));
    let ev = mouse_click(1, 0);
    assert_eq!(
        map_event(&ev, &Mode::Normal, &areas),
        Some(Action::ShowBranchList)
    );
}

#[test]
fn branch_list_new_button_click() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(1, 20, 7, 1), ButtonAction::EnterBranchCreate));
    let ev = mouse_click(3, 20);
    assert_eq!(
        map_event(&ev, &Mode::BranchList, &areas),
        Some(Action::EnterBranchCreate)
    );
}

#[test]
fn branch_create_create_button_click() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(1, 20, 10, 1), ButtonAction::ConfirmBranchCreate));
    let ev = mouse_click(5, 20);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::ConfirmBranchCreate)
    );
}

#[test]
fn branch_create_cancel_button_click() {
    let mut areas = ClickAreas::new();
    areas
        .buttons
        .push((Rect::new(12, 20, 10, 1), ButtonAction::CancelBranchCreate));
    let ev = mouse_click(15, 20);
    assert_eq!(
        map_event(&ev, &Mode::BranchCreate, &areas),
        Some(Action::CancelBranchCreate)
    );
}

#[test]
fn busy_mode_ctrl_c_quits() {
    let ev = key_event_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(map_busy_event(&ev), Some(Action::Quit));
}

#[test]
fn busy_mode_resize_allowed() {
    let ev = Event::Resize(80, 24);
    assert_eq!(map_busy_event(&ev), Some(Action::Resize));
}

#[test]
fn busy_mode_regular_key_is_ignored() {
    let ev = key_event(KeyCode::Char('p'));
    assert_eq!(map_busy_event(&ev), None);
}

#[test]
fn busy_mode_mouse_click_is_ignored() {
    let ev = mouse_click(2, 2);
    assert_eq!(map_busy_event(&ev), None);
}
