use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc;

use clickup_api::client::ClickUpClient;
use clickup_api::filter_config::{DueDateFilter, FilterStore, ListFilterConfig};

use crate::app::{self, App, Screen, ViewMode};
use crate::data;
use crate::event::AppEvent;
use crate::ui::filter_panel::{FilterSection, section_item_count};

/// Handles a key event, updating app state and spawning data loads.
pub fn handle_key(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    // Clear error on any key press.
    app.clear_error();

    // If filter panel is open, route keys there.
    if app.filter_panel_open {
        handle_filter_panel_input(app, key, client, tx);
        return;
    }

    // If filter is active, route keys to filter handler first.
    if app.filter_active {
        handle_filter_input(app, key);
        return;
    }

    // Global keys.
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            return;
        }
        KeyCode::Char('/') => {
            // Activate search on list screens.
            if matches!(
                app.screen,
                Screen::WorkspaceSelect
                    | Screen::SpaceList
                    | Screen::SpaceContent
                    | Screen::TaskList
            ) {
                app.start_filter();
                return;
            }
        }
        KeyCode::Char('f') if app.screen == Screen::TaskList => {
            app.filter_panel_open = true;
            app.filter_panel_state.reset();
            return;
        }
        KeyCode::Char('m') if app.screen == Screen::TaskList => {
            toggle_me_mode(app, client, tx);
            return;
        }
        KeyCode::Char('v') if app.screen == Screen::TaskList => {
            app.view_mode = app.view_mode.next();
            app.reset_vision_selection();
            save_filters(app);
            return;
        }
        _ => {}
    }

    // If help is showing, any other key closes it.
    if app.show_help {
        app.show_help = false;
        return;
    }

    // Screen-specific keys.
    match app.screen {
        Screen::Loading => {}
        Screen::WorkspaceSelect => {
            handle_workspace_select(app, key, client, tx);
        }
        Screen::SpaceList => {
            handle_space_list(app, key, client, tx);
        }
        Screen::SpaceContent => {
            handle_space_content(app, key, client, tx);
        }
        Screen::TaskList => {
            handle_task_list(app, key, client, tx);
        }
        Screen::TaskDetail => {
            handle_task_detail(app, key);
        }
    }
}

/// Handles mouse events — click to select items.
pub fn handle_mouse(
    app: &mut App,
    mouse: MouseEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    content_y_offset: u16,
) {
    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        // Calculate which row was clicked relative to content area.
        let row = mouse.row.saturating_sub(content_y_offset) as usize;

        match app.screen {
            Screen::WorkspaceSelect => {
                let indices = app.filtered_workspace_indices();
                if row < indices.len() {
                    app.selected_index = row;
                }
            }
            Screen::SpaceList => {
                let indices = app.filtered_space_indices();
                if row < indices.len() {
                    app.selected_index = row;
                }
            }
            Screen::SpaceContent => {
                let indices = app.filtered_content_indices();
                if row < indices.len() {
                    app.selected_index = row;
                }
            }
            Screen::TaskList => {
                // Account for header row.
                let task_row = row.saturating_sub(1);
                let indices = app.filtered_task_indices();
                if task_row < indices.len() {
                    app.selected_index = task_row;
                }
            }
            Screen::TaskDetail => {
                // Scroll with mouse could be added, but
                // for now clicking just clears error.
                app.clear_error();
            }
            Screen::Loading => {}
        }

        // Double-click is hard to detect with crossterm, ignore for now.
        let _ = (client, tx);
    }
}

fn handle_filter_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_filter();
        }
        KeyCode::Enter => {
            app.confirm_filter();
        }
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.selected_index = 0;
        }
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.selected_index = 0;
        }
        _ => {}
    }
}

fn navigate(app: &mut App, key: KeyEvent, max: usize) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next(max);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous(max);
        }
        _ => {}
    }
}

fn handle_workspace_select(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    let indices = app.filtered_workspace_indices();
    navigate(app, key, indices.len());

    match key.code {
        KeyCode::Enter if !indices.is_empty() => {
            let real_idx = indices[app.selected_index];
            let ws = app.workspaces[real_idx].clone();
            app.cancel_filter();
            app.push_breadcrumb(&ws.name);
            app.current_workspace = Some(ws.clone());
            app.reset_selection();
            app.screen = Screen::SpaceList;
            app.loading = true;
            data::spawn_load_spaces(client, tx, &ws.id);
        }
        KeyCode::Char('r') => {
            app.loading = true;
            data::spawn_load_workspaces(client, tx);
        }
        _ => {}
    }
}

