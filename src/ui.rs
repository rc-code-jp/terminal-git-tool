use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Mode};
use crate::event::{ButtonAction, ClickAreas};
use crate::git::FileStatus;

pub fn render(frame: &mut Frame, app: &App, click_areas: &mut ClickAreas) {
    click_areas.file_rows.clear();
    click_areas.buttons.clear();
    click_areas.branch_rows.clear();

    let area = frame.area();
    let width = area.width as usize;

    let chunks = Layout::vertical([
        Constraint::Length(2), // header
        Constraint::Min(1),    // file list
        Constraint::Length(2), // footer (buttons + status)
    ])
    .split(area);

    render_header(frame, app, chunks[0], width, click_areas);

    match app.mode {
        Mode::Normal => {
            render_file_list(frame, app, chunks[1], click_areas);
            render_footer_normal(frame, app, chunks[2], width, click_areas);
        }
        Mode::CommitInput => {
            render_commit_input(frame, app, chunks[1]);
            render_footer_commit(frame, app, chunks[2], width, click_areas);
        }
        Mode::Help => {
            render_help(frame, chunks[1], app.help_scroll);
            render_footer_help(frame, chunks[2]);
        }
        Mode::BranchList => {
            render_branch_list(frame, app, chunks[1], click_areas);
            render_footer_branch_list(frame, chunks[2], click_areas);
        }
        Mode::BranchCreate => {
            render_branch_create(frame, app, chunks[1]);
            render_footer_branch_create(frame, chunks[2], click_areas);
        }
    }
}

