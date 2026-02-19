use anyhow::{bail, Context, Result};

use crate::api::Client;
use crate::cli::MrCommands;
use crate::commands::print::print_mrs;
use crate::get_client;
use crate::{api::MrListParams, config::Config};

pub async fn handle(config: &mut Config, command: MrCommands) -> Result<()> {
    match command {
        MrCommands::List { state, author, created_after, created_before, updated_after, order_by, sort, per_page, project } => {
            handle_list(config, project.as_deref(), MrListParams { per_page, state, author_username: author, created_after, created_before, updated_after, order_by, sort }).await
        }
        MrCommands::Show { iid, project } => handle_show(config, project.as_deref(), iid).await,
        MrCommands::Automerge { iid, keep_branch, project } => handle_automerge(config, project.as_deref(), iid, keep_branch).await,
        MrCommands::Merge { iid, keep_branch, project } => handle_merge(config, project.as_deref(), iid, keep_branch).await,
        MrCommands::Diff { iid, json, project } => handle_diff(config, project.as_deref(), iid, json).await,
        MrCommands::Close { iid, project } => handle_close(config, project.as_deref(), iid).await,
        MrCommands::Comments { iid, per_page, project } => handle_comments(config, project.as_deref(), iid, per_page).await,
        MrCommands::Comment { iid, message, project } => handle_comment(config, project.as_deref(), iid, message).await,
        MrCommands::Approve { iid, project } => handle_approve(config, project.as_deref(), iid).await,
        MrCommands::Discussions { iid, unresolved, per_page, project } => handle_discussions(config, project.as_deref(), iid, unresolved, per_page).await,
        MrCommands::CommentInline { iid, file, line, old_line, base_sha, head_sha, start_sha, old_file, message, project } => {
            handle_comment_inline(config, project.as_deref(), iid, file, line, old_line, base_sha, head_sha, start_sha, old_file, message).await
        }
        MrCommands::Reply { iid, discussion, message, project } => handle_reply(config, project.as_deref(), iid, discussion, message).await,
        MrCommands::Resolve { iid, discussion, unresolve, project } => handle_resolve(config, project.as_deref(), iid, discussion, unresolve).await,
        MrCommands::Create { title, description, source, target, auto_merge, keep_branch, project } => {
            handle_create(config, project.as_deref(), title, description, source, target, auto_merge, keep_branch).await
        }
    }
}

async fn handle_list(config: &mut Config, project: Option<&str>, params: MrListParams) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.list_merge_requests(&params).await?;
    print_mrs(&result);
    Ok(())
}