fn handle_space_list(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    let indices = app.filtered_space_indices();
    navigate(app, key, indices.len());

    match key.code {
        KeyCode::Esc => {
            app.cancel_filter();
            app.pop_breadcrumb();
            app.reset_selection();
            // Keep spaces cached — don't clear.
            app.screen = Screen::WorkspaceSelect;
        }
        KeyCode::Enter if !indices.is_empty() => {
            let real_idx = indices[app.selected_index];
            let space = app.spaces[real_idx].clone();
            app.cancel_filter();
            app.push_breadcrumb(&space.name);
            app.current_space = Some(space.clone());
            app.reset_selection();
            app.screen = Screen::SpaceContent;
            app.loading = true;
            data::spawn_load_folders_and_lists(client, tx, &space.id);
        }
        KeyCode::Char('r') => {
            if let Some(ws) = &app.current_workspace {
                app.loading = true;
                data::spawn_load_spaces(client, tx, &ws.id);
            }
        }
        _ => {}
    }
}

fn handle_space_content(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    let indices = app.filtered_content_indices();
    navigate(app, key, indices.len());

    match key.code {
        KeyCode::Esc => {
            app.cancel_filter();
            app.pop_breadcrumb();
            app.reset_selection();
            app.space_content.clear();
            app.screen = Screen::SpaceList;
        }
        KeyCode::Enter if !indices.is_empty() => {
            let real_idx = indices[app.selected_index];
            match &app.space_content[real_idx] {
                app::SpaceContentItem::FolderItem { .. } => {
                    app.toggle_folder(real_idx);
                }
                app::SpaceContentItem::ListInFolder { list, .. }
                | app::SpaceContentItem::FolderlessList { list } => {
                    let list = list.clone();
                    app.saved_content_index = app.selected_index;
                    app.cancel_filter();
                    app.push_breadcrumb(&list.name);
                    app.current_list = Some(list.clone());
                    app.reset_selection();
                    app.reset_pagination();

                    // Load saved filters for this list.
                    load_saved_filters(app, &list.id);

                    app.screen = Screen::TaskList;
                    app.loading = true;
                    let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
                    data::spawn_load_tasks_page(
                        client,
                        tx,
                        &list.id,
                        0,
                        &app.task_filters.statuses,
                        &assignees,
                        app.task_filters.include_closed,
                    );
                }
            }
        }
        KeyCode::Char('r') => {
            if let Some(space) = &app.current_space {
                app.loading = true;
                data::spawn_load_folders_and_lists(client, tx, &space.id);
            }
        }
        _ => {}
    }
}

fn handle_task_list(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    match app.view_mode {
        ViewMode::List => handle_task_list_view(app, key, client, tx),
        ViewMode::VisionSections | ViewMode::VisionBoard => {
            handle_task_vision_view(app, key, client, tx);
        }
    }
}

fn handle_task_list_view(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    let indices = app.filtered_task_indices();
    navigate(app, key, indices.len());

    // Auto-load next page when near bottom.
    if app.has_more_pages
        && !app.loading_more
        && !indices.is_empty()
        && app.selected_index + 5 >= indices.len()
        && let Some(list) = app.current_list.clone()
    {
        app.loading_more = true;
        let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
        data::spawn_load_tasks_page(
            client,
            tx,
            &list.id,
            app.current_page + 1,
            &app.task_filters.statuses,
            &assignees,
            app.task_filters.include_closed,
        );
    }

    match key.code {
        KeyCode::Esc => {
            app.cancel_filter();
            app.pop_breadcrumb();
            app.clear_tasks();
            app.current_list = None;
            app.selected_index = app.saved_content_index;
            app.reset_pagination();
            app.screen = Screen::SpaceContent;
        }
        KeyCode::Enter if !indices.is_empty() => {
            let real_idx = indices[app.selected_index];
            let task = app.tasks[real_idx].clone();
            app.saved_task_index = app.selected_index;
            app.cancel_filter();
            app.push_breadcrumb(&task.name);
            app.reset_selection();
            app.screen = Screen::TaskDetail;
            app.loading = true;
            data::spawn_load_task_detail(client, tx, &task.id);
            data::spawn_load_comments(client, tx, &task.id);
        }
        KeyCode::Char('r') => {
            if let Some(list) = app.current_list.clone() {
                app.loading = true;
                app.reset_pagination();
                let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
                data::spawn_load_tasks_page(
                    client,
                    tx,
                    &list.id,
                    0,
                    &app.task_filters.statuses,
                    &assignees,
                    app.task_filters.include_closed,
                );
            }
        }
        _ => {}
    }
}

