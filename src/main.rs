mod api;
mod config;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

use api::{Client, MrListParams};
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
    /// Merge request commands
    Mr {
        #[command(subcommand)]
        command: MrCommands,
    },
    /// CI/CD commands
    Ci {
        #[command(subcommand)]
        command: CiCommands,
    },
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

fn get_client(config: &Config, project_override: Option<&str>) -> Result<Client> {
    let token = config
        .token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No token configured. Run: gitlab config --token <token>"))?;

    let project = project_override
        .map(|s| s.to_string())
        .or_else(|| config.project.clone())
        .ok_or_else(|| {
            anyhow::anyhow!("No project specified. Use --project or run: gitlab config --project <project>")
        })?;

    Client::new(config.host(), token, &project)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Config { host, token, project } => {
            if host.is_none() && token.is_none() && project.is_none() {
                println!("Current configuration:");
                println!("  host: {}", config.host());
                println!(
                    "  token: {}",
                    config.token.as_ref().map(|t| format!("{}...", &t[..8.min(t.len())])).unwrap_or_else(|| "(not set)".to_string())
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
                let client = get_client(&config, project.as_deref())?;
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
                let client = get_client(&config, project.as_deref())?;
                let result = client.get_merge_request(iid).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },

        Commands::Ci { command } => match command {
            CiCommands::Status { id, project } => {
                let client = get_client(&config, project.as_deref())?;
                let pipeline = if let Some(pid) = id {
                    client.get_pipeline(pid).await?
                } else {
                    let pipelines = client.list_pipelines(1).await?;
                    let arr = pipelines.as_array().ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;
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
            CiCommands::Logs { job, pipeline, project } => {
                let client = get_client(&config, project.as_deref())?;

                let pipeline_id = if let Some(pid) = pipeline {
                    pid
                } else {
                    let pipelines = client.list_pipelines(1).await?;
                    let arr = pipelines.as_array().ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;
                    if arr.is_empty() {
                        bail!("No pipelines found");
                    }
                    arr[0]["id"].as_u64().ok_or_else(|| anyhow::anyhow!("Invalid pipeline ID"))?
                };

                let jobs = client.list_pipeline_jobs(pipeline_id).await?;
                let jobs_arr = jobs.as_array().ok_or_else(|| anyhow::anyhow!("No jobs found"))?;

                // Find job by name or ID
                let job_id: u64 = if let Ok(id) = job.parse::<u64>() {
                    id
                } else {
                    jobs_arr
                        .iter()
                        .find(|j| j["name"].as_str() == Some(&job))
                        .and_then(|j| j["id"].as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Job '{}' not found in pipeline {}", job, pipeline_id))?
                };

                let log = client.get_job_log(job_id).await?;
                println!("{}", log);
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
