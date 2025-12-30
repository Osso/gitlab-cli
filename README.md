# gitlab-cli

[![CI](https://github.com/Osso/gitlab-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Osso/gitlab-cli/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/Osso/gitlab-cli)](https://github.com/Osso/gitlab-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

CLI for read-only GitLab operations (merge requests, CI/CD pipelines).

## Installation

```bash
cargo install --path .
```

## Setup

```bash
gitlab config
```

## Usage

```bash
gitlab mr list                             # List merge requests
gitlab mr list -s merged -a Osso -n 10     # List merged MRs by author
gitlab mr list --created-after 2025-12-01  # Filter by date
gitlab mr show <id>                        # Show MR details (JSON)
gitlab ci status                           # Show latest pipeline status
gitlab ci logs <job>                       # Show job logs
```

## License

MIT
