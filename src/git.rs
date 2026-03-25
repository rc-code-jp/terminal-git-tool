use anyhow::{Context, Result};
use std::process::Command;

#[derive(Clone, Debug, PartialEq)]
pub enum FileStatus {
    Staged,
    Modified,
    Both,
    Untracked,
}

#[derive(Clone, Debug)]
pub struct GitFile {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Clone, Debug)]
pub struct RepoState {
    pub branch: String,
    pub files: Vec<GitFile>,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub unpushed_count: usize,
    pub unpulled_count: usize,
}

pub fn fetch() {
    let _ = Command::new("git")
        .args(["fetch"])
        .output();
}

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_repo_state() -> Result<RepoState> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "-b", "-uall"])
        .output()
        .context("Failed to run git status")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let unpushed_count = get_unpushed_count();
    let unpulled_count = get_unpulled_count();
    Ok(parse_porcelain_output(&stdout, unpushed_count, unpulled_count))
}

pub fn parse_porcelain_output(stdout: &str, unpushed_count: usize, unpulled_count: usize) -> RepoState {
    let mut branch = String::from("HEAD");
    let mut files = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("## ") {
            branch = parse_branch(line);
            continue;
        }
        if line.len() < 4 {
            continue;
        }
        let x = line.as_bytes()[0];
        let y = line.as_bytes()[1];
        let path = line[3..].to_string();
        // Handle renames: "R  old -> new"
        let path = if let Some(pos) = path.find(" -> ") {
            path[pos + 4..].to_string()
        } else {
            path
        };

        let status = match (x, y) {
            (b'?', b'?') => FileStatus::Untracked,
            (b'!', b'!') => continue, // ignored
            (_, b' ') | (_, b'\t') => FileStatus::Staged,
            (b' ', _) => FileStatus::Modified,
            _ => {
                // Both index and worktree have changes
                if x != b' ' && x != b'?' && y != b' ' && y != b'?' {
                    FileStatus::Both
                } else if x != b' ' && x != b'?' {
                    FileStatus::Staged
                } else {
                    FileStatus::Modified
                }
            }
        };

        files.push(GitFile { path, status });
    }

    let staged_count = files
        .iter()
        .filter(|f| f.status == FileStatus::Staged || f.status == FileStatus::Both)
        .count();
    let unstaged_count = files
        .iter()
        .filter(|f| f.status == FileStatus::Modified || f.status == FileStatus::Both)
        .count();
    let untracked_count = files
        .iter()
        .filter(|f| f.status == FileStatus::Untracked)
        .count();

    RepoState {
        branch,
        files,
        staged_count,
        unstaged_count,
        untracked_count,
        unpushed_count,
        unpulled_count,
    }
}

pub fn parse_branch(line: &str) -> String {
    // "## main...origin/main" or "## main" or "## HEAD (no branch)"
    let rest = &line[3..];
    if let Some(pos) = rest.find("...") {
        rest[..pos].to_string()
    } else if rest.contains("(no branch)") || rest.contains("No commits yet") {
        rest.to_string()
    } else {
        rest.split_whitespace().next().unwrap_or("HEAD").to_string()
    }
}

fn get_unpulled_count() -> usize {
    Command::new("git")
        .args(["rev-list", "HEAD..@{upstream}", "--count"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<usize>()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn get_unpushed_count() -> usize {
    Command::new("git")
        .args(["rev-list", "@{upstream}..HEAD", "--count"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<usize>()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

pub fn stage_file(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["add", "--", path])
        .output()
        .context("Failed to run git add")?;
    if !output.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub fn unstage_file(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", path])
        .output()
        .context("Failed to run git reset")?;
    if !output.status.success() {
        anyhow::bail!(
            "git reset failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub fn stage_all() -> Result<()> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .output()
        .context("Failed to run git add -A")?;
    if !output.status.success() {
        anyhow::bail!(
            "git add -A failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub fn unstage_all() -> Result<()> {
    let output = Command::new("git")
        .args(["reset", "HEAD"])
        .output()
        .context("Failed to run git reset HEAD")?;
    if !output.status.success() {
        // git reset HEAD on initial commit may fail, try alternative
        let output2 = Command::new("git")
            .args(["rm", "--cached", "-r", "."])
            .output()
            .context("Failed to run git rm --cached")?;
        if !output2.status.success() {
            anyhow::bail!(
                "git unstage all failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    Ok(())
}

pub fn commit(message: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to run git commit")?;
    if !output.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("Committed")
        .to_string())
}

pub fn pull() -> Result<String> {
    let output = Command::new("git")
        .args(["pull"])
        .output()
        .context("Failed to run git pull")?;
    if !output.status.success() {
        anyhow::bail!(
            "git pull failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let msg = if stderr.is_empty() {
        stdout.to_string()
    } else {
        stderr.to_string()
    };
    Ok(msg.lines().last().unwrap_or("Pulled").to_string())
}

pub fn get_branches() -> Result<(Vec<String>, usize)> {
    let output = Command::new("git")
        .args(["branch", "--list", "--no-color"])
        .output()
        .context("Failed to run git branch")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_branch_list(&stdout))
}

pub fn parse_branch_list(stdout: &str) -> (Vec<String>, usize) {
    let mut branches = Vec::new();
    let mut current_index = 0;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("* ") {
            current_index = branches.len();
            branches.push(name.to_string());
        } else {
            branches.push(trimmed.to_string());
        }
    }
    (branches, current_index)
}

pub fn checkout_branch(name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["checkout", name])
        .output()
        .context("Failed to run git checkout")?;
    if !output.status.success() {
        anyhow::bail!(
            "git checkout failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr
        .lines()
        .next()
        .unwrap_or("Switched branch")
        .to_string())
}

pub fn create_and_checkout_branch(name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["checkout", "-b", name])
        .output()
        .context("Failed to run git checkout -b")?;
    if !output.status.success() {
        anyhow::bail!(
            "git checkout -b failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr
        .lines()
        .next()
        .unwrap_or("Created branch")
        .to_string())
}

pub fn push() -> Result<String> {
    let output = Command::new("git")
        .args(["push", "origin", "HEAD"])
        .output()
        .context("Failed to run git push")?;
    if !output.status.success() {
        anyhow::bail!(
            "git push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let msg = if stderr.is_empty() {
        stdout.to_string()
    } else {
        stderr.to_string()
    };
    Ok(msg.lines().last().unwrap_or("Pushed").to_string())
}