fn handle_task_vision_view(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    let num_groups = app.status_groups.len();

    match key.code {
        // Navigate between groups.
        KeyCode::Tab => {
            if num_groups > 0 {
                app.focused_group = (app.focused_group + 1) % num_groups;
                app.group_selected_index = 0;
            }
        }
        KeyCode::BackTab => {
            if num_groups > 0 {
                app.focused_group = if app.focused_group == 0 {
                    num_groups - 1
                } else {
                    app.focused_group - 1
                };
                app.group_selected_index = 0;
            }
        }
        // Navigate within group.
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(group) = app.status_groups.get(app.focused_group) {
                let max = group.task_indices.len();
                if max > 0 {
                    app.group_selected_index = (app.group_selected_index + 1) % max;
                }
            }
            // Auto-load next page when near bottom of last group.
            check_vision_pagination(app, client, tx);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(group) = app.status_groups.get(app.focused_group) {
                let max = group.task_indices.len();
                if max > 0 && app.group_selected_index == 0 {
                    app.group_selected_index = max - 1;
                } else if app.group_selected_index > 0 {
                    app.group_selected_index -= 1;
                }
            }
        }
        KeyCode::Esc => {
            app.cancel_filter();
            app.pop_breadcrumb();
            app.clear_tasks();
            app.current_list = None;
            app.selected_index = app.saved_content_index;
            app.reset_pagination();
            app.reset_vision_selection();
            app.screen = Screen::SpaceContent;
        }
        KeyCode::Enter => {
            if let Some(group) = app.status_groups.get(app.focused_group)
                && let Some(&task_idx) = group.task_indices.get(app.group_selected_index)
            {
                let task = app.tasks[task_idx].clone();
                app.saved_task_index = app.selected_index;
                app.cancel_filter();
                app.push_breadcrumb(&task.name);
                app.reset_selection();
                app.screen = Screen::TaskDetail;
                app.loading = true;
                data::spawn_load_task_detail(client, tx, &task.id);
                data::spawn_load_comments(client, tx, &task.id);
            }
        }
        KeyCode::Char('r') => {
            if let Some(list) = app.current_list.clone() {
                app.loading = true;
                app.reset_pagination();
                app.reset_vision_selection();
                let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
                data::spawn_load_tasks_page(
                    client,
                    tx,
                    &list.id,
                    0,
                    &app.task_filters.statuses,
                    &assignees,
                    app.task_filters.include_closed,
                );
            }
        }
        _ => {}
    }
}

/// Check if we need to auto-load the next page in vision views.
fn check_vision_pagination(
    app: &mut App,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    if !app.has_more_pages || app.loading_more {
        return;
    }
    // If we're in the last group and near its end, load more.
    if app.focused_group + 1 >= app.status_groups.len()
        && let Some(group) = app.status_groups.get(app.focused_group)
        && (group.task_indices.is_empty()
            || app.group_selected_index + 3 >= group.task_indices.len())
        && let Some(list) = app.current_list.clone()
    {
        app.loading_more = true;
        let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
        data::spawn_load_tasks_page(
            client,
            tx,
            &list.id,
            app.current_page + 1,
            &app.task_filters.statuses,
            &assignees,
            app.task_filters.include_closed,
        );
    }
}