async fn handle_show(config: &mut Config, project: Option<&str>, iid: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.get_merge_request(iid).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn handle_automerge(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    keep_branch: bool,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 0..max_retries {
        match client.set_automerge(iid, !keep_branch).await {
            Ok(result) => {
                let title = result["title"].as_str().unwrap_or("");
                println!("Auto-merge enabled for !{}: {}", iid, title);
                return Ok(());
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("405") && attempt < max_retries - 1 {
                    eprintln!(
                        "Pipeline not ready, retrying in 10s... ({}/{})",
                        attempt + 1,
                        max_retries
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    last_error = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }
    Ok(())
}

async fn handle_merge(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    keep_branch: bool,
) -> Result<()> {
    let client = get_client(config, project).await?;
    match client.merge_merge_request(iid, !keep_branch).await {
        Ok(result) => {
            let title = result["title"].as_str().unwrap_or("");
            println!("Merged !{}: {}", iid, title);
            Ok(())
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("405") {
                bail!(
                    "Cannot merge !{}: MR is not in a mergeable state \
                     (pipeline may be running, or merge conflicts exist)",
                    iid
                );
            }
            if err_str.contains("401") {
                bail!("Cannot merge !{}: insufficient permissions", iid);
            }
            Err(e)
        }
    }
}

async fn handle_diff(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    json: bool,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.get_merge_request_changes(iid).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_diff_changes(&result);
    }
    Ok(())
}

fn print_diff_changes(result: &serde_json::Value) {
    if let Some(changes) = result["changes"].as_array() {
        for change in changes {
            let old_path = change["old_path"].as_str().unwrap_or("");
            let new_path = change["new_path"].as_str().unwrap_or("");
            let diff = change["diff"].as_str().unwrap_or("");

            println!("--- a/{}", old_path);
            println!("+++ b/{}", new_path);
            print!("{}", diff);
        }
    }
}

async fn handle_close(config: &mut Config, project: Option<&str>, iid: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client
        .update_merge_request(iid, &serde_json::json!({"state_event": "close"}))
        .await?;
    let title = result["title"].as_str().unwrap_or("");
    println!("Closed !{}: {}", iid, title);
    Ok(())
}

async fn handle_comments(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    per_page: u32,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let notes = client.list_mr_notes(iid, per_page).await?;
    if let Some(arr) = notes.as_array() {
        if arr.is_empty() {
            println!("No comments on !{}", iid);
        } else {
            for note in arr {
                print_mr_note(note);
            }
        }
    }
    Ok(())
}

fn print_mr_note(note: &serde_json::Value) {
    let system = note["system"].as_bool().unwrap_or(false);
    if system {
        return;
    }
    let id = note["id"].as_u64().unwrap_or(0);
    let author = note["author"]["username"].as_str().unwrap_or("?");
    let created = note["created_at"].as_str().unwrap_or("?");
    let body = note["body"].as_str().unwrap_or("");
    println!("--- #{} by @{} ({})", id, author, created);
    println!("{}", body);
    println!();
}

async fn handle_comment(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    message: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let body = read_message(message)?;
    if body.trim().is_empty() {
        bail!("Comment body is empty");
    }
    let result = client.create_mr_note(iid, &body).await?;
    let note_id = result["id"].as_u64().unwrap_or(0);
    println!("Comment #{} added to !{}", note_id, iid);
    Ok(())
}

async fn handle_approve(config: &mut Config, project: Option<&str>, iid: u64) -> Result<()> {
    let client = get_client(config, project).await?;
    client.approve_merge_request(iid).await?;
    println!("Approved !{}", iid);
    Ok(())
}

async fn handle_discussions(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    unresolved: bool,
    per_page: u32,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let discussions = client.list_mr_discussions(iid, per_page).await?;
    if let Some(arr) = discussions.as_array() {
        let threads: Vec<_> = arr
            .iter()
            .filter(|d| is_visible_thread(d, unresolved))
            .collect();

        if threads.is_empty() {
            let qualifier = if unresolved { "unresolved " } else { "" };
            println!("No {}discussion threads on !{}", qualifier, iid);
        } else {
            for d in &threads {
                print_discussion_thread(d);
            }
        }
    }
    Ok(())
}

fn is_visible_thread(d: &serde_json::Value, unresolved: bool) -> bool {
    let notes = d["notes"].as_array();
    let is_thread = notes.map(|n| n.len() > 1).unwrap_or(false)
        || notes
            .and_then(|n| n.first())
            .and_then(|n| n["resolvable"].as_bool())
            .unwrap_or(false);
    if !is_thread {
        return false;
    }
    if unresolved {
        notes
            .map(|n| {
                n.iter().any(|note| {
                    note["resolvable"].as_bool().unwrap_or(false)
                        && !note["resolved"].as_bool().unwrap_or(true)
                })
            })
            .unwrap_or(false)
    } else {
        true
    }
}

fn print_discussion_thread(d: &serde_json::Value) {
    let disc_id = d["id"].as_str().unwrap_or("?");
    let notes = d["notes"].as_array();
    let first = notes.and_then(|n| n.first());

    let position = first.and_then(|n| n["position"].as_object());
    if let Some(pos) = position {
        let path = pos
            .get("new_path")
            .or(pos.get("old_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let line = pos
            .get("new_line")
            .or(pos.get("old_line"))
            .and_then(|v| v.as_u64())
            .map(|l| l.to_string())
            .unwrap_or_default();
        println!("--- {} ({}:{})", disc_id, path, line);
    } else {
        println!("--- {}", disc_id);
    }

    let resolved = first.and_then(|n| n["resolved"].as_bool()).unwrap_or(false);
    println!("  resolved: {}", resolved);

    if let Some(notes_arr) = notes {
        for note in notes_arr {
            let author = note["author"]["username"].as_str().unwrap_or("?");
            let body = note["body"].as_str().unwrap_or("");
            println!("  @{}: {}", author, body);
        }
    }
    println!();
}

#[allow(clippy::too_many_arguments)]
async fn handle_comment_inline(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    file: String,
    line: Option<u32>,
    old_line: Option<u32>,
    base_sha: String,
    head_sha: String,
    start_sha: String,
    old_file: Option<String>,
    message: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let body = read_message(message)?;
    if body.trim().is_empty() {
        bail!("Comment body is empty");
    }
    if line.is_none() && old_line.is_none() {
        bail!("Either --line or --old-line must be specified");
    }
    let position = build_inline_position(&file, old_file.as_deref(), line, old_line, &base_sha, &head_sha, &start_sha);
    let result = client.create_mr_discussion(iid, &body, &position).await?;
    let disc_id = result["id"].as_str().unwrap_or("?");
    println!(
        "Inline comment added to !{} at {}:{} (discussion {})",
        iid, file, line.or(old_line).unwrap_or(0), disc_id
    );
    Ok(())
}

fn build_inline_position(
    file: &str,
    old_file: Option<&str>,
    line: Option<u32>,
    old_line: Option<u32>,
    base_sha: &str,
    head_sha: &str,
    start_sha: &str,
) -> serde_json::Value {
    let old_path = old_file.unwrap_or(file);
    let mut position = serde_json::json!({
        "position_type": "text",
        "base_sha": base_sha,
        "head_sha": head_sha,
        "start_sha": start_sha,
        "new_path": file,
        "old_path": old_path,
    });
    if let Some(n) = line {
        position["new_line"] = serde_json::json!(n);
    }
    if let Some(n) = old_line {
        position["old_line"] = serde_json::json!(n);
    }
    position
}

async fn handle_reply(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    discussion: String,
    message: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let body = read_message(message)?;
    if body.trim().is_empty() {
        bail!("Reply body is empty");
    }
    let result = client.reply_to_discussion(iid, &discussion, &body).await?;
    let note_id = result["id"].as_u64().unwrap_or(0);
    println!(
        "Reply #{} added to discussion {} on !{}",
        note_id, discussion, iid
    );
    Ok(())
}

async fn handle_resolve(
    config: &mut Config,
    project: Option<&str>,
    iid: u64,
    discussion: String,
    unresolve: bool,
) -> Result<()> {
    let client = get_client(config, project).await?;
    let resolved = !unresolve;
    client
        .resolve_discussion(iid, &discussion, resolved)
        .await?;
    let action = if resolved { "Resolved" } else { "Unresolved" };
    println!("{} discussion {} on !{}", action, discussion, iid);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_create(
    config: &mut Config,
    project: Option<&str>,
    title: String,
    description: Option<String>,
    source: Option<String>,
    target: Option<String>,
    auto_merge: bool,
    keep_branch: bool,
) -> Result<()> {
    let source_branch = resolve_source_branch(source)?;
    let client = get_client(config, project).await?;
    let target_branch = resolve_target_branch(&client, target).await?;

    let result = client
        .create_merge_request(&title, &source_branch, &target_branch, description.as_deref())
        .await?;

    let iid = result["iid"].as_u64().unwrap_or(0);
    let web_url = result["web_url"].as_str().unwrap_or("");
    println!("Created !{}: {}", iid, title);
    println!("{}", web_url);

    if auto_merge {
        enable_automerge_after_create(&client, iid, keep_branch).await;
    }
    Ok(())
}

fn resolve_source_branch(source: Option<String>) -> Result<String> {
    if let Some(s) = source {
        return Ok(s);
    }
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current git branch")?;
    if !output.status.success() {
        bail!("Failed to get current git branch");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

async fn resolve_target_branch(client: &Client, target: Option<String>) -> Result<String> {
    if let Some(t) = target {
        return Ok(t);
    }
    let project_info = client.get_project().await?;
    Ok(project_info["default_branch"]
        .as_str()
        .unwrap_or("main")
        .to_string())
}

async fn enable_automerge_after_create(client: &Client, iid: u64, keep_branch: bool) {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    match client.set_automerge(iid, !keep_branch).await {
        Ok(_) => println!("Auto-merge enabled"),
        Err(e) => {
            eprintln!("Warning: Could not enable auto-merge: {}", e);
            eprintln!(
                "Pipeline may not have started yet. Run: gitlab mr automerge {}",
                iid
            );
        }
    }
}

fn read_message(message: Option<String>) -> Result<String> {
    match message {
        Some(m) => Ok(m),
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
