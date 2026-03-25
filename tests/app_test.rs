use pocogit::app::{App, Mode};
use pocogit::git::{FileStatus, GitFile, RepoState};

fn make_repo(files: Vec<(&str, FileStatus)>) -> RepoState {
    let git_files: Vec<GitFile> = files
        .iter()
        .map(|(path, status)| GitFile {
            path: path.to_string(),
            status: status.clone(),
        })
        .collect();
    let staged_count = git_files
        .iter()
        .filter(|f| f.status == FileStatus::Staged || f.status == FileStatus::Both)
        .count();
    let unstaged_count = git_files
        .iter()
        .filter(|f| f.status == FileStatus::Modified || f.status == FileStatus::Both)
        .count();
    let untracked_count = git_files
        .iter()
        .filter(|f| f.status == FileStatus::Untracked)
        .count();
    RepoState {
        branch: "main".to_string(),
        files: git_files,
        staged_count,
        unstaged_count,
        untracked_count,
        unpushed_count: 0,
        unpulled_count: 0,
    }
}

// =============================================================================
// 初期状態
// =============================================================================

#[test]
fn with_repo_initial_state() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let app = App::with_repo(repo);
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.list_offset, 0);
    assert!(app.commit_message.is_empty());
    assert!(app.status_message.is_empty());
    assert!(!app.should_quit);
}

#[test]
fn with_repo_empty_files() {
    let repo = make_repo(vec![]);
    let app = App::with_repo(repo);
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.repo.files.len(), 0);
}

// =============================================================================
// カーソル移動
// =============================================================================

#[test]
fn move_down_increments_selected() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    assert_eq!(app.selected_index, 0);

    app.move_down();
    assert_eq!(app.selected_index, 1);

    app.move_down();
    assert_eq!(app.selected_index, 2);
}

#[test]
fn move_down_stops_at_last() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.move_down();
    app.move_down();
    app.move_down(); // should not go beyond 1
    assert_eq!(app.selected_index, 1);
}

#[test]
fn move_up_decrements_selected() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.selected_index = 1;

    app.move_up();
    assert_eq!(app.selected_index, 0);
}

#[test]
fn move_up_stops_at_zero() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let mut app = App::with_repo(repo);
    app.move_up();
    assert_eq!(app.selected_index, 0);
}

#[test]
fn move_down_empty_file_list() {
    let repo = make_repo(vec![]);
    let mut app = App::with_repo(repo);
    app.move_down();
    assert_eq!(app.selected_index, 0);
}

// =============================================================================
// スクロール調整
// =============================================================================

#[test]
fn adjust_scroll_cursor_below_visible() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
        ("d.rs", FileStatus::Modified),
        ("e.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.selected_index = 4; // last item
    app.adjust_scroll(3); // visible height = 3
    // list_offset should be 4 - 3 + 1 = 2
    assert_eq!(app.list_offset, 2);
}

#[test]
fn adjust_scroll_cursor_above_visible() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.list_offset = 2;
    app.selected_index = 0;
    app.adjust_scroll(3);
    assert_eq!(app.list_offset, 0);
}

#[test]
fn adjust_scroll_zero_height() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let mut app = App::with_repo(repo);
    app.list_offset = 5;
    app.adjust_scroll(0);
    // Should not change offset
    assert_eq!(app.list_offset, 5);
}

#[test]
fn adjust_scroll_cursor_within_visible() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.list_offset = 0;
    app.selected_index = 1;
    app.adjust_scroll(3);
    // offset should remain 0 since cursor is visible
    assert_eq!(app.list_offset, 0);
}

// =============================================================================
// コミットモード遷移
// =============================================================================

#[test]
fn enter_commit_mode_with_staged_files() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    assert_eq!(app.mode, Mode::CommitInput);
    assert!(app.commit_message.is_empty());
}

#[test]
fn enter_commit_mode_without_staged_files() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    assert_eq!(app.mode, Mode::Normal); // should stay in Normal
    assert_eq!(app.status_message, "Nothing staged to commit");
}

#[test]
fn enter_commit_mode_empty_repo() {
    let repo = make_repo(vec![]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.status_message, "Nothing staged to commit");
}

#[test]
fn cancel_commit_returns_to_normal() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    app.commit_message = "some msg".to_string();
    app.cancel_commit();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.commit_message.is_empty());
    assert!(app.status_message.is_empty());
}

#[test]
fn confirm_commit_rejects_empty_message() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    // leave commit_message empty
    app.confirm_commit();
    assert_eq!(app.status_message, "Empty commit message");
    // should still be in CommitInput mode
    assert_eq!(app.mode, Mode::CommitInput);
}

#[test]
fn confirm_commit_rejects_whitespace_only() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.enter_commit_mode();
    app.commit_message = "   ".to_string();
    app.confirm_commit();
    assert_eq!(app.status_message, "Empty commit message");
}

