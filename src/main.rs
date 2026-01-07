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
}

#[derive(Subcommand)]
enum CiCommands {
    /// Show pipeline status
    Status {
        /// Pipeline ID (defaults to latest)
        #[arg(long)]
        id: Option<u64>,
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

        Commands::Mr { command } => match command {
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
                let result = client.set_automerge(iid, !keep_branch).await?;
                let title = result["title"].as_str().unwrap_or("");
                println!("Auto-merge enabled for !{}: {}", iid, title);
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
                    .create_merge_request(&title, &source_branch, &target_branch, description.as_deref())
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
        },

        Commands::Ci { command } => match command {
            CiCommands::Status { id, project } => {
                let client = get_client(&mut config, project.as_deref()).await?;
                let pipeline = if let Some(pid) = id {
                    client.get_pipeline(pid).await?
                } else {
                    let pipelines = client.list_pipelines(1).await?;
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
                    .create_issue(&title, description.as_deref(), labels.as_deref(), assignee.as_deref())
                    .await?;
                let iid = result["iid"].as_u64().unwrap_or(0);
                let web_url = result["web_url"].as_str().unwrap_or("");
                println!("Created issue #{}: {}", iid, title);
                println!("{}", web_url);
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
