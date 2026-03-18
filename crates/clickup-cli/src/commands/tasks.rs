use anyhow::{Context, Result};
use clap::Subcommand;

use crate::client_factory::create_client;
use crate::output;

/// Task subcommands.
#[derive(Subcommand)]
pub enum TaskCommands {
    /// List tasks in a list.
    List {
        /// List ID to fetch tasks from.
        #[arg(long)]
        list: String,

        /// Filter by status (can be repeated).
        #[arg(long)]
        status: Vec<String>,

        /// Filter by assignee (can be repeated).
        #[arg(long)]
        assignee: Vec<String>,

        /// Include closed tasks in the results.
        #[arg(long, default_value_t = false)]
        include_closed: bool,
    },
    /// Show task details.
    Get {
        /// The task ID.
        task_id: String,
    },
    /// Rich view of a task with rendered markdown description.
    View {
        /// The task ID.
        task_id: String,
    },
}

impl TaskCommands {
    /// Dispatches to the appropriate task handler.
    pub async fn run(self, format: &str, workspace_override: Option<&str>) -> Result<()> {
        match self {
            Self::List {
                list,
                status,
                assignee,
                include_closed,
            } => {
                list_tasks(
                    format,
                    workspace_override,
                    &list,
                    &status,
                    &assignee,
                    include_closed,
                )
                .await
            }
            Self::Get { task_id } => get_task(format, workspace_override, &task_id).await,
            Self::View { task_id } => view_task(workspace_override, &task_id).await,
        }
    }
}

async fn list_tasks(
    format: &str,
    workspace_override: Option<&str>,
    list_id: &str,
    statuses: &[String],
    assignees: &[String],
    include_closed: bool,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let tasks = if statuses.is_empty() && assignees.is_empty() && !include_closed {
        client
            .get_tasks(list_id)
            .await
            .context("failed to fetch tasks")?
    } else {
        let s_refs: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
        let a_refs: Vec<&str> = assignees.iter().map(|a| a.as_str()).collect();
        client
            .get_tasks_with_filters(list_id, &s_refs, &a_refs, include_closed)
            .await
            .context("failed to fetch tasks")?
    };

    if format == "json" {
        output::print_json(&tasks);
        return Ok(());
    }

    if tasks.is_empty() {
        output::info("No tasks found.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = tasks
        .iter()
        .map(|t| {
            let assignee_names: Vec<&str> =
                t.assignees.iter().map(|a| a.username.as_str()).collect();
            let priority_label = t.priority.as_ref().and_then(|p| p.priority.as_deref());

            vec![
                t.id.clone(),
                t.name.clone(),
                output::format_status(&t.status.status, &t.status.color),
                assignee_names.join(", "),
                output::format_priority(priority_label),
                output::format_date(t.due_date.as_deref()),
            ]
        })
        .collect();

    output::print_table(
        &["ID", "Name", "Status", "Assignee", "Priority", "Due"],
        rows,
    );

    Ok(())
}

async fn get_task(format: &str, workspace_override: Option<&str>, task_id: &str) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let task = client
        .get_task(task_id)
        .await
        .context("failed to fetch task")?;

    if format == "json" {
        output::print_json(&task);
        return Ok(());
    }

    print_task_detail(&task);

    Ok(())
}

async fn view_task(workspace_override: Option<&str>, task_id: &str) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let task = client
        .get_task(task_id)
        .await
        .context("failed to fetch task")?;

    print_task_detail(&task);

    // Render markdown description.
    if let Some(md) = &task.markdown_description {
        if !md.is_empty() {
            println!("\n─── Description ───\n");
            output::print_markdown(md);
        }
    } else if let Some(desc) = &task.description
        && !desc.is_empty()
    {
        println!("\n─── Description ───\n");
        println!("{desc}");
    }

    Ok(())
}

fn print_task_detail(task: &clickup_api::models::Task) {
    output::info(&format!("{} ({})", task.name, task.id));

    println!(
        "  Status:   {}",
        output::format_status(&task.status.status, &task.status.color,)
    );

    let priority_label = task.priority.as_ref().and_then(|p| p.priority.as_deref());
    println!("  Priority: {}", output::format_priority(priority_label));

    if !task.assignees.is_empty() {
        let names: Vec<&str> = task.assignees.iter().map(|a| a.username.as_str()).collect();
        println!("  Assignees: {}", names.join(", "));
    }

    println!(
        "  Created:  {}",
        output::format_date(Some(&task.date_created))
    );
    println!(
        "  Updated:  {}",
        output::format_date(Some(&task.date_updated))
    );
    println!(
        "  Due:      {}",
        output::format_date(task.due_date.as_deref())
    );

    if !task.tags.is_empty() {
        let tag_names: Vec<&str> = task.tags.iter().map(|t| t.name.as_str()).collect();
        println!("  Tags:     {}", tag_names.join(", "));
    }

    println!("  URL:      {}", task.url);
}
