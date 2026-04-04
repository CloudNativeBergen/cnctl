# cnctl

[![CI](https://github.com/CloudNativeBergen/cnctl/actions/workflows/ci.yml/badge.svg)](https://github.com/CloudNativeBergen/cnctl/actions/workflows/ci.yml)
[![Release](https://github.com/CloudNativeBergen/cnctl/actions/workflows/release.yml/badge.svg)](https://github.com/CloudNativeBergen/cnctl/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The organizer CLI for [Cloud Native Days Norway](https://cloudnativedays.no). Review talk proposals, manage the sponsor pipeline, and run your conference — all from the terminal.

## ✨ Features

- 🔐 **Browser-based login** — authenticate with GitHub or LinkedIn OAuth, no API keys to manage
- 📋 **Interactive proposal review** — fuzzy-search, filter by status/format, sort by rating, and scroll through details with vim-style keybindings
- 💰 **Sponsor pipeline** — track sponsors from prospect to paid, with contacts, tiers, and contract status
- 🎨 **Color-coded output** — status badges at a glance (green = confirmed, yellow = submitted, red = rejected, …)
- 📊 **JSON output** — pipe to `jq` or feed into scripts with `--json`
- 🖥️ **Cross-platform** — prebuilt binaries for macOS, Linux, and Windows

## 📦 Installation

### Download a prebuilt binary

Grab the latest build for your platform from [GitHub Releases](https://github.com/CloudNativeBergen/cnctl/releases/latest):

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

<details>
<summary>🔒 Verify your download</summary>

**Checksum:**

```sh
curl -LO https://github.com/CloudNativeBergen/cnctl/releases/latest/download/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

**Build provenance** (requires [GitHub CLI](https://cli.github.com/)):

```sh
gh attestation verify cnctl-aarch64-apple-darwin.tar.gz --repo CloudNativeBergen/cnctl
```

Every release is signed with [Sigstore](https://www.sigstore.dev/) via GitHub Artifact Attestations so you can verify exactly which commit and workflow produced your binary.

</details>

### Build from source

Requires [Rust 1.85+](https://rustup.rs/).

```sh
git clone https://github.com/CloudNativeBergen/cnctl.git
cd cnctl
cargo install --path .
```

## 🚀 Quick start

```sh
cnctl login          # opens browser → pick your conference
cnctl status         # verify session

cnctl admin proposals list                 # interactive fuzzy-search
cnctl admin proposals list --status accepted,confirmed --sort rating
cnctl admin proposals list --json | jq '.[] | .title'

cnctl admin proposals review <id>   # interactive review prompts
cnctl admin proposals review <id> --content 4 --relevance 3 --speaker 5 --comment "Great talk"

cnctl admin sponsors list
cnctl admin sponsors get <id>

cnctl logout         # clear credentials
```

## 📖 Usage

### Authentication

Log in via your browser using GitHub or LinkedIn OAuth:

```sh
cnctl login
```

This opens a browser window, authenticates you, and lets you select which conference to work with. Your session token is stored locally at `~/.config/cnctl/config.toml`.

> You must be registered as a conference organizer to use the admin commands.

Check your current session:

```sh
cnctl status
```

Log out and remove stored credentials:

```sh
cnctl logout
```

### Proposals

**Interactive mode** (default) — launches a fuzzy-search menu where you can type to filter, use arrow keys to navigate, and press enter to view details:

```sh
cnctl admin proposals list
```

**Filter and sort** directly from the command line:

```sh
# Only accepted talks, sorted by review rating (highest first)
cnctl admin proposals list --status accepted --sort rating

# Lightning talks that are still pending review
cnctl admin proposals list --status submitted --format lightning_10

# Sort alphabetically by speaker name, ascending
cnctl admin proposals list --sort speaker --asc
```

**Available filters:**

| Flag | Values |
|------|--------|
| `--status` | `submitted`, `accepted`, `confirmed`, `waitlisted`, `rejected`, `withdrawn`, `draft` |
| `--format` | `lightning_10`, `presentation_20`, `presentation_25`, `presentation_40`, `presentation_45`, `workshop_120`, `workshop_240` |
| `--sort` | `created`, `title`, `speaker`, `rating`, `reviews`, `status` |

**View a single proposal** with full details — speakers, topics, description, outline, and review scores:

```sh
cnctl admin proposals get <proposal-id>
```

In **interactive mode**, selecting a proposal opens a scrollable detail view:

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `Ctrl+U` / `PgUp` | Half-page up |
| `Ctrl+D` / `PgDn` | Half-page down |
| `←` / `h` | Previous proposal |
| `→` / `l` | Next proposal |
| `r` | Start a review |
| `q` / `Esc` | Back to list |

**JSON output** for scripting and automation:

```sh
cnctl admin proposals list --json
cnctl admin proposals get <proposal-id> --json
```

### Reviews

**Interactive review** — shows the proposal, then prompts for scores (1–5) and a comment:

```sh
cnctl admin proposals review <proposal-id>
```

**Non-interactive** — provide all scores and comment as flags:

```sh
cnctl admin proposals review <proposal-id> \
  --content 4 --relevance 3 --speaker 5 \
  --comment "Clear structure, relevant topic, confident speaker"
```

If the proposal has already been reviewed by you, your previous scores and comment are pre-filled as defaults. You can press `Esc` at any prompt to cancel, and a confirmation summary is shown before submitting.

### Sponsors

View the full sponsor pipeline with status, tier, and contract info:

```sh
cnctl admin sponsors list
```

Dive into a specific sponsor for contacts, billing details, and notes:

```sh
cnctl admin sponsors get <sponsor-id>
```

## 🛠️ Development

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
  main.rs         — CLI entry point and argument parsing (clap)
  lib.rs          — public module exports
  auth.rs         — browser-based OAuth flow with local callback server
  client.rs       — tRPC HTTP client
  config.rs       — TOML config read/write (~/.config/cnctl/)
  commands/       — command orchestration
  display/        — terminal output formatting (colors, layout, truncation)
  types/          — API response types (serde)
tests/
  e2e.rs          — end-to-end tests with wiremock
```

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

[MIT](LICENSE) © Hans Kristian Flaatten
