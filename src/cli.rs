use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitlab")]
#[command(about = "GitLab CLI for read-only operations")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Fetch a raw file from a repository
    File {
        /// File path in the repository (e.g., src/main.rs)
        path: String,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
        /// Git ref (branch, tag, or commit SHA; defaults to project default branch)
        #[arg(long, name = "ref")]
        git_ref: Option<String>,
    },
    /// Make a raw GitLab API call
    Api {
        /// API endpoint (e.g., /projects or /api/v4/projects)
        endpoint: String,
        /// HTTP method
        #[arg(long, short, default_value = "GET")]
        method: String,
        /// JSON request body
        #[arg(long, short)]
        data: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AuthCommands {
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
pub enum MrCommands {
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
    /// Merge a merge request immediately
    Merge {
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
    /// Resolve or unresolve a discussion thread on a merge request
    Resolve {
        /// Merge request IID
        iid: u64,
        /// Discussion ID to resolve
        #[arg(long, short)]
        discussion: String,
        /// Unresolve instead of resolve
        #[arg(long, short)]
        unresolve: bool,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CiCommands {
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
        /// Pipeline ID (defaults to latest for branch)
        #[arg(long)]
        pipeline: Option<u64>,
        /// Branch name (defaults to current git branch)
        #[arg(long, short)]
        branch: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Retry a failed job or pipeline
    Retry {
        /// Job name or ID (or pipeline ID with --pipeline)
        job: String,
        /// Retry entire pipeline instead of a single job
        #[arg(long)]
        pipeline: bool,
        /// Branch name (defaults to current git branch)
        #[arg(long, short)]
        branch: Option<String>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
    /// Manage CI/CD variables
    Vars {
        #[command(subcommand)]
        command: Option<VarsCommands>,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum VarsCommands {
    /// Get the value of a single CI/CD variable
    Get {
        /// Variable key name
        key: String,
        /// Override default project
        #[arg(long, short)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GroupCommands {
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
pub enum ProjectCommands {
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
    /// Update project settings
    Update {
        /// Project path (e.g., group/project)
        project: String,
        /// Repository access level (enabled, private, disabled)
        #[arg(long)]
        repository_access_level: Option<String>,
        /// Issues access level (enabled, private, disabled)
        #[arg(long)]
        issues_access_level: Option<String>,
        /// Merge requests access level (enabled, private, disabled)
        #[arg(long)]
        merge_requests_access_level: Option<String>,
        /// CI/CD builds access level (enabled, private, disabled)
        #[arg(long)]
        builds_access_level: Option<String>,
        /// Wiki access level (enabled, private, disabled)
        #[arg(long)]
        wiki_access_level: Option<String>,
        /// Snippets access level (enabled, private, disabled)
        #[arg(long)]
        snippets_access_level: Option<String>,
        /// Pages access level (enabled, private, disabled)
        #[arg(long)]
        pages_access_level: Option<String>,
        /// Archive or unarchive the project (true/false)
        #[arg(long)]
        archived: Option<bool>,
        /// Project description
        #[arg(long)]
        description: Option<String>,
        /// Default branch
        #[arg(long)]
        default_branch: Option<String>,
        /// Project visibility (private, internal, public)
        #[arg(long)]
        visibility: Option<String>,
    },
    /// Manage push mirrors
    Mirrors {
        #[command(subcommand)]
        command: MirrorCommands,
    },
}

#[derive(Subcommand)]
pub enum MirrorCommands {
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
pub enum BranchCommands {
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
pub enum WebhookCommands {
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
pub enum IssueCommands {
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
