use anyhow::Result;

use crate::cli::BranchCommands;
use crate::commands::print::print_protected_branches;
use crate::config::Config;
use crate::get_client;

pub async fn handle(config: &mut Config, command: BranchCommands) -> Result<()> {
    match command {
        BranchCommands::List { project } => handle_list(config, project.as_deref()).await,
        BranchCommands::Protect { branch, allow_force_push, project } => {
            handle_protect(config, project.as_deref(), &branch, allow_force_push).await
        }
        BranchCommands::Unprotect { branch, project } => {
            handle_unprotect(config, project.as_deref(), &branch).await
        }
    }
}

async fn handle_list(config: &mut Config, project: Option<&str>) -> Result<()> {
    let client = get_client(config, project).await?;
    let result = client.list_protected_branches().await?;
    print_protected_branches(&result);
    Ok(())
}

async fn handle_protect(
    config: &mut Config,
    project: Option<&str>,
    branch: &str,
    allow_force_push: bool,
) -> Result<()> {
    let client = get_client(config, project).await?;
    client.protect_branch(branch, allow_force_push).await?;
    println!("Protected branch: {}", branch);
    Ok(())
}

async fn handle_unprotect(config: &mut Config, project: Option<&str>, branch: &str) -> Result<()> {
    let client = get_client(config, project).await?;
    client.unprotect_branch(branch).await?;
    println!("Unprotected branch: {}", branch);
    Ok(())
}
