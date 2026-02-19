use anyhow::{bail, Result};

use crate::cli::{MirrorCommands, ProjectCommands};
use crate::commands::print::{print_mirrors, print_projects};
use crate::config::Config;
use crate::get_group_client;

pub async fn handle(config: &mut Config, command: ProjectCommands) -> Result<()> {
    match command {
        ProjectCommands::Archive { project } => handle_archive(config, &project).await,
        ProjectCommands::Unarchive { project } => handle_unarchive(config, &project).await,
        ProjectCommands::List { group, archived, per_page } => handle_list(config, &group, per_page, archived).await,
        ProjectCommands::Update {
            project,
            repository_access_level,
            issues_access_level,
            merge_requests_access_level,
            builds_access_level,
            wiki_access_level,
            snippets_access_level,
            pages_access_level,
            archived,
            description,
            default_branch,
            visibility,
        } => {
            let body = build_update_body(
                repository_access_level,
                issues_access_level,
                merge_requests_access_level,
                builds_access_level,
                wiki_access_level,
                snippets_access_level,
                pages_access_level,
                archived,
                description,
                default_branch,
                visibility,
            )?;
            handle_update(config, &project, &body).await
        }
        ProjectCommands::Mirrors { command } => handle_mirrors(config, command).await,
    }
}

async fn handle_archive(config: &mut Config, project: &str) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.archive_project(project).await?;
    let name = result["path_with_namespace"].as_str().unwrap_or(project);
    println!("Archived: {}", name);
    Ok(())
}

async fn handle_unarchive(config: &mut Config, project: &str) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.unarchive_project(project).await?;
    let name = result["path_with_namespace"].as_str().unwrap_or(project);
    println!("Unarchived: {}", name);
    Ok(())
}

async fn handle_list(
    config: &mut Config,
    group: &str,
    per_page: u32,
    archived: bool,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.list_group_projects(group, per_page, archived).await?;
    print_projects(&result);
    Ok(())
}

fn insert_access_level(
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<String>,
) -> Result<()> {
    if let Some(v) = value {
        match v.as_str() {
            "enabled" | "private" | "disabled" => {
                body.insert(key.to_string(), serde_json::Value::String(v));
            }
            _ => bail!("Invalid value for {}: '{}' (expected: enabled, private, disabled)", key, v),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_update_body(
    repository_access_level: Option<String>,
    issues_access_level: Option<String>,
    merge_requests_access_level: Option<String>,
    builds_access_level: Option<String>,
    wiki_access_level: Option<String>,
    snippets_access_level: Option<String>,
    pages_access_level: Option<String>,
    archived: Option<bool>,
    description: Option<String>,
    default_branch: Option<String>,
    visibility: Option<String>,
) -> Result<serde_json::Value> {
    let mut body = serde_json::Map::new();

    insert_access_level(&mut body, "repository_access_level", repository_access_level)?;
    insert_access_level(&mut body, "issues_access_level", issues_access_level)?;
    insert_access_level(&mut body, "merge_requests_access_level", merge_requests_access_level)?;
    insert_access_level(&mut body, "builds_access_level", builds_access_level)?;
    insert_access_level(&mut body, "wiki_access_level", wiki_access_level)?;
    insert_access_level(&mut body, "snippets_access_level", snippets_access_level)?;
    insert_access_level(&mut body, "pages_access_level", pages_access_level)?;

    if let Some(v) = archived {
        body.insert("archived".to_string(), serde_json::Value::Bool(v));
    }
    if let Some(v) = description {
        body.insert("description".to_string(), serde_json::Value::String(v));
    }
    if let Some(v) = default_branch {
        body.insert("default_branch".to_string(), serde_json::Value::String(v));
    }
    if let Some(v) = visibility {
        match v.as_str() {
            "private" | "internal" | "public" => {
                body.insert("visibility".to_string(), serde_json::Value::String(v));
            }
            _ => bail!("Invalid visibility: '{}' (expected: private, internal, public)", v),
        }
    }

    if body.is_empty() {
        bail!("No settings specified. Use --help to see available options.");
    }

    Ok(serde_json::Value::Object(body))
}

async fn handle_update(
    config: &mut Config,
    project: &str,
    body: &serde_json::Value,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.update_project(project, body).await?;
    let name = result["path_with_namespace"]
        .as_str()
        .unwrap_or(project);
    println!("Updated: {}", name);
    print_updated_fields(body);
    Ok(())
}

fn print_updated_fields(body: &serde_json::Value) {
    if let Some(obj) = body.as_object() {
        for (key, value) in obj {
            let display = key.replace('_', " ");
            match value {
                serde_json::Value::String(s) => println!("  {} = {}", display, s),
                serde_json::Value::Bool(b) => println!("  {} = {}", display, b),
                _ => println!("  {} = {}", display, value),
            }
        }
    }
}

async fn handle_mirrors(config: &mut Config, command: MirrorCommands) -> Result<()> {
    match command {
        MirrorCommands::List { project } => handle_mirror_list(config, &project).await,
        MirrorCommands::Create { project, url, only_protected } => handle_mirror_create(config, &project, &url, only_protected).await,
        MirrorCommands::CreateHttps { project, url, user, password, only_protected } => handle_mirror_create_https(config, &project, &url, &user, &password, only_protected).await,
        MirrorCommands::Remove { project, mirror_id } => handle_mirror_remove(config, &project, mirror_id).await,
        MirrorCommands::Sync { project, mirror_id } => handle_mirror_sync(config, &project, mirror_id).await,
    }
}

async fn handle_mirror_list(config: &mut Config, project: &str) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.list_push_mirrors(project).await?;
    print_mirrors(&result);
    Ok(())
}

async fn handle_mirror_create(
    config: &mut Config,
    project: &str,
    url: &str,
    only_protected: bool,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.create_push_mirror(project, url, true, only_protected).await?;
    let id = result["id"].as_u64().unwrap_or(0);
    let mirror_url = result["url"].as_str().unwrap_or(url);
    println!("Created push mirror (id: {}) -> {}", id, mirror_url);

    if let Ok(ssh_key) = client.get_push_mirror_public_key(project, id).await {
        if !ssh_key.is_empty() {
            println!("\nSSH public key (add as deploy key with write access):");
            println!("{}", ssh_key);
        }
    }

    println!(
        "\nNOTE: Go to GitLab UI and click 'Detect host keys' to complete setup:"
    );
    println!(
        "https://gitlab.com/{}/-/settings/repository#js-push-remote-settings",
        project
    );
    Ok(())
}

async fn handle_mirror_create_https(
    config: &mut Config,
    project: &str,
    url: &str,
    user: &str,
    password: &str,
    only_protected: bool,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client
        .create_push_mirror_https(project, url, user, password, only_protected)
        .await?;
    let id = result["id"].as_u64().unwrap_or(0);
    let mirror_url = result["url"].as_str().unwrap_or(url);
    println!("Created HTTPS push mirror (id: {}) -> {}", id, mirror_url);
    Ok(())
}

async fn handle_mirror_remove(config: &mut Config, project: &str, mirror_id: u64) -> Result<()> {
    let client = get_group_client(config).await?;
    client.delete_push_mirror(project, mirror_id).await?;
    println!("Removed mirror {}", mirror_id);
    Ok(())
}

async fn handle_mirror_sync(config: &mut Config, project: &str, mirror_id: u64) -> Result<()> {
    let client = get_group_client(config).await?;
    client.sync_push_mirror(project, mirror_id).await?;
    println!("Triggered sync for mirror {}", mirror_id);
    Ok(())
}
