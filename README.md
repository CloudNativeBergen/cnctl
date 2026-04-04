# cnctl

CLI for [Cloud Native Days Norway](https://cloudnativedays.no) conference organizers.

Manage talk proposals and sponsor pipelines from the terminal.

## Installation

### From releases

Download the latest binary for your platform from [GitHub Releases](https://github.com/CloudNativeBergen/cnctl/releases):

```sh
# macOS (Apple Silicon)
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/cnctl-aarch64-apple-darwin.tar.gz
tar xzf cnctl-aarch64-apple-darwin.tar.gz
sudo mv cnctl /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/cnctl-x86_64-apple-darwin.tar.gz
tar xzf cnctl-x86_64-apple-darwin.tar.gz
sudo mv cnctl /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/cnctl-x86_64-unknown-linux-gnu.tar.gz
tar xzf cnctl-x86_64-unknown-linux-gnu.tar.gz
sudo mv cnctl /usr/local/bin/

# Linux (arm64)
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/cnctl-aarch64-unknown-linux-gnu.tar.gz
tar xzf cnctl-aarch64-unknown-linux-gnu.tar.gz
sudo mv cnctl /usr/local/bin/
```

Verify the checksum:

```sh
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

Verify build provenance (requires [GitHub CLI](https://cli.github.com/)):

```sh
gh attestation verify cnctl-aarch64-apple-darwin.tar.gz --repo CloudNativeBergen/cnctl
```

### From source

Requires [Rust 1.85+](https://rustup.rs/).

```sh
cargo install --path .
```

## Usage

### Authentication

Log in via your browser using GitHub or LinkedIn OAuth:

```sh
cnctl login
```

This opens a browser window, authenticates you, and stores a session token locally at `~/.config/cnctl/config.toml`. You must be registered as a conference organizer.

Check your current session:

```sh
cnctl status
```

Log out (removes stored credentials):

```sh
cnctl logout
```

### Proposals

List all talk proposals for the current conference:

```sh
cnctl admin proposals list
```

View details for a specific proposal:

```sh
cnctl admin proposals get <proposal-id>
```

### Sponsors

List the sponsor pipeline:

```sh
cnctl admin sponsors list
```

View sponsor details:

```sh
cnctl admin sponsors get <sponsor-id>
```

## Development

### Prerequisites

- [Rust 1.85+](https://rustup.rs/)
- [mise](https://mise.jdx.dev/) (optional, for task runner)

### Build and test

```sh
# Using mise (recommended)
mise run check    # clippy + fmt-check + test (parallel)
mise run build    # release build

# Using cargo directly
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
cargo test
```

### Project structure

```text
src/
  lib.rs          — public module exports
  main.rs         — CLI argument parsing (clap)
  auth.rs         — browser-based OAuth login flow
  client.rs       — tRPC HTTP client
  config.rs       — config file read/write (~/.config/cnctl/)
  display/        — terminal output formatting
    proposal.rs   — proposal list/detail rendering
    sponsor.rs    — sponsor list/detail rendering
  types/          — API response types (serde)
    proposal.rs   — proposal domain types
    sponsor.rs    — sponsor domain types
  commands/       — command orchestration
    login.rs      — login flow
    logout.rs     — credential cleanup
    status.rs     — session info
    proposals.rs  — proposal fetch + display
    sponsors.rs   — sponsor fetch + display
tests/
  e2e.rs          — end-to-end integration tests (wiremock)
```

## License

[MIT](LICENSE)