fn render_header(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    width: usize,
    click_areas: &mut ClickAreas,
) {
    let branch = &app.repo.branch;
    let s = app.repo.staged_count;
    let m = app.repo.unstaged_count;
    let u = app.repo.untracked_count;
    let p = app.repo.unpushed_count;
    let d = app.repo.unpulled_count;

    let header_text = if width >= 60 {
        let mut parts = format!("  {}  +{} ~{} ?{}", branch, s, m, u);
        if p > 0 {
            parts.push_str(&format!(" ↑{}", p));
        }
        if d > 0 {
            parts.push_str(&format!(" ↓{}", d));
        }
        parts
    } else if width >= 40 {
        let mut parts = format!("  {} +{}~{}?{}", branch, s, m, u);
        if p > 0 {
            parts.push_str(&format!(" ↑{}", p));
        }
        if d > 0 {
            parts.push_str(&format!(" ↓{}", d));
        }
        parts
    } else {
        let mut parts = format!("  {} +{}~{}", branch, s, m);
        if p > 0 {
            parts.push_str(&format!(" ↑{}", p));
        }
        if d > 0 {
            parts.push_str(&format!(" ↓{}", d));
        }
        parts
    };

    // Register [+] button click area at the start
    let plus_text = " [+]";
    let plus_x = area.x;
    let plus_width = plus_text.len() as u16;
    click_areas.buttons.push((
        Rect::new(plus_x, area.y, plus_width, 1),
        ButtonAction::EnterBranchCreate,
    ));

    // Register branch name click area (after "[+]" + "  " prefix in header_text)
    let branch_x = area.x + plus_width + 2; // "[+]" then "  " then branch name
    let branch_width = branch.len() as u16;
    click_areas.buttons.push((
        Rect::new(branch_x, area.y, branch_width, 1),
        ButtonAction::ShowBranchList,
    ));

    let header = Paragraph::new(Line::from(vec![
        Span::styled(plus_text, Style::default().fg(Color::DarkGray)),
        Span::styled(header_text, Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect, click_areas: &mut ClickAreas) {
    let visible_height = area.height as usize;
    let width = area.width as usize;

    if app.repo.files.is_empty() {
        let msg = Paragraph::new("  No changes");
        frame.render_widget(msg, area);
        return;
    }

    let mut lines = Vec::new();
    let start = app.list_offset;
    let end = (start + visible_height).min(app.repo.files.len());

    for i in start..end {
        let file = &app.repo.files[i];
        let is_selected = i == app.selected_index;

        let (symbol, symbol_color) = match file.status {
            FileStatus::Staged => ("+", Color::Green),
            FileStatus::Modified => ("~", Color::Red),
            FileStatus::Both => ("±", Color::Yellow),
            FileStatus::Untracked => ("?", Color::DarkGray),
        };

        let cursor = if is_selected { " > " } else { "   " };
        let path = truncate_path(&file.path, width.saturating_sub(6));

        let style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(format!("{} ", symbol), Style::default().fg(symbol_color)),
            Span::styled(path, style),
        ]);
        lines.push(line);

        // Register click area
        let row_y = area.y + (i - start) as u16;
        click_areas
            .file_rows
            .push((Rect::new(area.x, row_y, area.width, 1), i));
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

fn render_footer_normal(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    width: usize,
    click_areas: &mut ClickAreas,
) {
    let button_area = Rect::new(area.x, area.y, area.width, 1);
    let status_area = Rect::new(area.x, area.y + 1, area.width, 1);

    // Buttons
    let (stage_all_text, commit_text, pull_text, push_text) = if width >= 60 {
        (" [Stage All] ", " [Commit] ", " [Pull] ", " [Push] ")
    } else if width >= 40 {
        (" [StgAll] ", " [Cmt] ", " [Pull] ", " [Push] ")
    } else {
        (" [SA] ", " [C] ", " [Pl] ", " [P] ")
    };

    let btn_style = Style::default().fg(Color::White).bg(Color::DarkGray);

    let mut spans = Vec::new();
    let mut x_offset = area.x + 1;

    // Stage All button
    let sa_len = stage_all_text.len() as u16;
    spans.push(Span::styled(stage_all_text, btn_style));
    click_areas.buttons.push((
        Rect::new(x_offset, area.y, sa_len, 1),
        ButtonAction::StageAll,
    ));
    x_offset += sa_len;

    spans.push(Span::raw(" "));
    x_offset += 1;

    // Commit button
    let c_len = commit_text.len() as u16;
    spans.push(Span::styled(commit_text, btn_style));
    click_areas
        .buttons
        .push((Rect::new(x_offset, area.y, c_len, 1), ButtonAction::Commit));
    x_offset += c_len;

    spans.push(Span::raw(" "));
    x_offset += 1;

    // Push button
    let p_len = push_text.len() as u16;
    spans.push(Span::styled(push_text, btn_style));
    click_areas
        .buttons
        .push((Rect::new(x_offset, area.y, p_len, 1), ButtonAction::Push));
    x_offset += p_len;

    spans.push(Span::raw(" "));
    x_offset += 1;

    // Pull button
    let pl_len = pull_text.len() as u16;
    spans.push(Span::styled(pull_text, btn_style));
    click_areas
        .buttons
        .push((Rect::new(x_offset, area.y, pl_len, 1), ButtonAction::Pull));

    let buttons_line = Paragraph::new(Line::from(spans));
    frame.render_widget(buttons_line, button_area);

    // Status message
    if !app.status_message.is_empty() {
        let status = Paragraph::new(format!("  {}", app.status_message))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(status, status_area);
    }
}

fn render_commit_input(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let title = Paragraph::new("  COMMIT").style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let label = Paragraph::new("  msg:");
    frame.render_widget(label, chunks[1]);

    let input = Paragraph::new(format!("  > {}_", app.commit_message))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[2]);
}

fn render_footer_commit(
    frame: &mut Frame,
    _app: &App,
    area: Rect,
    _width: usize,
    click_areas: &mut ClickAreas,
) {
    let button_area = Rect::new(area.x, area.y, area.width, 1);
    let hint_area = Rect::new(area.x, area.y + 1, area.width, 1);

    let btn_style = Style::default().fg(Color::White).bg(Color::DarkGray);

    let commit_text = " [Commit] ";
    let cancel_text = " [Cancel] ";
    let c_len = commit_text.len() as u16;
    let ca_len = cancel_text.len() as u16;

    let mut x_offset = area.x + 1;

    let spans = vec![
        Span::styled(
            commit_text,
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(cancel_text, btn_style),
    ];

    click_areas.buttons.push((
        Rect::new(x_offset, area.y, c_len, 1),
        ButtonAction::ConfirmCommit,
    ));
    x_offset += c_len + 1;
    click_areas.buttons.push((
        Rect::new(x_offset, area.y, ca_len, 1),
        ButtonAction::CancelCommit,
    ));

    let buttons_line = Paragraph::new(Line::from(spans));
    frame.render_widget(buttons_line, button_area);

    let hint =
        Paragraph::new("  Enter: commit  Esc: cancel").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, hint_area);
}

pub const HELP_LINE_COUNT: usize = 14;

fn render_help(frame: &mut Frame, area: Rect, scroll: usize) {
    let help_lines = vec![
        Line::from(Span::styled(
            "  HELP",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j/↓  ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  k/↑  ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  s/Enter  ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle stage"),
        ]),
        Line::from(vec![
            Span::styled("  A    ", Style::default().fg(Color::Yellow)),
            Span::raw("Stage all"),
        ]),
        Line::from(vec![
            Span::styled("  U    ", Style::default().fg(Color::Yellow)),
            Span::raw("Unstage all"),
        ]),
        Line::from(vec![
            Span::styled("  c    ", Style::default().fg(Color::Yellow)),
            Span::raw("Commit"),
        ]),
        Line::from(vec![
            Span::styled("  p    ", Style::default().fg(Color::Yellow)),
            Span::raw("Push"),
        ]),
        Line::from(vec![
            Span::styled("  P    ", Style::default().fg(Color::Yellow)),
            Span::raw("Pull"),
        ]),
        Line::from(vec![
            Span::styled("  b    ", Style::default().fg(Color::Yellow)),
            Span::raw("Branches"),
        ]),
        Line::from(vec![
            Span::styled("  r    ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh"),
        ]),
        Line::from(vec![
            Span::styled("  ?    ", Style::default().fg(Color::Yellow)),
            Span::raw("Help"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw("  Quit"),
        ]),
    ];

    let help = Paragraph::new(help_lines).scroll((scroll as u16, 0));
    frame.render_widget(help, area);
}

fn render_footer_help(frame: &mut Frame, area: Rect) {
    let hint =
        Paragraph::new("  Press q, Esc, or ? to close").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, area);
}

fn render_branch_list(frame: &mut Frame, app: &App, area: Rect, click_areas: &mut ClickAreas) {
    let visible_height = area.height as usize;

    if app.branches.is_empty() {
        let msg = Paragraph::new("  No branches found");
        frame.render_widget(msg, area);
        return;
    }

    let mut lines = Vec::new();
    let start = app.branch_scroll;
    let end = (start + visible_height).min(app.branches.len());

    for i in start..end {
        let branch = &app.branches[i];
        let is_selected = i == app.branch_selected;
        let is_current = *branch == app.repo.branch;

        let cursor = if is_selected { " > " } else { "   " };
        let marker = if is_current { "* " } else { "  " };

        let style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let name_style = if is_current {
            if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Green)
            }
        } else {
            style
        };

        let line = Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(marker, Style::default().fg(Color::Green)),
            Span::styled(branch.clone(), name_style),
        ]);
        lines.push(line);

        // Register click area
        let row_y = area.y + (i - start) as u16;
        click_areas.branch_rows.push(crate::event::BranchRow {
            rect: Rect::new(area.x, row_y, area.width, 1),
            index: i,
        });
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

fn render_footer_branch_list(frame: &mut Frame, area: Rect, click_areas: &mut ClickAreas) {
    let button_area = Rect::new(area.x, area.y, area.width, 1);
    let hint_area = Rect::new(area.x, area.y + 1, area.width, 1);

    let btn_style = Style::default().fg(Color::White).bg(Color::DarkGray);
    let new_text = " [New] ";
    let new_len = new_text.len() as u16;
    let x_offset = area.x + 1;

    let spans = vec![Span::styled(new_text, btn_style)];
    click_areas.buttons.push((
        Rect::new(x_offset, area.y, new_len, 1),
        ButtonAction::EnterBranchCreate,
    ));

    let buttons_line = Paragraph::new(Line::from(spans));
    frame.render_widget(buttons_line, button_area);

    let hint = Paragraph::new("  Enter: switch  n: new branch  Esc: close")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, hint_area);
}

