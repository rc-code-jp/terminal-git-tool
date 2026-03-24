use crate::git::{self, FileStatus, RepoState};

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    CommitInput,
}

pub struct App {
    pub mode: Mode,
    pub repo: RepoState,
    pub selected_index: usize,
    pub list_offset: usize,
    pub commit_message: String,
    pub status_message: String,
    pub should_quit: bool,
}

impl App {
    pub fn with_repo(repo: RepoState) -> Self {
        Self {
            mode: Mode::Normal,
            repo,
            selected_index: 0,
            list_offset: 0,
            commit_message: String::new(),
            status_message: String::new(),
            should_quit: false,
        }
    }

    pub fn new() -> Self {
        let repo = git::get_repo_state().unwrap_or(RepoState {
            branch: String::from("???"),
            files: Vec::new(),
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            unpushed_count: 0,
        });
        Self {
            mode: Mode::Normal,
            repo,
            selected_index: 0,
            list_offset: 0,
            commit_message: String::new(),
            status_message: String::new(),
            should_quit: false,
        }
    }

    pub fn refresh(&mut self) {
        let old_selected = self.selected_index;
        match git::get_repo_state() {
            Ok(repo) => {
                self.repo = repo;
                if self.selected_index >= self.repo.files.len() && !self.repo.files.is_empty() {
                    self.selected_index = self.repo.files.len() - 1;
                } else if self.repo.files.is_empty() {
                    self.selected_index = 0;
                } else {
                    self.selected_index = old_selected.min(self.repo.files.len() - 1);
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.repo.files.is_empty() && self.selected_index < self.repo.files.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn toggle_stage_selected(&mut self) {
        if let Some(file) = self.repo.files.get(self.selected_index) {
            let result = match file.status {
                FileStatus::Staged => git::unstage_file(&file.path),
                FileStatus::Modified | FileStatus::Untracked | FileStatus::Both => {
                    git::stage_file(&file.path)
                }
            };
            match result {
                Ok(()) => self.refresh(),
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    pub fn stage_all(&mut self) {
        match git::stage_all() {
            Ok(()) => {
                self.status_message = String::from("Staged all");
                self.refresh();
            }
            Err(e) => self.status_message = format!("Error: {}", e),
        }
    }

    pub fn unstage_all(&mut self) {
        match git::unstage_all() {
            Ok(()) => {
                self.status_message = String::from("Unstaged all");
                self.refresh();
            }
            Err(e) => self.status_message = format!("Error: {}", e),
        }
    }

    pub fn enter_commit_mode(&mut self) {
        if self.repo.staged_count == 0 {
            self.status_message = String::from("Nothing staged to commit");
            return;
        }
        self.mode = Mode::CommitInput;
        self.commit_message.clear();
        self.status_message.clear();
    }

    pub fn confirm_commit(&mut self) {
        if self.commit_message.trim().is_empty() {
            self.status_message = String::from("Empty commit message");
            return;
        }
        match git::commit(&self.commit_message) {
            Ok(msg) => {
                self.status_message = msg;
                self.mode = Mode::Normal;
                self.commit_message.clear();
                self.refresh();
            }
            Err(e) => self.status_message = format!("Error: {}", e),
        }
    }

    pub fn cancel_commit(&mut self) {
        self.mode = Mode::Normal;
        self.commit_message.clear();
        self.status_message.clear();
    }

    pub fn push(&mut self) {
        self.status_message = String::from("Pushing...");
        match git::push() {
            Ok(msg) => {
                self.status_message = msg;
                self.refresh();
            }
            Err(e) => self.status_message = format!("Error: {}", e),
        }
    }

    pub fn scroll_up(&mut self) {
        if self.list_offset > 0 {
            self.list_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_offset = self.repo.files.len().saturating_sub(visible_height);
        if self.list_offset < max_offset {
            self.list_offset += 1;
        }
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_index < self.list_offset {
            self.list_offset = self.selected_index;
        } else if self.selected_index >= self.list_offset + visible_height {
            self.list_offset = self.selected_index - visible_height + 1;
        }
    }
}
