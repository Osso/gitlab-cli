mod api;
mod auth;
mod config;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use api::{Client, IssueListParams, MrListParams};
use config::Config;

#[derive(Parser)]
#[command(name = "gitlab")]
#[command(about = "GitLab CLI for read-only operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure GitLab host, token, and default project
    Config {
        /// GitLab host URL (e.g., https://gitlab.com)
        #[arg(long)]
        host: Option<String>,
        /// Personal access token
        #[arg(long)]
        token: Option<String>,
        /// Default project (e.g., group/project)
        #[arg(long)]
        project: Option<String>,
    },
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Merge request commands
    Mr {
        #[command(subcommand)]
        command: MrCommands,
    },
    /// Issue commands
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    /// CI/CD commands
    Ci {
        #[command(subcommand)]
        command: CiCommands,
    },
    /// Group commands
    Group {
        #[command(subcommand)]
        command: GroupCommands,
    },
    /// Project commands
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Branch protection commands
    Branch {
        #[command(subcommand)]
        command: BranchCommands,
    },
    /// Webhook management commands
    Webhook {
        #[command(subcommand)]
        command: WebhookCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Authenticate with GitLab using OAuth2
    Login {
        /// OAuth2 application client ID (defaults to glab's client ID for gitlab.com)
        #[arg(long)]
        client_id: Option<String>,
        /// GitLab host URL (overrides configured host)
        #[arg(long)]
        host: Option<String>,
    },
    /// Show authentication status
    Status,
}

