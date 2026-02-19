use anyhow::Result;

use crate::cli::GroupCommands;
use crate::commands::print::{print_group_members, print_subgroups};
use crate::config::Config;
use crate::get_group_client;

pub async fn handle(config: &mut Config, command: GroupCommands) -> Result<()> {
    match command {
        GroupCommands::Members { group, per_page, email } => handle_members(config, &group, per_page, email).await,
        GroupCommands::Subgroups { group, per_page } => handle_subgroups(config, &group, per_page).await,
        GroupCommands::Show { group } => handle_show(config, &group).await,
    }
}

async fn handle_members(
    config: &mut Config,
    group: &str,
    per_page: u32,
    email: bool,
) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.list_group_members(group, per_page, email).await?;
    print_group_members(&result, email);
    Ok(())
}

async fn handle_subgroups(config: &mut Config, group: &str, per_page: u32) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.list_group_subgroups(group, per_page).await?;
    print_subgroups(&result);
    Ok(())
}

async fn handle_show(config: &mut Config, group: &str) -> Result<()> {
    let client = get_group_client(config).await?;
    let result = client.get_group(group).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
