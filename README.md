# gitlab-cli

[![CI](https://github.com/Osso/gitlab-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Osso/gitlab-cli/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/Osso/gitlab-cli)](https://github.com/Osso/gitlab-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Lightweight Rust CLI for GitLab operations. Preferred over `glab` for read operations, auto-merge, and date filtering.

## Installation

```bash
cargo install --git https://github.com/Osso/gitlab-cli
```

Or build from source:

```bash
git clone https://github.com/Osso/gitlab-cli
cd gitlab-cli
cargo build --release
```

## Setup

Authenticate with OAuth2 (recommended):

```bash
gitlab auth login
```

Or configure manually:

```bash
gitlab config --host https://gitlab.com --project group/project
```

## Usage

### Merge Requests

```bash
gitlab mr list                             # List open MRs
gitlab mr list -s merged -a username       # List merged MRs by author
gitlab mr list --created-after 2025-01-01  # Filter by date
gitlab mr show <iid>                       # Show MR details (JSON)
gitlab mr automerge <iid>                  # Auto-merge when pipeline passes
gitlab mr automerge <iid> --keep-branch    # Auto-merge, keep source branch
```

### Issues

```bash
gitlab issue list                          # List open issues
gitlab issue list -s closed                # List closed issues
gitlab issue list --assignee username      # Filter by assignee
gitlab issue list --labels bug,urgent      # Filter by labels
gitlab issue list --search "keyword"       # Search in title/description
gitlab issue show <iid>                    # Show issue details (JSON)
gitlab issue create -t "Title" -d "Desc"   # Create new issue
gitlab issue create -t "Title" -a user     # Create and assign
```

### CI/CD

```bash
gitlab ci status                           # Show latest pipeline status
gitlab ci logs <job_name>                  # Show job logs
gitlab ci logs <job_name> --pipeline 123   # Logs from specific pipeline
```

### Authentication

```bash
gitlab auth login                          # OAuth2 login (opens browser)
gitlab auth status                         # Show auth status
```

## When to Use gitlab vs glab

| Operation | Recommended |
|-----------|-------------|
| List MRs/issues | `gitlab` (date filtering) |
| Show MR/issue details | `gitlab` (JSON output) |
| Auto-merge | `gitlab` (more reliable) |
| Create MR | `glab` |
| Update MR | `glab` |
| Create issue | `gitlab` or `glab` |
| CI trace (real-time) | `glab` |
| API calls | `glab api` |

## License

MIT
