use pocogit::app::{App, BusyAction, Mode};
use pocogit::event::ClickAreas;
use pocogit::git::{FileStatus, GitFile, RepoState};
use pocogit::ui::truncate_path;
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

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
// truncate_path
// =============================================================================

#[test]
fn truncate_short_path_unchanged() {
    assert_eq!(truncate_path("src/main.rs", 30), "src/main.rs");
}

#[test]
fn truncate_exact_length_unchanged() {
    assert_eq!(truncate_path("abcde", 5), "abcde");
}

#[test]
fn truncate_long_path() {
    let result = truncate_path("very/long/path/to/file.rs", 15);
    assert_eq!(result.len(), 15);
    assert!(result.starts_with(".."));
    assert!(result.ends_with("file.rs"));
}

#[test]
fn truncate_max_width_zero() {
    assert_eq!(truncate_path("anything.rs", 0), "");
}

#[test]
fn truncate_max_width_one() {
    let result = truncate_path("abcdef", 1);
    assert_eq!(result.len(), 1);
}

#[test]
fn truncate_max_width_two() {
    let result = truncate_path("abcdef", 2);
    assert_eq!(result.len(), 2);
}

#[test]
fn truncate_max_width_three() {
    // max_width <= 3 takes last N chars without ".." prefix
    let result = truncate_path("abcdef", 3);
    assert_eq!(result.len(), 3);
    assert_eq!(result, "def");
}

// =============================================================================
// 描画テスト: TestBackendで実際にrenderして内容確認
// =============================================================================

#[test]
fn render_shows_branch_name() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let header_line = buffer_line_to_string(&buf, 0);
    assert!(
        header_line.contains("main"),
        "Header should contain branch name 'main', got: '{}'",
        header_line
    );
}

#[test]
fn render_shows_file_with_status_symbol() {
    let repo = make_repo(vec![
        ("staged.rs", FileStatus::Staged),
        ("modified.rs", FileStatus::Modified),
        ("untracked.rs", FileStatus::Untracked),
    ]);
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    // File list starts at row 2 (after 2-row header)
    let row2 = buffer_line_to_string(&buf, 2);
    let row3 = buffer_line_to_string(&buf, 3);
    let row4 = buffer_line_to_string(&buf, 4);

    assert!(
        row2.contains("+") && row2.contains("staged.rs"),
        "Row 2: '{}'",
        row2
    );
    assert!(
        row3.contains("~") && row3.contains("modified.rs"),
        "Row 3: '{}'",
        row3
    );
    assert!(
        row4.contains("?") && row4.contains("untracked.rs"),
        "Row 4: '{}'",
        row4
    );
}

#[test]
fn render_shows_selected_cursor() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Modified),
    ]);
    let mut app = App::with_repo(repo);
    app.selected_index = 1;
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let row2 = buffer_line_to_string(&buf, 2); // first file, not selected
    let row3 = buffer_line_to_string(&buf, 3); // second file, selected

    assert!(!row2.contains(">"), "First row should not have cursor");
    assert!(
        row3.contains(">"),
        "Second row should have cursor, got: '{}'",
        row3
    );
}

#[test]
fn render_no_changes_message() {
    let repo = make_repo(vec![]);
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let row2 = buffer_line_to_string(&buf, 2);
    assert!(
        row2.contains("No changes"),
        "Should show 'No changes', got: '{}'",
        row2
    );
}

#[test]
fn render_commit_input_mode() {
    let mut repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    repo.staged_count = 1;
    let mut app = App::with_repo(repo);
    app.mode = Mode::CommitInput;
    app.commit_message = "feat: test".to_string();
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let all_text = buffer_all_to_string(&buf);
    assert!(all_text.contains("COMMIT"), "Should show COMMIT title");
    assert!(
        all_text.contains("feat: test"),
        "Should show commit message"
    );
}

#[test]
fn render_click_areas_populated() {
    let repo = make_repo(vec![
        ("a.rs", FileStatus::Modified),
        ("b.rs", FileStatus::Staged),
    ]);
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    // Should have file row click areas
    assert_eq!(click_areas.file_rows.len(), 2);
    assert_eq!(click_areas.file_rows[0].1, 0); // index 0
    assert_eq!(click_areas.file_rows[1].1, 1); // index 1

    // Should have at least 3 buttons (Stage All, Commit, Push)
    assert!(
        click_areas.buttons.len() >= 3,
        "Expected at least 3 buttons, got {}",
        click_areas.buttons.len()
    );
}

#[test]
fn render_header_counts() {
    let repo = RepoState {
        branch: "develop".to_string(),
        files: vec![
            GitFile {
                path: "a.rs".to_string(),
                status: FileStatus::Staged,
            },
            GitFile {
                path: "b.rs".to_string(),
                status: FileStatus::Modified,
            },
            GitFile {
                path: "c.rs".to_string(),
                status: FileStatus::Untracked,
            },
        ],
        staged_count: 1,
        unstaged_count: 1,
        untracked_count: 1,
        unpushed_count: 2,
        unpulled_count: 0,
    };
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let header = buffer_line_to_string(&buf, 0);
    assert!(header.contains("develop"), "Header should show branch name");
    assert!(header.contains("+1"), "Header should show staged count");
    assert!(header.contains("~1"), "Header should show unstaged count");
    assert!(header.contains("?1"), "Header should show untracked count");
    assert!(header.contains("↑2"), "Header should show unpushed count");
}

#[test]
fn render_narrow_width_buttons() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let app = App::with_repo(repo);
    let mut click_areas = ClickAreas::new();

    // Narrow terminal (width < 40)
    let backend = TestBackend::new(30, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let footer = buffer_line_to_string(&buf, 8); // footer area
                                                 // Narrow buttons should be abbreviated
    assert!(
        footer.contains("[SA]") || footer.contains("[C]") || footer.contains("[P]"),
        "Narrow width should show abbreviated buttons, got: '{}'",
        footer
    );
}

#[test]
fn render_busy_footer_shows_loading_label() {
    let repo = make_repo(vec![("a.rs", FileStatus::Modified)]);
    let mut app = App::with_repo(repo);
    app.begin_busy(BusyAction::Push);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let footer = buffer_line_to_string(&buf, 8);
    let status = buffer_line_to_string(&buf, 9);
    assert!(footer.contains("[Pushing...]"), "footer: '{}'", footer);
    assert!(status.contains("Pushing..."), "status: '{}'", status);
}

#[test]
fn render_busy_commit_footer_shows_loading_label() {
    let repo = make_repo(vec![("a.rs", FileStatus::Staged)]);
    let mut app = App::with_repo(repo);
    app.mode = Mode::CommitInput;
    app.begin_busy(BusyAction::Commit);
    let mut click_areas = ClickAreas::new();

    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            pocogit::ui::render(frame, &app, &mut click_areas);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let footer = buffer_line_to_string(&buf, 10);
    let status = buffer_line_to_string(&buf, 11);
    assert!(footer.contains("[Committing...]"), "footer: '{}'", footer);
    assert!(status.contains("Committing..."), "status: '{}'", status);
}

// =============================================================================
// ヘルパー
// =============================================================================

fn buffer_line_to_string(buf: &Buffer, row: u16) -> String {
    let width = buf.area.width;
    (0..width)
        .map(|col| buf.cell((col, row)).map(|c| c.symbol()).unwrap_or(" "))
        .collect::<String>()
}

fn buffer_all_to_string(buf: &Buffer) -> String {
    let height = buf.area.height;
    (0..height)
        .map(|row| buffer_line_to_string(buf, row))
        .collect::<Vec<_>>()
        .join("\n")
}
