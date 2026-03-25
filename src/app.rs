use crate::git::{self, FileStatus, RepoState};

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    CommitInput,
    Help,
    BranchList,
    BranchCreate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BusyAction {
    StageAll,
    UnstageAll,
    Commit,
    Pull,
    Push,
    BranchSwitch,
    BranchCreate,
}

impl BusyAction {
    pub fn status_text(&self) -> &'static str {
        match self {
            BusyAction::StageAll => "Staging all...",
            BusyAction::UnstageAll => "Unstaging all...",
            BusyAction::Commit => "Committing...",
            BusyAction::Pull => "Pulling...",
            BusyAction::Push => "Pushing...",
            BusyAction::BranchSwitch => "Switching branch...",
            BusyAction::BranchCreate => "Creating branch...",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BusyState {
    pub action: BusyAction,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppCommand {
    StageAll,
    UnstageAll,
    Commit { message: String },
    Pull,
    Push,
    CheckoutBranch { name: String },
    CreateBranch { name: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandRequest {
    pub busy_action: BusyAction,
    pub command: AppCommand,
}

pub struct App {
    pub mode: Mode,
    pub repo: RepoState,
    pub selected_index: usize,
    pub list_offset: usize,
    pub help_scroll: usize,
    pub commit_message: String,
    pub status_message: String,
    pub should_quit: bool,
    pub dirty: bool,
    pub branches: Vec<String>,
    pub branch_selected: usize,
    pub branch_scroll: usize,
    pub branch_name_input: String,
    pub busy: Option<BusyState>,
}

impl App {
    /// Test helper: create App with a given RepoState (used by integration tests)
    #[allow(dead_code)]
    pub fn with_repo(repo: RepoState) -> Self {
        Self {
            mode: Mode::Normal,
            repo,
            selected_index: 0,
            list_offset: 0,
            help_scroll: 0,
            commit_message: String::new(),
            status_message: String::new(),
            should_quit: false,
            dirty: true,
            branches: Vec::new(),
            branch_selected: 0,
            branch_scroll: 0,
            branch_name_input: String::new(),
            busy: None,
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
            unpulled_count: 0,
        });
        Self {
            mode: Mode::Normal,
            repo,
            selected_index: 0,
            list_offset: 0,
            help_scroll: 0,
            commit_message: String::new(),
            status_message: String::new(),
            should_quit: false,
            dirty: true,
            branches: Vec::new(),
            branch_selected: 0,
            branch_scroll: 0,
            branch_name_input: String::new(),
            busy: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.busy.is_some()
    }

    pub fn begin_busy(&mut self, action: BusyAction) {
        let message = action.status_text().to_string();
        self.status_message = message.clone();
        self.busy = Some(BusyState { action, message });
        self.dirty = true;
    }

    pub fn finish_busy_success(&mut self, action: BusyAction, message: String) {
        self.busy = None;
        self.status_message = message;
        match action {
            BusyAction::Commit => {
                self.mode = Mode::Normal;
                self.commit_message.clear();
            }
            BusyAction::BranchSwitch => {
                self.mode = Mode::Normal;
                self.branches.clear();
            }
            BusyAction::BranchCreate => {
                self.mode = Mode::Normal;
                self.branch_name_input.clear();
                self.branches.clear();
            }
            BusyAction::StageAll | BusyAction::UnstageAll | BusyAction::Pull | BusyAction::Push => {
            }
        }
        self.refresh();
    }

    pub fn finish_busy_error(&mut self, _action: BusyAction, error: String) {
        self.busy = None;
        self.status_message = format!("Error: {}", error);
        self.dirty = true;
    }

    pub fn refresh(&mut self) {
        self.dirty = true;
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
            self.dirty = true;
        }
    }

    pub fn move_down(&mut self) {
        if !self.repo.files.is_empty() && self.selected_index < self.repo.files.len() - 1 {
            self.selected_index += 1;
            self.dirty = true;
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

    pub fn stage_all(&mut self) -> Option<CommandRequest> {
        Some(CommandRequest {
            busy_action: BusyAction::StageAll,
            command: AppCommand::StageAll,
        })
    }

    pub fn unstage_all(&mut self) -> Option<CommandRequest> {
        Some(CommandRequest {
            busy_action: BusyAction::UnstageAll,
            command: AppCommand::UnstageAll,
        })
    }

    pub fn enter_commit_mode(&mut self) {
        if self.repo.staged_count == 0 {
            self.status_message = String::from("Nothing staged to commit");
            self.dirty = true;
            return;
        }
        self.mode = Mode::CommitInput;
        self.commit_message.clear();
        self.status_message.clear();
        self.dirty = true;
    }

    pub fn confirm_commit(&mut self) -> Option<CommandRequest> {
        if self.commit_message.trim().is_empty() {
            self.status_message = String::from("Empty commit message");
            self.dirty = true;
            return None;
        }
        Some(CommandRequest {
            busy_action: BusyAction::Commit,
            command: AppCommand::Commit {
                message: self.commit_message.clone(),
            },
        })
    }

    pub fn cancel_commit(&mut self) {
        self.mode = Mode::Normal;
        self.commit_message.clear();
        self.status_message.clear();
        self.dirty = true;
    }

    pub fn pull(&mut self) -> Option<CommandRequest> {
        Some(CommandRequest {
            busy_action: BusyAction::Pull,
            command: AppCommand::Pull,
        })
    }

    pub fn push(&mut self) -> Option<CommandRequest> {
        Some(CommandRequest {
            busy_action: BusyAction::Push,
            command: AppCommand::Push,
        })
    }

    pub fn show_help(&mut self) {
        self.mode = Mode::Help;
        self.help_scroll = 0;
        self.dirty = true;
    }

    pub fn close_help(&mut self) {
        self.mode = Mode::Normal;
        self.dirty = true;
    }

    pub fn help_scroll_up(&mut self) {
        if self.help_scroll > 0 {
            self.help_scroll -= 1;
            self.dirty = true;
        }
    }

    pub fn help_scroll_down(&mut self, total_lines: usize, visible_height: usize) {
        let max_offset = total_lines.saturating_sub(visible_height);
        if self.help_scroll < max_offset {
            self.help_scroll += 1;
            self.dirty = true;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.list_offset > 0 {
            self.list_offset -= 1;
            self.dirty = true;
        }
    }

    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_offset = self.repo.files.len().saturating_sub(visible_height);
        if self.list_offset < max_offset {
            self.list_offset += 1;
            self.dirty = true;
        }
    }

    pub fn show_branch_list(&mut self) {
        match git::get_branches() {
            Ok((branches, current_index)) => {
                self.branches = branches;
                self.branch_selected = current_index;
                self.branch_scroll = 0;
                self.mode = Mode::BranchList;
                self.dirty = true;
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.dirty = true;
            }
        }
    }

    pub fn close_branch_list(&mut self) {
        self.mode = Mode::Normal;
        self.branches.clear();
        self.dirty = true;
    }

    pub fn branch_list_move_up(&mut self) {
        if self.branch_selected > 0 {
            self.branch_selected -= 1;
            self.dirty = true;
        }
    }

    pub fn branch_list_move_down(&mut self) {
        if !self.branches.is_empty() && self.branch_selected < self.branches.len() - 1 {
            self.branch_selected += 1;
            self.dirty = true;
        }
    }

    pub fn branch_list_scroll_up(&mut self) {
        if self.branch_scroll > 0 {
            self.branch_scroll -= 1;
            self.dirty = true;
        }
    }

    pub fn branch_list_scroll_down(&mut self, visible_height: usize) {
        let max_offset = self.branches.len().saturating_sub(visible_height);
        if self.branch_scroll < max_offset {
            self.branch_scroll += 1;
            self.dirty = true;
        }
    }

    pub fn adjust_branch_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.branch_selected < self.branch_scroll {
            self.branch_scroll = self.branch_selected;
        } else if self.branch_selected >= self.branch_scroll + visible_height {
            self.branch_scroll = self.branch_selected - visible_height + 1;
        }
    }

    pub fn confirm_branch_switch(&mut self) -> Option<CommandRequest> {
        if let Some(name) = self.branches.get(self.branch_selected) {
            if *name == self.repo.branch {
                self.status_message = format!("Already on '{}'", name);
                self.mode = Mode::Normal;
                self.branches.clear();
                self.dirty = true;
                return None;
            }
            let name = name.clone();
            return Some(CommandRequest {
                busy_action: BusyAction::BranchSwitch,
                command: AppCommand::CheckoutBranch { name },
            });
        }
        None
    }

    pub fn enter_branch_create(&mut self) {
        self.mode = Mode::BranchCreate;
        self.branch_name_input.clear();
        self.status_message.clear();
        self.dirty = true;
    }

    pub fn confirm_branch_create(&mut self) -> Option<CommandRequest> {
        if self.branch_name_input.trim().is_empty() {
            self.status_message = String::from("Empty branch name");
            self.dirty = true;
            return None;
        }
        Some(CommandRequest {
            busy_action: BusyAction::BranchCreate,
            command: AppCommand::CreateBranch {
                name: self.branch_name_input.trim().to_string(),
            },
        })
    }

    pub fn cancel_branch_create(&mut self) {
        if self.branches.is_empty() {
            self.mode = Mode::Normal;
        } else {
            self.mode = Mode::BranchList;
        }
        self.branch_name_input.clear();
        self.dirty = true;
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
