mod api;
mod auth;
pub mod cli;
mod commands;
mod config;

use anyhow::Result;

use cli::{Cli, Commands};
use clap::Parser;
use config::Config;

pub async fn get_client(config: &mut Config, project_override: Option<&str>) -> Result<api::Client> {
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

    api::Client::new(config.host(), token, &project)
}

pub async fn get_group_client(config: &mut Config) -> Result<api::Client> {
    if let Some(oauth2) = &config.oauth2 {
        if oauth2.is_expired() {
            eprintln!("Token expired, refreshing...");
            auth::refresh_token(config).await?;
        }
    }

    let token = config.get_access_token().ok_or_else(|| {
        anyhow::anyhow!("No token configured. Run: gitlab auth login --client-id <id>")
    })?;

    api::Client::new(config.host(), token, "_")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Config { host, token, project } => handle_config(&mut config, host, token, project),
        Commands::Auth { command } => handle_auth(&mut config, command).await,
        Commands::Mr { command } => commands::mr::handle(&mut config, command).await,
        Commands::Issue { command } => commands::issue::handle(&mut config, command).await,
        Commands::Ci { command } => commands::ci::handle(&mut config, command).await,
        Commands::Group { command } => commands::group::handle(&mut config, command).await,
        Commands::Project { command } => commands::project::handle(&mut config, command).await,
        Commands::Webhook { command } => commands::webhook::handle(&mut config, command).await,
        Commands::Branch { command } => commands::branch::handle(&mut config, command).await,
        Commands::File { path, project, git_ref } => handle_file(&mut config, path, project, git_ref).await,
        Commands::Api { endpoint, method, data } => handle_api(&mut config, endpoint, method, data).await,
    }
}

fn handle_config(
    config: &mut Config,
    host: Option<String>,
    token: Option<String>,
    project: Option<String>,
) -> Result<()> {
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
    Ok(())
}

async fn handle_auth(config: &mut Config, command: cli::AuthCommands) -> Result<()> {
    match command {
        cli::AuthCommands::Login { client_id, host } => {
            handle_auth_login(config, client_id, host).await
        }
        cli::AuthCommands::Status => {
            print_auth_status(config);
            Ok(())
        }
    }
}

async fn handle_auth_login(
    config: &mut Config,
    client_id: Option<String>,
    host: Option<String>,
) -> Result<()> {
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
    config.token = None;
    if host.is_some() {
        config.host = host;
    }
    config.save()?;
    println!("Authentication successful!");
    Ok(())
}

fn print_auth_status(config: &Config) {
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

async fn handle_file(
    config: &mut Config,
    path: String,
    project: Option<String>,
    git_ref: Option<String>,
) -> Result<()> {
    let client = get_client(config, project.as_deref()).await?;
    let ref_name = match git_ref {
        Some(r) => r,
        None => {
            let project_info = client.get_project().await?;
            project_info["default_branch"]
                .as_str()
                .unwrap_or("master")
                .to_string()
        }
    };
    let content = client.get_raw_file(&path, &ref_name).await?;
    print!("{}", content);
    Ok(())
}

async fn handle_api(
    config: &mut Config,
    endpoint: String,
    method: String,
    data: Option<String>,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let body = client
        .raw_request(&method, &endpoint, data.as_deref())
        .await?;
    println!("{}", body);
    Ok(())
}
