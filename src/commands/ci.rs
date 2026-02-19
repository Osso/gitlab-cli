use anyhow::{bail, Context, Result};

use crate::cli::{CiCommands, VarsCommands};
use crate::commands::print::{print_ci_variables};
use crate::config::Config;
use crate::get_client;

pub async fn handle(config: &mut Config, command: CiCommands) -> Result<()> {
    match command {
        CiCommands::Status { id, branch, mr, project } => handle_status(config, project.as_deref(), id, branch, mr).await,
        CiCommands::Wait { id, branch, interval, project } => handle_wait(config, project.as_deref(), id, branch, interval).await,
        CiCommands::Logs { job, pipeline, branch, project } => handle_logs(config, project.as_deref(), job, pipeline, branch).await,
        CiCommands::Retry { job, pipeline, branch, project } => handle_retry(config, project.as_deref(), job, pipeline, branch).await,
        CiCommands::Vars { command, project } => handle_vars(config, project.as_deref(), command).await,
    }
}

async fn handle_vars(
    config: &mut Config,
    project: Option<&str>,
    command: Option<VarsCommands>,
) -> Result<()> {
    match command {
        None => handle_vars_list(config, project).await,
        Some(VarsCommands::Get { key, project: var_project }) => {
            let effective_project = var_project.as_deref().or(project);
            handle_vars_get(config, effective_project, &key).await
        }
    }
}

async fn handle_vars_list(config: &mut Config, project: Option<&str>) -> Result<()> {
    let client = get_client(config, project).await?;
    let vars = client.list_ci_variables().await?;
    print_ci_variables(&vars);
    Ok(())
}

async fn handle_vars_get(config: &mut Config, project: Option<&str>, key: &str) -> Result<()> {
    let client = get_client(config, project).await?;
    let var = client.get_ci_variable(key).await?;
    let value = var["value"].as_str().unwrap_or("");
    print!("{}", value);
    Ok(())
}

async fn handle_status(
    config: &mut Config,
    project: Option<&str>,
    id: Option<u64>,
    branch: Option<String>,
    mr: Option<u64>,
) -> Result<()> {
    let client = get_client(config, project).await?;
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
        let ref_name = detect_branch(branch)?;
        find_latest_pipeline(&client, &ref_name).await?
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
    Ok(())
}

async fn handle_wait(
    config: &mut Config,
    project: Option<&str>,
    id: Option<u64>,
    branch: Option<String>,
    interval: u64,
) -> Result<()> {
    let client = get_client(config, project).await?;

    let ref_name = if id.is_none() {
        Some(detect_branch(branch)?)
    } else {
        branch
    };

    loop {
        let pipeline = if let Some(pid) = id {
            client.get_pipeline(pid).await?
        } else {
            let pipelines = client
                .list_pipelines_for_branch(ref_name.as_deref(), 1)
                .await?;
            let arr = pipelines.as_array().ok_or_else(|| {
                anyhow::anyhow!(
                    "No pipelines found for branch {}",
                    ref_name.as_deref().unwrap_or("?")
                )
            })?;
            if arr.is_empty() {
                bail!(
                    "No pipelines found for branch {}",
                    ref_name.as_deref().unwrap_or("?")
                );
            }
            arr[0].clone()
        };

        let status = pipeline["status"].as_str().unwrap_or("unknown");
        let pipeline_ref = pipeline["ref"].as_str().unwrap_or("");
        let pipeline_id = pipeline["id"].as_u64().unwrap();

        eprintln!("Pipeline #{} - {} ({})", pipeline_id, status, pipeline_ref);

        match status {
            "success" => {
                println!("Pipeline #{} succeeded", pipeline_id);
                break;
            }
            "failed" | "canceled" | "skipped" => {
                bail!("Pipeline #{} {}", pipeline_id, status);
            }
            "running" | "pending" | "created" | "waiting_for_resource" | "preparing"
            | "scheduled" => {
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            }
            _ => {
                bail!("Unknown pipeline status: {}", status);
            }
        }
    }
    Ok(())
}

async fn handle_logs(
    config: &mut Config,
    project: Option<&str>,
    job: String,
    pipeline: Option<u64>,
    branch: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;

    let pipeline_id = if let Some(pid) = pipeline {
        pid
    } else {
        let ref_name = detect_branch(branch)?;
        find_latest_pipeline_id(&client, &ref_name).await?
    };

    let job_id = resolve_job_id(&client, &job, pipeline_id).await?;
    let log = client.get_job_log(job_id).await?;
    println!("{}", log);
    Ok(())
}

async fn handle_retry(
    config: &mut Config,
    project: Option<&str>,
    job: String,
    retry_pipeline: bool,
    branch: Option<String>,
) -> Result<()> {
    let client = get_client(config, project).await?;

    if retry_pipeline {
        let pipeline_id: u64 = job.parse().context("Pipeline ID must be numeric")?;
        let result = client.retry_pipeline(pipeline_id).await?;
        let new_pipeline_id = result["id"].as_u64().unwrap_or(pipeline_id);
        let web_url = result["web_url"].as_str().unwrap_or("");
        println!("Pipeline #{} retried", new_pipeline_id);
        if !web_url.is_empty() {
            println!("{}", web_url);
        }
    } else {
        let job_id = resolve_job_id_from_branch(&client, &job, branch).await?;
        let result = client.retry_job(job_id).await?;
        let job_name = result["name"].as_str().unwrap_or("unknown");
        let new_job_id = result["id"].as_u64().unwrap_or(job_id);
        let web_url = result["web_url"].as_str().unwrap_or("");
        println!("Job '{}' (#{}) retried", job_name, new_job_id);
        if !web_url.is_empty() {
            println!("{}", web_url);
        }
    }
    Ok(())
}

fn detect_branch(branch: Option<String>) -> Result<String> {
    if let Some(b) = branch {
        return Ok(b);
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

async fn find_latest_pipeline(
    client: &crate::api::Client,
    ref_name: &str,
) -> Result<serde_json::Value> {
    let pipelines = client.list_pipelines_for_branch(Some(ref_name), 1).await?;
    let arr = pipelines
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No pipelines found for branch {}", ref_name))?;
    if arr.is_empty() {
        bail!("No pipelines found for branch {}", ref_name);
    }
    Ok(arr[0].clone())
}

async fn find_latest_pipeline_id(
    client: &crate::api::Client,
    ref_name: &str,
) -> Result<u64> {
    let pipeline = find_latest_pipeline(client, ref_name).await?;
    pipeline["id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Invalid pipeline ID"))
}

async fn resolve_job_id(
    client: &crate::api::Client,
    job: &str,
    pipeline_id: u64,
) -> Result<u64> {
    if let Ok(id) = job.parse::<u64>() {
        return Ok(id);
    }
    let jobs = client.list_pipeline_jobs(pipeline_id).await?;
    let jobs_arr = jobs
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No jobs found"))?;
    jobs_arr
        .iter()
        .find(|j| j["name"].as_str() == Some(job))
        .and_then(|j| j["id"].as_u64())
        .ok_or_else(|| anyhow::anyhow!("Job '{}' not found in pipeline {}", job, pipeline_id))
}

async fn resolve_job_id_from_branch(
    client: &crate::api::Client,
    job: &str,
    branch: Option<String>,
) -> Result<u64> {
    if let Ok(id) = job.parse::<u64>() {
        return Ok(id);
    }
    let ref_name = detect_branch(branch)?;
    let pipeline_id = find_latest_pipeline_id(client, &ref_name).await?;
    resolve_job_id(client, job, pipeline_id).await
}
