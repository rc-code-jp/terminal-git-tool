use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// テスト用の一時gitリポジトリを作成・管理するヘルパー
struct TempRepo {
    dir: PathBuf,
}

impl TempRepo {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("git-pocket-test-{}-{}", name, std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();

        // git init
        Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output()
            .unwrap();
        // Configure user for commits
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&dir)
            .output()
            .unwrap();

        Self { dir }
    }

    fn path(&self) -> &PathBuf {
        &self.dir
    }

    fn write_file(&self, name: &str, content: &str) {
        let path = self.dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn git_status_porcelain(&self) -> String {
        self.git(&["status", "--porcelain=v1", "-b"])
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

// =============================================================================
// parse_porcelain_output + 実git出力
// =============================================================================

#[test]
fn integration_parse_untracked_files() {
    let repo = TempRepo::new("untracked");
    repo.write_file("a.txt", "hello");
    repo.write_file("b.txt", "world");

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 2);
    assert!(state.files.iter().all(|f| f.status == pocogit::git::FileStatus::Untracked));
    assert_eq!(state.untracked_count, 2);
}

#[test]
fn integration_parse_staged_files() {
    let repo = TempRepo::new("staged");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "a.txt"]);

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Staged);
    assert_eq!(state.staged_count, 1);
}

#[test]
fn integration_parse_modified_files() {
    let repo = TempRepo::new("modified");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "a.txt"]);
    repo.git(&["commit", "-m", "init"]);
    repo.write_file("a.txt", "hello modified");

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Modified);
    assert_eq!(state.unstaged_count, 1);
}

#[test]
fn integration_parse_both_status() {
    let repo = TempRepo::new("both");
    repo.write_file("a.txt", "v1");
    repo.git(&["add", "a.txt"]);
    repo.git(&["commit", "-m", "init"]);
    repo.write_file("a.txt", "v2");
    repo.git(&["add", "a.txt"]);
    repo.write_file("a.txt", "v3"); // modify again without staging

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Both);
    assert_eq!(state.staged_count, 1);
    assert_eq!(state.unstaged_count, 1);
}

#[test]
fn integration_parse_mixed_statuses() {
    let repo = TempRepo::new("mixed");
    // Create initial commit
    repo.write_file("initial.txt", "init");
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "init"]);

    // Staged new file
    repo.write_file("new.txt", "new");
    repo.git(&["add", "new.txt"]);

    // Modified existing file (unstaged)
    repo.write_file("initial.txt", "modified");

    // Untracked file
    repo.write_file("untracked.txt", "untracked");

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 3);
    assert_eq!(state.staged_count, 1);
    assert_eq!(state.unstaged_count, 1);
    assert_eq!(state.untracked_count, 1);
}

#[test]
fn integration_parse_empty_repo() {
    let repo = TempRepo::new("empty");

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 0);
    assert_eq!(state.staged_count, 0);
    assert_eq!(state.unstaged_count, 0);
    assert_eq!(state.untracked_count, 0);
}

#[test]
fn integration_parse_deleted_file() {
    let repo = TempRepo::new("deleted");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "a.txt"]);
    repo.git(&["commit", "-m", "init"]);
    fs::remove_file(repo.path().join("a.txt")).unwrap();

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 1);
    // Deleted but not staged = Modified (unstaged change)
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Modified);
}

#[test]
fn integration_parse_staged_delete() {
    let repo = TempRepo::new("staged-del");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "a.txt"]);
    repo.git(&["commit", "-m", "init"]);
    repo.git(&["rm", "a.txt"]);

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 1);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Staged);
}

#[test]
fn integration_parse_subdirectory_files() {
    let repo = TempRepo::new("subdir");
    repo.write_file("src/main.rs", "fn main() {}");
    repo.write_file("src/lib.rs", "pub mod lib;");

    // Use -uall to show individual untracked files instead of directories
    let porcelain = repo.git(&["status", "--porcelain=v1", "-b", "-uall"]);
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);

    assert_eq!(state.files.len(), 2);
    let paths: Vec<&str> = state.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/main.rs"));
    assert!(paths.contains(&"src/lib.rs"));
}

// =============================================================================
// git操作: stage / unstage / commit
// =============================================================================

#[test]
fn integration_stage_and_unstage_file() {
    let repo = TempRepo::new("stage-unstage");
    repo.write_file("a.txt", "hello");

    // Stage
    let output = Command::new("git")
        .args(["add", "--", "a.txt"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Staged);

    // Unstage (using reset HEAD will fail on first commit, use rm --cached)
    let output = Command::new("git")
        .args(["rm", "--cached", "a.txt"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    assert_eq!(state.files[0].status, pocogit::git::FileStatus::Untracked);
}

#[test]
fn integration_commit_flow() {
    let repo = TempRepo::new("commit");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "a.txt"]);

    let output = Command::new("git")
        .args(["commit", "-m", "test commit"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    assert_eq!(state.files.len(), 0, "No files after commit");
}

#[test]
fn integration_stage_all() {
    let repo = TempRepo::new("stage-all");
    repo.write_file("a.txt", "hello");
    repo.write_file("b.txt", "world");
    repo.write_file("src/c.txt", "nested");

    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    assert!(
        state.files.iter().all(|f| f.status == pocogit::git::FileStatus::Staged),
        "All files should be staged"
    );
    assert_eq!(state.staged_count, 3);
}

// =============================================================================
// ブランチ名取得: 実git
// =============================================================================

#[test]
fn integration_branch_name_default() {
    let repo = TempRepo::new("branch-default");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "init"]);

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    // Default branch could be "main" or "master" depending on git config
    assert!(
        !state.branch.is_empty(),
        "Branch name should not be empty"
    );
}

#[test]
fn integration_branch_name_feature() {
    let repo = TempRepo::new("branch-feature");
    repo.write_file("a.txt", "hello");
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "init"]);
    repo.git(&["checkout", "-b", "feature/test-branch"]);

    let porcelain = repo.git_status_porcelain();
    let state = pocogit::git::parse_porcelain_output(&porcelain, 0, 0);
    assert_eq!(state.branch, "feature/test-branch");
}
