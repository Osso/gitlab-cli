use anyhow::Result;

use crate::api::IssueListParams;
use crate::cli::IssueCommands;
use crate::commands::print::print_issues;
use crate::config::Config;
use crate::get_client;

pub async fn handle(config: &mut Config, command: IssueCommands) -> Result<()> {
    match command {
        IssueCommands::List { state, author, assignee, labels, search, created_after, per_page, project } => {
            handle_list(config, project.as_deref(), IssueListParams { per_page, state, author_username: author, assignee_username: assignee, labels, search, created_after }).await
        }
        IssueCommands::Show { iid, project } => handle_show(config, project.as_deref(), iid).await,
        IssueCommands::Create { title, description, labels, assignee, project } => {
            handle_create(config, project.as_deref(), title, description, labels, assignee).await
        }
    }
}

async fn handle_list(
    config: &mut Config,
    project: Option<&str>,
    params: IssueListParams,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.list_issues(&params).await?;
    print_issues(&result);
    Ok(())
}

async fn handle_show(config: &mut Config, project: Option<&str>, iid: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.get_issue(iid).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn handle_create(
    config: &mut Config,
    project: Option<&str>,
    title: String,
    description: Option<String>,
    labels: Option<String>,
    assignee: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client
        .create_issue(
            &title,
            description.as_deref(),
            labels.as_deref(),
            assignee.as_deref(),
        )
        .await?;
    let iid = result["iid"].as_u64().unwrap_or(0);
    let web_url = result["web_url"].as_str().unwrap_or("");
    println!("Created issue #{}: {}", iid, title);
    println!("{}", web_url);
    Ok(())
}