#[derive(Subcommand)]
enum MrCommands {
    /// List merge requests
    List {
        /// Filter by state: opened, closed, merged, all
        #[arg(long, short, default_value = "opened")]
        state: String,
        /// Filter by author username
        #[arg(long, short)]
        author: Option<String>,
        /// Filter by created after date (ISO 8601)
        #[arg(long)]
        created_after: Option<String>,
        /// Filter by created before date (ISO 8601)
        #[arg(long)]
        created_before: Option<String>,
        /// Filter by updated after date (ISO 8601)
        #[arg(long)]
        updated_after: Option<String>,
        /// Order by: created_at, updated_at, merged_at
        #[arg(long, short)]
        order_by: Option<String>,
        /// Sort direction: asc, desc
        #[arg(long)]
        sort: Option<String>,
        /// Number of results per page
        #[arg(long, short = 'n', default_value = "20")]
        per_page: u32,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Show merge request details
    Show {
        /// Merge request IID
        iid: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Set merge request to auto-merge when pipeline succeeds
    Automerge {
        /// Merge request IID
        iid: u64,
        /// Keep source branch after merge
        #[arg(long)]
        keep_branch: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Create a new merge request
    Create {
        /// Merge request title
        #[arg(long, short)]
        title: String,
        /// Merge request description
        #[arg(long, short)]
        description: Option<String>,
        /// Source branch (defaults to current branch)
        #[arg(long, short)]
        source: Option<String>,
        /// Target branch (defaults to default branch)
        #[arg(long)]
        target: Option<String>,
        /// Set to auto-merge when pipeline succeeds
        #[arg(long)]
        auto_merge: bool,
        /// Keep source branch after merge (only with --auto-merge)
        #[arg(long)]
        keep_branch: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Show merge request diff/changes
    Diff {
        /// Merge request IID
        iid: u64,
        /// Output as JSON instead of unified diff
        #[arg(long)]
        json: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Close a merge request
    Close {
        /// Merge request IID
        iid: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// List comments on a merge request
    Comments {
        /// Merge request IID
        iid: u64,
        /// Number of comments to show
        #[arg(long, short = 'n', default_value = "10")]
        per_page: u32,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Add a comment to a merge request
    Comment {
        /// Merge request IID
        iid: u64,
        /// Comment body (reads from stdin if not provided)
        #[arg(long, short)]
        message: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Approve a merge request
    Approve {
        /// Merge request IID
        iid: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// List discussion threads on a merge request
    Discussions {
        /// Merge request IID
        iid: u64,
        /// Only show unresolved threads
        #[arg(long, short)]
        unresolved: bool,
        /// Number of discussions to fetch
        #[arg(long, short = 'n', default_value = "50")]
        per_page: u32,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Post an inline comment on a specific line in a merge request diff
    CommentInline {
        /// Merge request IID
        iid: u64,
        /// File path (new_path in the diff)
        #[arg(long)]
        file: String,
        /// Line number on the new side of the diff
        #[arg(long)]
        line: Option<u32>,
        /// Line number on the old side of the diff
        #[arg(long)]
        old_line: Option<u32>,
        /// Base commit SHA
        #[arg(long)]
        base_sha: String,
        /// Head commit SHA
        #[arg(long)]
        head_sha: String,
        /// Start commit SHA (merge base)
        #[arg(long)]
        start_sha: String,
        /// Old file path (if renamed, defaults to --file)
        #[arg(long)]
        old_file: Option<String>,
        /// Comment body (reads from stdin if not provided)
        #[arg(long, short)]
        message: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Reply to a discussion thread on a merge request
    Reply {
        /// Merge request IID
        iid: u64,
        /// Discussion ID to reply to
        #[arg(long, short)]
        discussion: String,
        /// Reply message (reads from stdin if not provided)
        #[arg(long, short)]
        message: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum CiCommands {
    /// Show pipeline status
    Status {
        /// Pipeline ID (defaults to latest)
        #[arg(long)]
        id: Option<u64>,
        /// Branch name (defaults to current branch)
        #[arg(long, short)]
        branch: Option<String>,
        /// Merge request IID
        #[arg(long, short)]
        mr: Option<u64>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Wait for pipeline to complete
    Wait {
        /// Pipeline ID (defaults to latest)
        #[arg(long)]
        id: Option<u64>,
        /// Branch name (defaults to current branch)
        #[arg(long, short)]
        branch: Option<String>,
        /// Poll interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Show job logs
    Logs {
        /// Job name or ID
        job: String,
        /// Pipeline ID (defaults to latest)
        #[arg(long)]
        pipeline: Option<u64>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Retry a failed job or pipeline
    Retry {
        /// Job ID or pipeline ID to retry
        id: u64,
        /// Retry entire pipeline instead of a single job
        #[arg(long)]
        pipeline: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum GroupCommands {
    /// List group members
    Members {
        /// Group path (e.g., globalcomix)
        group: String,
        /// Number of results per page
        #[arg(long, short = 'n', default_value = "100")]
        per_page: u32,
        /// Show email addresses (requires admin access)
        #[arg(long, short)]
        email: bool,
    },
    /// List subgroups
    Subgroups {
        /// Group path (e.g., globalcomix)
        group: String,
        /// Number of results per page
        #[arg(long, short = 'n', default_value = "30")]
        per_page: u32,
    },
    /// Show group details
    Show {
        /// Group path (e.g., globalcomix)
        group: String,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Archive a project
    Archive {
        /// Project path (e.g., group/project)
        project: String,
    },
    /// Unarchive a project
    Unarchive {
        /// Project path (e.g., group/project)
        project: String,
    },
    /// List projects in a group
    List {
        /// Group path (e.g., globalcomix)
        group: String,
        /// Include archived projects (excluded by default)
        #[arg(long, short)]
        archived: bool,
        /// Number of results per page
        #[arg(long, short = 'n', default_value = "50")]
        per_page: u32,
    },
    /// Manage push mirrors
    Mirrors {
        #[command(subcommand)]
        command: MirrorCommands,
    },
}

#[derive(Subcommand)]
enum MirrorCommands {
    /// List push mirrors for a project
    List {
        /// Project path (e.g., group/project)
        project: String,
    },
    /// Create push mirror to another Git host (SSH - requires UI to complete setup)
    Create {
        /// Project path (e.g., group/project)
        project: String,
        /// Target repository SSH URL (e.g., ssh://git@github.com/org/repo.git)
        url: String,
        /// Only mirror protected branches
        #[arg(long)]
        only_protected: bool,
    },
    /// Create push mirror with HTTPS and password/token authentication
    CreateHttps {
        /// Project path (e.g., group/project)
        project: String,
        /// Target repository HTTPS URL (e.g., https://github.com/org/repo.git)
        url: String,
        /// Username for authentication
        #[arg(long, short)]
        user: String,
        /// Password or token for authentication
        #[arg(long, short = 'P')]
        password: String,
        /// Only mirror protected branches
        #[arg(long)]
        only_protected: bool,
    },
    /// Remove a push mirror
    Remove {
        /// Project path (e.g., group/project)
        project: String,
        /// Mirror ID to remove
        mirror_id: u64,
    },
    /// Trigger a push mirror sync
    Sync {
        /// Project path (e.g., group/project)
        project: String,
        /// Mirror ID to sync
        mirror_id: u64,
    },
}

#[derive(Subcommand)]
enum BranchCommands {
    /// List protected branches
    List {
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Protect a branch
    Protect {
        /// Branch name to protect
        branch: String,
        /// Allow force push
        #[arg(long)]
        allow_force_push: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Unprotect a branch
    Unprotect {
        /// Branch name to unprotect
        branch: String,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum WebhookCommands {
    /// List webhooks for a project
    List {
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Show webhook details
    Show {
        /// Webhook ID
        id: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Create a new webhook
    Create {
        /// Webhook URL
        #[arg(long, short)]
        url: String,
        /// Secret token for webhook verification
        #[arg(long, short)]
        token: Option<String>,
        /// Enable push events
        #[arg(long)]
        push: bool,
        /// Enable merge request events
        #[arg(long)]
        merge_request: bool,
        /// Enable issue events
        #[arg(long)]
        issue: bool,
        /// Enable pipeline events
        #[arg(long)]
        pipeline: bool,
        /// Enable tag push events
        #[arg(long)]
        tag: bool,
        /// Enable note (comment) events
        #[arg(long)]
        note: bool,
        /// Enable job events
        #[arg(long)]
        job: bool,
        /// Enable release events
        #[arg(long)]
        release: bool,
        /// Enable SSL verification
        #[arg(long, default_value = "true")]
        ssl_verification: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Update an existing webhook
    Update {
        /// Webhook ID
        id: u64,
        /// Webhook URL
        #[arg(long, short)]
        url: Option<String>,
        /// Secret token for webhook verification
        #[arg(long, short)]
        token: Option<String>,
        /// Enable push events
        #[arg(long)]
        push: Option<bool>,
        /// Enable merge request events
        #[arg(long)]
        merge_request: Option<bool>,
        /// Enable issue events
        #[arg(long)]
        issue: Option<bool>,
        /// Enable pipeline events
        #[arg(long)]
        pipeline: Option<bool>,
        /// Enable tag push events
        #[arg(long)]
        tag: Option<bool>,
        /// Enable note (comment) events
        #[arg(long)]
        note: Option<bool>,
        /// Enable job events
        #[arg(long)]
        job: Option<bool>,
        /// Enable release events
        #[arg(long)]
        release: Option<bool>,
        /// Enable SSL verification
        #[arg(long)]
        ssl_verification: Option<bool>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Delete a webhook
    Delete {
        /// Webhook ID
        id: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Test a webhook by sending a test event
    Test {
        /// Webhook ID
        id: u64,
        /// Event type to test (push, tag_push, note, issue, merge_request, etc.)
        #[arg(long, short, default_value = "push")]
        event: String,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum IssueCommands {
    /// List issues
    List {
        /// Filter by state: opened, closed, all
        #[arg(long, short, default_value = "opened")]
        state: String,
        /// Filter by author username
        #[arg(long, short)]
        author: Option<String>,
        /// Filter by assignee username
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by labels (comma-separated)
        #[arg(long, short)]
        labels: Option<String>,
        /// Search in title and description
        #[arg(long)]
        search: Option<String>,
        /// Filter by created after date (ISO 8601)
        #[arg(long)]
        created_after: Option<String>,
        /// Number of results per page
        #[arg(long, short = 'n', default_value = "20")]
        per_page: u32,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Show issue details
    Show {
        /// Issue IID
        iid: u64,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Create a new issue
    Create {
        /// Issue title
        #[arg(long, short)]
        title: String,
        /// Issue description
        #[arg(long, short)]
        description: Option<String>,
        /// Labels (comma-separated)
        #[arg(long, short)]
        labels: Option<String>,
        /// Assignee username
        #[arg(long, short)]
        assignee: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

async fn get_client(config: &mut Config, project_override: Option<&str>) -> Result<Client> {
    // Check if OAuth2 token needs refresh
    if let Some(oauth2) = &config.oauth2 {
        if oauth2.is_expired() {
            eprintln!("Token expired, refreshing...");
            auth::refresh_token(config).await?;
        }
    }

    let token = config.get_access_token().ok_or_else(|| {
        anyhow::anyhow!("No token configured. Run: gitlab auth login --client-id <id>")
    })?;

    let project = project_override
        .map(|s| s.to_string())
        .or_else(|| config.project.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No project specified. Use --project or run: gitlab config --project <project>"
            )
        })?;

    Client::new(config.host(), token, &project)
}

/// Get a client for group operations (doesn't require a project)
async fn get_group_client(config: &mut Config) -> Result<Client> {
    // Check if OAuth2 token needs refresh
    if let Some(oauth2) = &config.oauth2 {
        if oauth2.is_expired() {
            eprintln!("Token expired, refreshing...");
            auth::refresh_token(config).await?;
        }
    }

    let token = config.get_access_token().ok_or_else(|| {
        anyhow::anyhow!("No token configured. Run: gitlab auth login --client-id <id>")
    })?;

    // Use a dummy project since group operations don't need it
    Client::new(config.host(), token, "_")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Config {
            host,
            token,
            project,
        } => {
            if host.is_none() && token.is_none() && project.is_none() {
                println!("Current configuration:");
                println!("  host: {}", config.host());
                println!(
                    "  token: {}",
                    config
                        .token
                        .as_ref()
                        .map(|t| format!("{}...", &t[..8.min(t.len())]))
                        .unwrap_or_else(|| "(not set)".to_string())
                );
                println!(
                    "  project: {}",
                    config.project.as_deref().unwrap_or("(not set)")
                );
                return Ok(());
            }
            if let Some(h) = host {
                config.host = Some(h);
            }
            if let Some(t) = token {
                config.token = Some(t);
            }
            if let Some(p) = project {
                config.project = Some(p);
            }
            config.save()?;
            println!("Configuration saved.");
        }

        Commands::Auth { command } => match command {
            AuthCommands::Login { client_id, host } => {
                let auth_host = host.as_deref().unwrap_or_else(|| config.host());
                let cid = client_id.as_deref().unwrap_or(auth::default_client_id());
                let flow = auth::AuthFlow::new(auth_host, cid);

                let auth_url = flow.authorization_url();
                println!("Opening browser for authorization...");
                println!("If browser doesn't open, visit: {}", auth_url);

                if let Err(e) = open::that(&auth_url) {
                    eprintln!("Failed to open browser: {}", e);
                }

                let code = flow.wait_for_callback()?;
                println!("Authorization code received, exchanging for token...");

                let oauth2_config = flow.exchange_code(&code).await?;
                config.oauth2 = Some(oauth2_config);
                config.token = None; // Clear old static token
                if host.is_some() {
                    config.host = host;
                }
                config.save()?;
                println!("Authentication successful!");
            }
            AuthCommands::Status => {
                if let Some(oauth2) = &config.oauth2 {
                    println!("OAuth2 authenticated");
                    println!(
                        "  client_id: {}...",
                        &oauth2.client_id[..8.min(oauth2.client_id.len())]
                    );
                    println!("  expires_at: {}", oauth2.expires_at);
                    println!("  expired: {}", oauth2.is_expired());
                } else if config.token.is_some() {
                    println!("Using static token (legacy)");
                } else {
                    println!("Not authenticated");
                }
            }
        },

        Commands::Mr { command } => {
            match command {
                MrCommands::List {
                    state,
                    author,
                    created_after,
                    created_before,
                    updated_after,
                    order_by,
                    sort,
                    per_page,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let params = MrListParams {
                        per_page,
                        state,
                        author_username: author,
                        created_after,
                        created_before,
                        updated_after,
                        order_by,
                        sort,
                    };
                    let result = client.list_merge_requests(&params).await?;
                    print_mrs(&result);
                }
                MrCommands::Show { iid, project } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let result = client.get_merge_request(iid).await?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                MrCommands::Automerge {
                    iid,
                    keep_branch,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let max_retries = 3;
                    let mut last_error = None;

                    for attempt in 0..max_retries {
                        match client.set_automerge(iid, !keep_branch).await {
                            Ok(result) => {
                                let title = result["title"].as_str().unwrap_or("");
                                println!("Auto-merge enabled for !{}: {}", iid, title);
                                last_error = None;
                                break;
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
                }
                MrCommands::Diff { iid, json, project } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let result = client.get_merge_request_changes(iid).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        // Output as unified diff format
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
                }
                MrCommands::Close { iid, project } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let result = client
                        .update_merge_request(iid, &serde_json::json!({"state_event": "close"}))
                        .await?;
                    let title = result["title"].as_str().unwrap_or("");
                    println!("Closed !{}: {}", iid, title);
                }
                MrCommands::Comments {
                    iid,
                    per_page,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let notes = client.list_mr_notes(iid, per_page).await?;
                    if let Some(arr) = notes.as_array() {
                        if arr.is_empty() {
                            println!("No comments on !{}", iid);
                        } else {
                            for note in arr {
                                let system = note["system"].as_bool().unwrap_or(false);
                                if system {
                                    continue;
                                }
                                let id = note["id"].as_u64().unwrap_or(0);
                                let author = note["author"]["username"].as_str().unwrap_or("?");
                                let created = note["created_at"].as_str().unwrap_or("?");
                                let body = note["body"].as_str().unwrap_or("");
                                println!("--- #{} by @{} ({})", id, author, created);
                                println!("{}", body);
                                println!();
                            }
                        }
                    }
                }
                MrCommands::Comment {
                    iid,
                    message,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let body = match message {
                        Some(m) => m,
                        None => {
                            use std::io::Read;
                            let mut buf = String::new();
                            std::io::stdin().read_to_string(&mut buf)?;
                            buf
                        }
                    };
                    if body.trim().is_empty() {
                        bail!("Comment body is empty");
                    }
                    let result = client.create_mr_note(iid, &body).await?;
                    let note_id = result["id"].as_u64().unwrap_or(0);
                    println!("Comment #{} added to !{}", note_id, iid);
                }
                MrCommands::Approve { iid, project } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    client.approve_merge_request(iid).await?;
                    println!("Approved !{}", iid);
                }
                MrCommands::Discussions {
                    iid,
                    unresolved,
                    per_page,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let discussions = client.list_mr_discussions(iid, per_page).await?;
                    if let Some(arr) = discussions.as_array() {
                        let threads: Vec<_> = arr
                            .iter()
                            .filter(|d| {
                                // Skip individual notes (non-threaded)
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
                                    // Keep only threads with at least one unresolved note
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
                            })
                            .collect();

                        if threads.is_empty() {
                            let qualifier = if unresolved { "unresolved " } else { "" };
                            println!("No {}discussion threads on !{}", qualifier, iid);
                        } else {
                            for d in &threads {
                                let disc_id = d["id"].as_str().unwrap_or("?");
                                let notes = d["notes"].as_array();
                                let first = notes.and_then(|n| n.first());

                                // File position info
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

                                let resolved =
                                    first.and_then(|n| n["resolved"].as_bool()).unwrap_or(false);
                                println!("  resolved: {}", resolved);

                                if let Some(notes_arr) = notes {
                                    for note in notes_arr {
                                        let author =
                                            note["author"]["username"].as_str().unwrap_or("?");
                                        let body = note["body"].as_str().unwrap_or("");
                                        println!("  @{}: {}", author, body);
                                    }
                                }
                                println!();
                            }
                        }
                    }
                }
                MrCommands::CommentInline {
                    iid,
                    file,
                    line,
                    old_line,
                    base_sha,
                    head_sha,
                    start_sha,
                    old_file,
                    message,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let body = match message {
                        Some(m) => m,
                        None => {
                            use std::io::Read;
                            let mut buf = String::new();
                            std::io::stdin().read_to_string(&mut buf)?;
                            buf
                        }
                    };
                    if body.trim().is_empty() {
                        bail!("Comment body is empty");
                    }
                    if line.is_none() && old_line.is_none() {
                        bail!("Either --line or --old-line must be specified");
                    }
                    let old_path = old_file.as_deref().unwrap_or(&file);
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
                    let result = client.create_mr_discussion(iid, &body, &position).await?;
                    let disc_id = result["id"].as_str().unwrap_or("?");
                    println!(
                        "Inline comment added to !{} at {}:{} (discussion {})",
                        iid,
                        file,
                        line.or(old_line).unwrap_or(0),
                        disc_id
                    );
                }
                MrCommands::Reply {
                    iid,
                    discussion,
                    message,
                    project,
                } => {
                    let client = get_client(&mut config, project.as_deref()).await?;
                    let body = match message {
                        Some(m) => m,
                        None => {
                            use std::io::Read;
                            let mut buf = String::new();
                            std::io::stdin().read_to_string(&mut buf)?;
                            buf
                        }
                    };
                    if body.trim().is_empty() {
                        bail!("Reply body is empty");
                    }
                    let result = client.reply_to_discussion(iid, &discussion, &body).await?;
                    let note_id = result["id"].as_u64().unwrap_or(0);
                    println!(
                        "Reply #{} added to discussion {} on !{}",
                        note_id, discussion, iid
                    );
                }
                MrCommands::Create {
                    title,
                    description,
                    source,
                    target,
                    auto_merge,
                    keep_branch,
                    project,
                } => {
                    // Get source branch from git if not provided
                    let source_branch = if let Some(s) = source {
                        s
                    } else {
                        let output = std::process::Command::new("git")
                            .args(["rev-parse", "--abbrev-ref", "HEAD"])
                            .output()
                            .context("Failed to get current git branch")?;
                        if !output.status.success() {
                            bail!("Failed to get current git branch");
                        }
                        String::from_utf8(output.stdout)?.trim().to_string()
                    };

                    let client = get_client(&mut config, project.as_deref()).await?;

                    // Get target branch from project default if not provided
                    let target_branch = if let Some(t) = target {
                        t
                    } else {
                        let project_info = client.get_project().await?;
                        project_info["default_branch"]
                            .as_str()
                            .unwrap_or("main")
                            .to_string()
                    };

                    let result = client
                        .create_merge_request(
                            &title,
                            &source_branch,
                            &target_branch,
                            description.as_deref(),
                        )
                        .await?;

                    let iid = result["iid"].as_u64().unwrap_or(0);
                    let web_url = result["web_url"].as_str().unwrap_or("");

                    println!("Created !{}: {}", iid, title);
                    println!("{}", web_url);

                    if auto_merge {
                        // Wait briefly for pipeline to start
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                        match client.set_automerge(iid, !keep_branch).await {
                            Ok(_) => println!("Auto-merge enabled"),
                            Err(e) => {
                                eprintln!("Warning: Could not enable auto-merge: {}", e);
                                eprintln!("Pipeline may not have started yet. Run: gitlab mr automerge {}", iid);
                            }
                        }
                    }
                }
            }
        }

        Commands::Ci { command } => match command {
            CiCommands::Status {
                id,
                branch,
                mr,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let pipeline = if let Some(pid) = id {
                    client.get_pipeline(pid).await?
                } else if let Some(mr_iid) = mr {
                    let pipelines = client.list_mr_pipelines(mr_iid).await?;
                    let arr = pipelines
                        .as_array()
                        .ok_or_else(|| anyhow::anyhow!("No pipelines found for MR !{}", mr_iid))?;
                    if arr.is_empty() {
                        bail!("No pipelines found for MR !{}", mr_iid);
                    }
                    arr[0].clone()
                } else {
                    let pipelines = client
                        .list_pipelines_for_branch(branch.as_deref(), 1)
                        .await?;
                    let arr = pipelines
                        .as_array()
                        .ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;
                    if arr.is_empty() {
                        bail!("No pipelines found");
                    }
                    arr[0].clone()
                };

                let pipeline_id = pipeline["id"].as_u64().unwrap();
                let jobs = client.list_pipeline_jobs(pipeline_id).await?;

                println!(
                    "Pipeline #{} - {} ({})",
                    pipeline["id"],
                    pipeline["status"].as_str().unwrap_or("unknown"),
                    pipeline["ref"].as_str().unwrap_or("")
                );
                println!();

                if let Some(jobs_arr) = jobs.as_array() {
                    for job in jobs_arr {
                        println!(
                            "  {} - {} ({})",
                            job["name"].as_str().unwrap_or("?"),
                            job["status"].as_str().unwrap_or("?"),
                            job["stage"].as_str().unwrap_or("?")
                        );
                    }
                }
            }
            CiCommands::Wait {
                id,
                branch,
                interval,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;

                loop {
                    let pipeline = if let Some(pid) = id {
                        client.get_pipeline(pid).await?
                    } else {
                        let pipelines = client
                            .list_pipelines_for_branch(branch.as_deref(), 1)
                            .await?;
                        let arr = pipelines
                            .as_array()
                            .ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;
                        if arr.is_empty() {
                            bail!("No pipelines found");
                        }
                        arr[0].clone()
                    };

                    let status = pipeline["status"].as_str().unwrap_or("unknown");
                    let pipeline_ref = pipeline["ref"].as_str().unwrap_or("");
                    let pipeline_id = pipeline["id"].as_u64().unwrap();

                    // Print current status
                    eprintln!("Pipeline #{} - {} ({})", pipeline_id, status, pipeline_ref);

                    // Check if pipeline is finished
                    match status {
                        "success" => {
                            println!("Pipeline #{} succeeded", pipeline_id);
                            break;
                        }
                        "failed" | "canceled" | "skipped" => {
                            bail!("Pipeline #{} {}", pipeline_id, status);
                        }
                        "running"
                        | "pending"
                        | "created"
                        | "waiting_for_resource"
                        | "preparing"
                        | "scheduled" => {
                            // Still running, wait and check again
                            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
                        }
                        _ => {
                            bail!("Unknown pipeline status: {}", status);
                        }
                    }
                }
            }
            CiCommands::Logs {
                job,
                pipeline,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;

                let pipeline_id = if let Some(pid) = pipeline {
                    pid
                } else {
                    let pipelines = client.list_pipelines(1).await?;
                    let arr = pipelines
                        .as_array()
                        .ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;
                    if arr.is_empty() {
                        bail!("No pipelines found");
                    }
                    arr[0]["id"]
                        .as_u64()
                        .ok_or_else(|| anyhow::anyhow!("Invalid pipeline ID"))?
                };

                let jobs = client.list_pipeline_jobs(pipeline_id).await?;
                let jobs_arr = jobs
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("No jobs found"))?;

                // Find job by name or ID
                let job_id: u64 = if let Ok(id) = job.parse::<u64>() {
                    id
                } else {
                    jobs_arr
                        .iter()
                        .find(|j| j["name"].as_str() == Some(&job))
                        .and_then(|j| j["id"].as_u64())
                        .ok_or_else(|| {
                            anyhow::anyhow!("Job '{}' not found in pipeline {}", job, pipeline_id)
                        })?
                };

                let log = client.get_job_log(job_id).await?;
                println!("{}", log);
            }
            CiCommands::Retry {
                id,
                pipeline,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;

                if pipeline {
                    let result = client.retry_pipeline(id).await?;
                    let new_pipeline_id = result["id"].as_u64().unwrap_or(id);
                    let web_url = result["web_url"].as_str().unwrap_or("");
                    println!("Pipeline #{} retried", new_pipeline_id);
                    if !web_url.is_empty() {
                        println!("{}", web_url);
                    }
                } else {
                    let result = client.retry_job(id).await?;
                    let job_name = result["name"].as_str().unwrap_or("unknown");
                    let job_id = result["id"].as_u64().unwrap_or(id);
                    let web_url = result["web_url"].as_str().unwrap_or("");
                    println!("Job '{}' (#{}) retried", job_name, job_id);
                    if !web_url.is_empty() {
                        println!("{}", web_url);
                    }
                }
            }
        },

        Commands::Issue { command } => match command {
            IssueCommands::List {
                state,
                author,
                assignee,
                labels,
                search,
                created_after,
                per_page,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let params = IssueListParams {
                    per_page,
                    state,
                    author_username: author,
                    assignee_username: assignee,
                    labels,
                    search,
                    created_after,
                };
                let result = client.list_issues(&params).await?;
                print_issues(&result);
            }
            IssueCommands::Show { iid, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let result = client.get_issue(iid).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            IssueCommands::Create {
                title,
                description,
                labels,
                assignee,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
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
            }
        },

        Commands::Group { command } => match command {
            GroupCommands::Members {
                group,
                per_page,
                email,
            } => {
                let client = get_group_client(&mut config).await?;
                let result = client.list_group_members(&group, per_page, email).await?;
                print_group_members(&result, email);
            }
            GroupCommands::Subgroups { group, per_page } => {
                let client = get_group_client(&mut config).await?;
                let result = client.list_group_subgroups(&group, per_page).await?;
                print_subgroups(&result);
            }
            GroupCommands::Show { group } => {
                let client = get_group_client(&mut config).await?;
                let result = client.get_group(&group).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },

        Commands::Project { command } => match command {
            ProjectCommands::Archive { project } => {
                let client = get_group_client(&mut config).await?;
                let result = client.archive_project(&project).await?;
                let name = result["path_with_namespace"].as_str().unwrap_or(&project);
                println!("Archived: {}", name);
            }
            ProjectCommands::Unarchive { project } => {
                let client = get_group_client(&mut config).await?;
                let result = client.unarchive_project(&project).await?;
                let name = result["path_with_namespace"].as_str().unwrap_or(&project);
                println!("Unarchived: {}", name);
            }
            ProjectCommands::List {
                group,
                archived,
                per_page,
            } => {
                let client = get_group_client(&mut config).await?;
                let result = client
                    .list_group_projects(&group, per_page, archived)
                    .await?;
                print_projects(&result);
            }
            ProjectCommands::Mirrors { command } => match command {
                MirrorCommands::List { project } => {
                    let client = get_group_client(&mut config).await?;
                    let result = client.list_push_mirrors(&project).await?;
                    print_mirrors(&result);
                }
                MirrorCommands::Create {
                    project,
                    url,
                    only_protected,
                } => {
                    let client = get_group_client(&mut config).await?;
                    let result = client
                        .create_push_mirror(&project, &url, true, only_protected)
                        .await?;
                    let id = result["id"].as_u64().unwrap_or(0);
                    let mirror_url = result["url"].as_str().unwrap_or(&url);
                    println!("Created push mirror (id: {}) -> {}", id, mirror_url);

                    // Fetch SSH public key via dedicated endpoint
                    if let Ok(ssh_key) = client.get_push_mirror_public_key(&project, id).await {
                        if !ssh_key.is_empty() {
                            println!("\nSSH public key (add as deploy key with write access):");
                            println!("{}", ssh_key);
                        }
                    }

                    // Note about host keys
                    println!(
                        "\nNOTE: Go to GitLab UI and click 'Detect host keys' to complete setup:"
                    );
                    println!(
                        "https://gitlab.com/{}/-/settings/repository#js-push-remote-settings",
                        project
                    );
                }
                MirrorCommands::CreateHttps {
                    project,
                    url,
                    user,
                    password,
                    only_protected,
                } => {
                    let client = get_group_client(&mut config).await?;
                    let result = client
                        .create_push_mirror_https(&project, &url, &user, &password, only_protected)
                        .await?;
                    let id = result["id"].as_u64().unwrap_or(0);
                    let mirror_url = result["url"].as_str().unwrap_or(&url);
                    println!("Created HTTPS push mirror (id: {}) -> {}", id, mirror_url);
                }
                MirrorCommands::Remove { project, mirror_id } => {
                    let client = get_group_client(&mut config).await?;
                    client.delete_push_mirror(&project, mirror_id).await?;
                    println!("Removed mirror {}", mirror_id);
                }
                MirrorCommands::Sync { project, mirror_id } => {
                    let client = get_group_client(&mut config).await?;
                    client.sync_push_mirror(&project, mirror_id).await?;
                    println!("Triggered sync for mirror {}", mirror_id);
                }
            },
        },

        Commands::Webhook { command } => match command {
            WebhookCommands::List { project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let result = client.list_webhooks().await?;
                print_webhooks(&result);
            }
            WebhookCommands::Show { id, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let result = client.get_webhook(id).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WebhookCommands::Create {
                url,
                token,
                push,
                merge_request,
                issue,
                pipeline,
                tag,
                note,
                job,
                release,
                ssl_verification,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let params = api::WebhookCreateParams {
                    url,
                    token,
                    push_events: push,
                    merge_requests_events: merge_request,
                    issues_events: issue,
                    pipeline_events: pipeline,
                    tag_push_events: tag,
                    note_events: note,
                    job_events: job,
                    releases_events: release,
                    enable_ssl_verification: ssl_verification,
                };
                let result = client.create_webhook(&params).await?;
                let hook_id = result["id"].as_u64().unwrap_or(0);
                let hook_url = result["url"].as_str().unwrap_or("");
                println!("Created webhook {} -> {}", hook_id, hook_url);
            }
            WebhookCommands::Update {
                id,
                url,
                token,
                push,
                merge_request,
                issue,
                pipeline,
                tag,
                note,
                job,
                release,
                ssl_verification,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let params = api::WebhookUpdateParams {
                    url,
                    token,
                    push_events: push,
                    merge_requests_events: merge_request,
                    issues_events: issue,
                    pipeline_events: pipeline,
                    tag_push_events: tag,
                    note_events: note,
                    job_events: job,
                    releases_events: release,
                    enable_ssl_verification: ssl_verification,
                };
                let result = client.update_webhook(id, &params).await?;
                let hook_url = result["url"].as_str().unwrap_or("");
                println!("Updated webhook {} -> {}", id, hook_url);
            }
            WebhookCommands::Delete { id, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                client.delete_webhook(id).await?;
                println!("Deleted webhook {}", id);
            }
            WebhookCommands::Test { id, event, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                client.test_webhook(id, &event).await?;
                println!("Sent test {} event to webhook {}", event, id);
            }
        },

        Commands::Branch { command } => match command {
            BranchCommands::List { project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let result = client.list_protected_branches().await?;
                print_protected_branches(&result);
            }
            BranchCommands::Protect {
                branch,
                allow_force_push,
                project,
            } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                client.protect_branch(&branch, allow_force_push).await?;
                println!("Protected branch: {}", branch);
            }
            BranchCommands::Unprotect { branch, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                client.unprotect_branch(&branch).await?;
                println!("Unprotected branch: {}", branch);
            }
        },
    }

    Ok(())
}

fn print_mrs(value: &serde_json::Value) {
    if let Some(mrs) = value.as_array() {
        for mr in mrs {
            let iid = mr["iid"].as_u64().unwrap_or(0);
            let title = mr["title"].as_str().unwrap_or("");
            let state = mr["state"].as_str().unwrap_or("");
            let source = mr["source_branch"].as_str().unwrap_or("");
            let target = mr["target_branch"].as_str().unwrap_or("");
            let author = mr["author"]["username"].as_str().unwrap_or("");

            println!("!{:<5} {} [{}]", iid, title, state);
            println!("       {} -> {} (@{})", source, target, author);
        }
    }
}

fn print_issues(value: &serde_json::Value) {
    if let Some(issues) = value.as_array() {
        for issue in issues {
            let iid = issue["iid"].as_u64().unwrap_or(0);
            let title = issue["title"].as_str().unwrap_or("");
            let state = issue["state"].as_str().unwrap_or("");
            let author = issue["author"]["username"].as_str().unwrap_or("");
            let labels: Vec<&str> = issue["labels"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|l| l.as_str()).collect())
                .unwrap_or_default();

            println!("#{:<5} {} [{}]", iid, title, state);
            if labels.is_empty() {
                println!("       @{}", author);
            } else {
                println!("       @{} | {}", author, labels.join(", "));
            }
        }
    }
}

fn access_level_name(level: u64) -> &'static str {
    match level {
        10 => "Guest",
        20 => "Reporter",
        30 => "Developer",
        40 => "Maintainer",
        50 => "Owner",
        _ => "Unknown",
    }
}

fn print_group_members(value: &serde_json::Value, show_email: bool) {
    if let Some(members) = value.as_array() {
        if members.is_empty() {
            println!("No members found");
            return;
        }
        for member in members {
            let username = member["username"].as_str().unwrap_or("");
            let name = member["name"].as_str().unwrap_or("");
            let access_level = member["access_level"].as_u64().unwrap_or(0);
            let access = access_level_name(access_level);
            if show_email {
                let email = member["email"].as_str().unwrap_or("-");
                println!("{:<25} {:<12} {:<35} {}", username, access, email, name);
            } else {
                println!("{:<25} {:<12} {}", username, access, name);
            }
        }
    }
}

fn print_subgroups(value: &serde_json::Value) {
    if let Some(groups) = value.as_array() {
        if groups.is_empty() {
            println!("No subgroups found");
            return;
        }
        for group in groups {
            let path = group["full_path"].as_str().unwrap_or("");
            let name = group["name"].as_str().unwrap_or("");
            let visibility = group["visibility"].as_str().unwrap_or("");
            println!("{:<40} {:<10} {}", path, visibility, name);
        }
    }
}

fn print_projects(value: &serde_json::Value) {
    if let Some(projects) = value.as_array() {
        if projects.is_empty() {
            println!("No projects found");
            return;
        }
        for project in projects {
            let path = project["path_with_namespace"].as_str().unwrap_or("");
            let visibility = project["visibility"].as_str().unwrap_or("");
            let archived = project["archived"].as_bool().unwrap_or(false);
            let default_branch = project["default_branch"].as_str().unwrap_or("-");
            let status = if archived { "[archived]" } else { "" };
            println!(
                "{:<45} {:<10} {:<10} {}",
                path, visibility, default_branch, status
            );
        }
    }
}

fn print_mirrors(value: &serde_json::Value) {
    if let Some(mirrors) = value.as_array() {
        if mirrors.is_empty() {
            println!("No push mirrors configured");
            return;
        }
        for mirror in mirrors {
            let id = mirror["id"].as_u64().unwrap_or(0);
            let url = mirror["url"].as_str().unwrap_or("");
            let enabled = mirror["enabled"].as_bool().unwrap_or(false);
            let only_protected = mirror["only_protected_branches"].as_bool().unwrap_or(false);
            let auth_method = mirror["auth_method"].as_str().unwrap_or("password");
            let last_update = mirror["last_update_at"].as_str().unwrap_or("-");
            let last_error = mirror["last_error"].as_str();

            let status = if enabled { "enabled" } else { "disabled" };
            let protected = if only_protected {
                "[protected-only]"
            } else {
                "[all-branches]"
            };

            println!("{:<6} {:<10} {} {}", id, status, protected, url);
            println!("       Auth: {}", auth_method);
            if let Some(err) = last_error {
                if !err.is_empty() {
                    println!("       Last error: {}", err);
                }
            }
            println!("       Last sync: {}", last_update);

            // Show SSH public key if using ssh_public_key auth
            if auth_method == "ssh_public_key" {
                if let Some(ssh_key) = mirror["ssh_public_key"].as_str() {
                    println!("       SSH key: {}", ssh_key);
                }
            }
        }
    }
}

fn print_webhooks(value: &serde_json::Value) {
    if let Some(hooks) = value.as_array() {
        if hooks.is_empty() {
            println!("No webhooks configured");
            return;
        }
        for hook in hooks {
            let id = hook["id"].as_u64().unwrap_or(0);
            let url = hook["url"].as_str().unwrap_or("");
            let ssl = hook["enable_ssl_verification"].as_bool().unwrap_or(true);

            // Collect enabled events
            let mut events = vec![];
            if hook["push_events"].as_bool().unwrap_or(false) {
                events.push("push");
            }
            if hook["merge_requests_events"].as_bool().unwrap_or(false) {
                events.push("merge_request");
            }
            if hook["issues_events"].as_bool().unwrap_or(false) {
                events.push("issue");
            }
            if hook["pipeline_events"].as_bool().unwrap_or(false) {
                events.push("pipeline");
            }
            if hook["tag_push_events"].as_bool().unwrap_or(false) {
                events.push("tag");
            }
            if hook["note_events"].as_bool().unwrap_or(false) {
                events.push("note");
            }
            if hook["job_events"].as_bool().unwrap_or(false) {
                events.push("job");
            }
            if hook["releases_events"].as_bool().unwrap_or(false) {
                events.push("release");
            }

            let ssl_status = if ssl { "" } else { " [ssl-off]" };
            println!("{:<6} {}{}", id, url, ssl_status);
            if !events.is_empty() {
                println!("       Events: {}", events.join(", "));
            }
        }
    }
}

fn print_protected_branches(value: &serde_json::Value) {
    if let Some(branches) = value.as_array() {
        if branches.is_empty() {
            println!("No protected branches");
            return;
        }
        for branch in branches {
            let name = branch["name"].as_str().unwrap_or("");
            let allow_force_push = branch["allow_force_push"].as_bool().unwrap_or(false);
            let force_push_str = if allow_force_push {
                "[force-push-allowed]"
            } else {
                ""
            };
            println!("{} {}", name, force_push_str);
        }
    }
}
