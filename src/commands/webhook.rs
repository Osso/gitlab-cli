use anyhow::Result;

use crate::api;
use crate::cli::WebhookCommands;
use crate::commands::print::print_webhooks;
use crate::config::Config;
use crate::get_client;

pub async fn handle(config: &mut Config, command: WebhookCommands) -> Result<()> {
    match command {
        WebhookCommands::List { project } => handle_list(config, project.as_deref()).await,
        WebhookCommands::Show { id, project } => handle_show(config, project.as_deref(), id).await,
        WebhookCommands::Create { url, token, push, merge_request, issue, pipeline, tag, note, job, release, ssl_verification, project } => {
            let params = api::WebhookCreateParams { url, token, push_events: push, merge_requests_events: merge_request, issues_events: issue, pipeline_events: pipeline, tag_push_events: tag, note_events: note, job_events: job, releases_events: release, enable_ssl_verification: ssl_verification };
            handle_create(config, project.as_deref(), params).await
        }
        WebhookCommands::Update { id, url, token, push, merge_request, issue, pipeline, tag, note, job, release, ssl_verification, project } => {
            let params = api::WebhookUpdateParams { url, token, push_events: push, merge_requests_events: merge_request, issues_events: issue, pipeline_events: pipeline, tag_push_events: tag, note_events: note, job_events: job, releases_events: release, enable_ssl_verification: ssl_verification };
            handle_update(config, project.as_deref(), id, params).await
        }
        WebhookCommands::Delete { id, project } => handle_delete(config, project.as_deref(), id).await,
        WebhookCommands::Test { id, event, project } => handle_test(config, project.as_deref(), id, &event).await,
    }
}

async fn handle_list(config: &mut Config, project: Option<&str>) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.list_webhooks().await?;
    print_webhooks(&result);
    Ok(())
}

async fn handle_show(config: &mut Config, project: Option<&str>, id: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.get_webhook(id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn handle_create(
    config: &mut Config,
    project: Option<&str>,
    params: api::WebhookCreateParams,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.create_webhook(&params).await?;
    let hook_id = result["id"].as_u64().unwrap_or(0);
    let hook_url = result["url"].as_str().unwrap_or("");
    println!("Created webhook {} -> {}", hook_id, hook_url);
    Ok(())
}

async fn handle_update(
    config: &mut Config,
    project: Option<&str>,
    id: u64,
    params: api::WebhookUpdateParams,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.update_webhook(id, &params).await?;
    let hook_url = result["url"].as_str().unwrap_or("");
    println!("Updated webhook {} -> {}", id, hook_url);
    Ok(())
}

async fn handle_delete(config: &mut Config, project: Option<&str>, id: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    client.delete_webhook(id).await?;
    println!("Deleted webhook {}", id);
    Ok(())
}

async fn handle_test(
    config: &mut Config,
    project: Option<&str>,
    id: u64,
    event: &str,
) -> Result<()> {
    let client = get_client(config, project).await?;
    client.test_webhook(id, event).await?;
    println!("Sent test {} event to webhook {}", event, id);
    Ok(())
}