fn handle_task_detail(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.pop_breadcrumb();
            app.current_task = None;
            app.comments.clear();
            app.selected_index = app.saved_task_index;
            app.scroll_offset = 0;
            app.screen = Screen::TaskList;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_filter_panel_input(
    app: &mut App,
    key: KeyEvent,
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('f') => {
            app.filter_panel_open = false;
        }
        KeyCode::Tab => {
            app.filter_panel_state.next_section();
        }
        KeyCode::BackTab => {
            app.filter_panel_state.prev_section();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = section_item_count(app, app.filter_panel_state.section);
            if app.filter_panel_state.cursor + 1 < max {
                app.filter_panel_state.cursor += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.filter_panel_state.cursor > 0 {
                app.filter_panel_state.cursor -= 1;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            let needs_refetch = toggle_filter_item(app);
            if needs_refetch {
                refetch_tasks(app, client, tx);
            }
            save_filters(app);
        }
        _ => {}
    }
}

/// Toggles the currently focused filter item. Returns `true` if server-side
/// filters changed and a re-fetch is needed.
fn toggle_filter_item(app: &mut App) -> bool {
    let section = app.filter_panel_state.section;
    let cursor = app.filter_panel_state.cursor;
    let had_server_filters = app.task_filters.has_server_filters();

    match section {
        FilterSection::Statuses => {
            let statuses: Vec<String> = app
                .current_space
                .as_ref()
                .map(|s| s.statuses.iter().map(|st| st.status.clone()).collect())
                .unwrap_or_default();
            if let Some(status_name) = statuses.get(cursor) {
                toggle_vec_item(&mut app.task_filters.statuses, status_name);
            }
        }
        FilterSection::Priorities => {
            let priorities = ["urgent", "high", "normal", "low", "none"];
            if let Some(prio) = priorities.get(cursor) {
                toggle_vec_item(&mut app.task_filters.priorities, prio);
            }
        }
        FilterSection::DueDate => {
            let options = [
                DueDateFilter::All,
                DueDateFilter::Overdue,
                DueDateFilter::Today,
                DueDateFilter::ThisWeek,
                DueDateFilter::NoDueDate,
            ];
            if let Some(&variant) = options.get(cursor) {
                app.task_filters.due_date_filter = variant;
            }
        }
        FilterSection::MeMode => {
            app.task_filters.me_mode = !app.task_filters.me_mode;
        }
        FilterSection::IncludeClosed => {
            app.task_filters.include_closed = !app.task_filters.include_closed;
        }
    }

    // Server-side filters changed if status, assignee (me_mode), or include_closed changed.
    let has_server_filters = app.task_filters.has_server_filters();
    let me_mode_changed = matches!(section, FilterSection::MeMode);
    had_server_filters != has_server_filters
        || matches!(
            section,
            FilterSection::Statuses | FilterSection::IncludeClosed
        )
        || me_mode_changed
}

/// Toggles an item in a Vec<String> (add if absent, remove if present).
fn toggle_vec_item(vec: &mut Vec<String>, item: &str) {
    if let Some(pos) = vec.iter().position(|s| s.eq_ignore_ascii_case(item)) {
        vec.remove(pos);
    } else {
        vec.push(item.to_string());
    }
}

/// Toggles Me Mode and refetches if needed.
fn toggle_me_mode(app: &mut App, client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>) {
    app.task_filters.me_mode = !app.task_filters.me_mode;
    refetch_tasks(app, client, tx);
    save_filters(app);
}

/// Re-fetches tasks from page 0 with current filters.
fn refetch_tasks(app: &mut App, client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>) {
    if let Some(list) = app.current_list.clone() {
        app.loading = true;
        app.reset_pagination();
        let assignees = app.task_filters.server_assignees(app.current_user.as_ref());
        data::spawn_load_tasks_page(
            client,
            tx,
            &list.id,
            0,
            &app.task_filters.statuses,
            &assignees,
            app.task_filters.include_closed,
        );
    }
}

/// Saves current filters to persistent storage.
fn save_filters(app: &App) {
    if let Some(list) = &app.current_list {
        let mut config = ListFilterConfig::from(&app.task_filters);
        config.view_mode = app.view_mode.into();
        if let Err(e) = FilterStore::save(&list.id, &config) {
            tracing::warn!("failed to save filters: {e}");
        }
    }
}

/// Loads saved filters for a list and applies them to app state.
fn load_saved_filters(app: &mut App, list_id: &str) {
    match FilterStore::load(list_id) {
        Ok(Some(config)) => {
            app.view_mode = config.view_mode.into();
            app.task_filters = config.into();
            tracing::debug!(%list_id, "loaded saved filters");
        }
        Ok(None) => {
            app.task_filters = Default::default();
            app.view_mode = Default::default();
        }
        Err(e) => {
            tracing::warn!("failed to load saved filters: {e}");
            app.task_filters = Default::default();
            app.view_mode = Default::default();
        }
    }
}