#[test]
fn enter_commit_mode_clears_previous_message() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.status_message = "old message".to_string();
    app.commit_message = "leftover".to_string();
    app.enter_commit_mode();
    assert!(app.commit_message.is_empty());
    assert!(app.status_message.is_empty());
}

// =============================================================================
// toggle_stage: トグル動作の状態チェック
// （実git呼び出しが発生するため、ここではロジック分岐のみテスト可能な範囲で）
// =============================================================================

// NOTE: toggle_stage_selected はgitコマンドを呼ぶので、
// 純粋なユニットテストとしては統合テスト側でカバーする。
// ここではAppのメソッドが存在することの確認のみ。
#[test]
fn toggle_stage_method_exists() {
    let repo = make_repo(vec![]);
    let mut app = App::with_repo(repo);
    // Empty file list: should not panic
    app.toggle_stage_selected();
}

// =============================================================================
// マウススクロール（ビューのみ移動、カーソル不変）
// =============================================================================

#[test]
fn scroll_down_increments_offset() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
        ("d.rs", FileStatus::Modified),
        ("e.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.scroll_down(3);
    assert_eq!(app.list_offset, 1);
    assert_eq!(app.selected_index, 0); // cursor unchanged
}

#[test]
fn scroll_down_stops_at_max() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    // visible_height=2, max_offset = 3 - 2 = 1
    app.scroll_down(2);
    app.scroll_down(2);
    app.scroll_down(2);
    assert_eq!(app.list_offset, 1);
}

#[test]
fn scroll_up_decrements_offset() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.list_offset = 2;
    app.scroll_up();
    assert_eq!(app.list_offset, 1);
    assert_eq!(app.selected_index, 0); // cursor unchanged
}

#[test]
fn scroll_up_stops_at_zero() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let mut app = App::with_repo(repo);
    app.scroll_up();
    assert_eq!(app.list_offset, 0);
}

#[test]
fn scroll_does_not_move_cursor() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
        ("c.rs", FileStatus::Modified),
        ("d.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.selected_index = 1;
    app.scroll_down(3);
    assert_eq!(app.selected_index, 1);
    app.scroll_up();
    assert_eq!(app.selected_index, 1);
}

// =============================================================================
// Branch list
// =============================================================================

#[test]
fn branch_list_move_up_stops_at_zero() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.branches = vec!["main".into(), "dev".into()];
    app.branch_selected = 0;
    app.branch_list_move_up();
    assert_eq!(app.branch_selected, 0);
}

#[test]
fn branch_list_move_down_increments() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.branches = vec!["main".into(), "dev".into(), "staging".into()];
    app.branch_selected = 0;
    app.branch_list_move_down();
    assert_eq!(app.branch_selected, 1);
}

#[test]
fn branch_list_move_down_stops_at_last() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.branches = vec!["main".into(), "dev".into()];
    app.branch_selected = 1;
    app.branch_list_move_down();
    assert_eq!(app.branch_selected, 1);
}

#[test]
fn close_branch_list_returns_to_normal() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchList;
    app.branches = vec!["main".into()];
    app.close_branch_list();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.branches.is_empty());
}

#[test]
fn enter_branch_create_mode() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchList;
    app.enter_branch_create();
    assert_eq!(app.mode, Mode::BranchCreate);
    assert!(app.branch_name_input.is_empty());
}

#[test]
fn cancel_branch_create_returns_to_list() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchCreate;
    app.branches = vec!["main".into(), "dev".into()];
    app.branch_name_input = "feature/test".into();
    app.cancel_branch_create();
    assert_eq!(app.mode, Mode::BranchList);
    assert!(app.branch_name_input.is_empty());
}

#[test]
fn cancel_branch_create_returns_to_normal_when_no_branches() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchCreate;
    app.branch_name_input = "feature/test".into();
    app.cancel_branch_create();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.branch_name_input.is_empty());
}

#[test]
fn confirm_branch_create_rejects_empty() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchCreate;
    app.branch_name_input = "   ".into();
    app.confirm_branch_create();
    assert_eq!(app.mode, Mode::BranchCreate);
    assert_eq!(app.status_message, "Empty branch name");
}

#[test]
fn confirm_branch_switch_already_on_current() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.mode = Mode::BranchList;
    app.branches = vec!["main".into(), "dev".into()];
    app.branch_selected = 0; // "main" which matches repo.branch
    app.confirm_branch_switch();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.status_message.contains("Already on"));
}

#[test]
fn adjust_branch_scroll_cursor_below() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.branches = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
    app.branch_selected = 4;
    app.branch_scroll = 0;
    app.adjust_branch_scroll(3);
    assert_eq!(app.branch_scroll, 2);
}

#[test]
fn adjust_branch_scroll_cursor_above() {
    let mut app = App::with_repo(make_repo(vec![]));
    app.branches = vec!["a".into(), "b".into(), "c".into()];
    app.branch_selected = 0;
    app.branch_scroll = 2;
    app.adjust_branch_scroll(3);
    assert_eq!(app.branch_scroll, 0);
}
