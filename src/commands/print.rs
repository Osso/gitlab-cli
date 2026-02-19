use serde_json::Value;

pub fn print_mrs(value: &Value) {
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

pub fn print_issues(value: &Value) {
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

pub fn print_group_members(value: &Value, show_email: bool) {
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

pub fn print_subgroups(value: &Value) {
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

pub fn print_projects(value: &Value) {
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

pub fn print_mirrors(value: &Value) {
    if let Some(mirrors) = value.as_array() {
        if mirrors.is_empty() {
            println!("No push mirrors configured");
            return;
        }
        for mirror in mirrors {
            print_mirror(mirror);
        }
    }
}

fn print_mirror(mirror: &Value) {
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

    if auth_method == "ssh_public_key" {
        if let Some(ssh_key) = mirror["ssh_public_key"].as_str() {
            println!("       SSH key: {}", ssh_key);
        }
    }
}

pub fn print_webhooks(value: &Value) {
    if let Some(hooks) = value.as_array() {
        if hooks.is_empty() {
            println!("No webhooks configured");
            return;
        }
        for hook in hooks {
            print_webhook(hook);
        }
    }
}

fn print_webhook(hook: &Value) {
    let id = hook["id"].as_u64().unwrap_or(0);
    let url = hook["url"].as_str().unwrap_or("");
    let ssl = hook["enable_ssl_verification"].as_bool().unwrap_or(true);

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

pub fn print_ci_variables(value: &Value) {
    if let Some(vars) = value.as_array() {
        if vars.is_empty() {
            println!("No CI/CD variables found");
            return;
        }
        println!("{:<40} {:<10} {:<8} {}", "KEY", "PROTECTED", "MASKED", "ENVIRONMENT");
        println!("{}", "-".repeat(80));
        for var in vars {
            let key = var["key"].as_str().unwrap_or("");
            let protected = if var["protected"].as_bool().unwrap_or(false) { "yes" } else { "no" };
            let masked = if var["masked"].as_bool().unwrap_or(false) { "yes" } else { "no" };
            let env_scope = var["environment_scope"].as_str().unwrap_or("*");
            println!("{:<40} {:<10} {:<8} {}", key, protected, masked, env_scope);
        }
    }
}

pub fn print_protected_branches(value: &Value) {
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