fn render_branch_create(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let title = Paragraph::new("  NEW BRANCH").style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let label = Paragraph::new("  name:");
    frame.render_widget(label, chunks[1]);

    let input = Paragraph::new(format!("  > {}_", app.branch_name_input))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[2]);
}

fn render_footer_branch_create(frame: &mut Frame, area: Rect, click_areas: &mut ClickAreas) {
    let button_area = Rect::new(area.x, area.y, area.width, 1);
    let hint_area = Rect::new(area.x, area.y + 1, area.width, 1);

    let btn_style = Style::default().fg(Color::White).bg(Color::DarkGray);

    let create_text = " [Create] ";
    let cancel_text = " [Cancel] ";
    let cr_len = create_text.len() as u16;
    let ca_len = cancel_text.len() as u16;

    let mut x_offset = area.x + 1;

    let spans = vec![
        Span::styled(
            create_text,
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(cancel_text, btn_style),
    ];

    click_areas.buttons.push((
        Rect::new(x_offset, area.y, cr_len, 1),
        ButtonAction::ConfirmBranchCreate,
    ));
    x_offset += cr_len + 1;
    click_areas.buttons.push((
        Rect::new(x_offset, area.y, ca_len, 1),
        ButtonAction::CancelBranchCreate,
    ));

    let buttons_line = Paragraph::new(Line::from(spans));
    frame.render_widget(buttons_line, button_area);

    let hint =
        Paragraph::new("  Enter: create  Esc: cancel").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, hint_area);
}

pub fn truncate_path(path: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if path.len() <= max_width {
        return path.to_string();
    }
    if max_width <= 3 {
        return path[path.len() - max_width..].to_string();
    }
    format!("..{}", &path[path.len() - (max_width - 2)..])
}
