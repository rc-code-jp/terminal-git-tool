use pocogit::git::{parse_branch, parse_porcelain_output, FileStatus};

// =============================================================================
// parse_branch
// =============================================================================

#[test]
fn parse_branch_simple() {
    assert_eq!(parse_branch("## main"), "main");
}

#[test]
fn parse_branch_with_upstream() {
    assert_eq!(parse_branch("## main...origin/main"), "main");
}

#[test]
fn parse_branch_with_upstream_and_ahead() {
    assert_eq!(
        parse_branch("## main...origin/main [ahead 2]"),
        "main"
    );
}

#[test]
fn parse_branch_detached_head() {
    let result = parse_branch("## HEAD (no branch)");
    assert!(result.contains("no branch"));
}

#[test]
fn parse_branch_no_commits() {
    let result = parse_branch("## No commits yet on main");
    assert!(result.contains("No commits yet"));
}

#[test]
fn parse_branch_feature_slash() {
    assert_eq!(
        parse_branch("## feature/my-branch...origin/feature/my-branch"),
        "feature/my-branch"
    );
}

// =============================================================================
// parse_porcelain_output: ファイルステータスの判定
// =============================================================================

#[test]
fn porcelain_untracked_file() {
    let output = "## main\n?? README.md\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].path, "README.md");
    assert_eq!(state.files[0].status, FileStatus::Untracked);
}

#[test]
fn porcelain_staged_new_file() {
    let output = "## main\nA  src/new.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Staged);
}

#[test]
fn porcelain_staged_modified() {
    let output = "## main\nM  src/app.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Staged);
}

#[test]
fn porcelain_unstaged_modified() {
    let output = "## main\n M src/app.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Modified);
}

#[test]
fn porcelain_both_staged_and_unstaged() {
    let output = "## main\nMM src/app.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Both);
}

#[test]
fn porcelain_staged_deleted() {
    let output = "## main\nD  old_file.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Staged);
}

#[test]
fn porcelain_unstaged_deleted() {
    let output = "## main\n D old_file.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, FileStatus::Modified);
}

#[test]
fn porcelain_rename() {
    let output = "## main\nR  old.rs -> new.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].path, "new.rs");
    assert_eq!(state.files[0].status, FileStatus::Staged);
}

#[test]
fn porcelain_ignored_files_skipped() {
    let output = "## main\n!! ignored_dir/\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 0);
}

// =============================================================================
// parse_porcelain_output: カウント集計
// =============================================================================

#[test]
fn porcelain_counts_mixed() {
    let output = "\
## main
M  staged.rs
 M unstaged.rs
?? untracked.rs
MM both.rs
A  added.rs
";
    let state = parse_porcelain_output(output, 3, 0);

    // staged: staged.rs, both.rs, added.rs => 3
    assert_eq!(state.staged_count, 3);
    // unstaged: unstaged.rs, both.rs => 2
    assert_eq!(state.unstaged_count, 2);
    // untracked: untracked.rs => 1
    assert_eq!(state.untracked_count, 1);
    assert_eq!(state.unpushed_count, 3);
    assert_eq!(state.files.len(), 5);
}

#[test]
fn porcelain_empty_output() {
    let output = "## main\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.branch, "main");
    assert_eq!(state.files.len(), 0);
    assert_eq!(state.staged_count, 0);
    assert_eq!(state.unstaged_count, 0);
    assert_eq!(state.untracked_count, 0);
}

#[test]
fn porcelain_all_untracked() {
    let output = "\
## main
?? a.rs
?? b.rs
?? c.rs
";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.untracked_count, 3);
    assert_eq!(state.staged_count, 0);
    assert_eq!(state.unstaged_count, 0);
}

#[test]
fn porcelain_path_with_spaces() {
    let output = "## main\n?? path with spaces/file.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files[0].path, "path with spaces/file.rs");
}

#[test]
fn porcelain_short_lines_skipped() {
    // Lines shorter than 4 chars should be safely skipped
    let output = "## main\nXX\n M src/app.rs\n";
    let state = parse_porcelain_output(output, 0, 0);
    assert_eq!(state.files.len(), 1);
}
