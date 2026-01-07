# gitlab-cli

GitLab CLI tool for merge requests, issues, and CI/CD operations.

## Build

Default target is `x86_64-unknown-linux-musl` (statically linked). This is set in `.cargo/config.toml`.

```bash
cargo build --release
```

Requires musl target installed:
```bash
rustup target add x86_64-unknown-linux-musl
```
