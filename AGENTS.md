# AGENTS.md — cnctl CLI

Instructions for AI coding agents working on the `cnctl` CLI.

## Project

Rust CLI (`cnctl`) for managing Cloud Native Days Norway conferences.
Standalone Cargo project inside `cli/` of the main website monorepo.

## Build & Test

```sh
mise run check          # clippy + fmt-check + test (parallel)
mise run clippy         # pedantic lints, warnings as errors
mise run fmt            # format with rustfmt
mise run test           # all tests (unit + E2E)
cargo test              # alternative without mise
```

## Architecture

- **Commands** live in `src/commands/<domain>/` as multi-file modules (`mod.rs`, `args.rs`, `interactive.rs`, etc.)
- **Types** live in `src/types/` — one file per API domain, re-exported from `mod.rs`
- **Client** (`src/client.rs`) — tRPC HTTP client, all API calls go through here
- **Display** (`src/display/`) — terminal rendering, table formatting, colors
- **Template** (`src/template.rs`) — `{{{VAR}}}` substitution engine
- Keep commands thin: API calls in `mod.rs`, display logic in `display/`, business logic in domain modules

## Coding Conventions

- Clippy pedantic enabled — no warnings allowed
- Serde: `#[serde(rename_all = "camelCase")]`, `#[serde(rename = "_id")]`, `#[serde(default)]`
- Unknown enum variants: `#[serde(other)] Unknown` with custom `Deserialize` + `Default` impls
- Error handling: return `anyhow::Result<T>`, avoid `unwrap()` / `process::exit()`
- Tests: unit tests in `#[cfg(test)]` modules, E2E tests in `tests/e2e.rs` using wiremock

## Commit Messages & Release Notes

Release notes are **auto-generated** from commit messages using [git-cliff](https://git-cliff.org/) (configured in `cliff.toml`).

**All commits must use [Conventional Commits](https://www.conventionalcommits.org/) format:**

- `feat:` / `feat(scope):` — new features (included in release notes)
- `fix:` — bug fixes (included)
- `perf:` — performance improvements (included)
- `refactor:` — code refactoring (included)
- `docs:` — documentation changes (included)
- `chore:`, `style:`, `ci:`, `test:`, `build(deps):` — excluded from release notes

Examples:
```
feat(sponsors): add email functionality with template support
fix: handle missing conference ID in sponsor lookup
refactor: extract shared email builder into helper module
```

## Releases

Releases are fully automated via GitHub Actions:

1. Push to `main` triggers CI (`ci.yml`: clippy, fmt, test on Linux/macOS/Windows)
2. On CI success, release workflow (`release.yml`) cross-compiles for 5 targets
3. git-cliff generates release notes from commits since the last tag
4. A GitHub Release is created with binaries, checksums, and build provenance attestations

Tags use date-based versioning: `YYYY.MM.DD-<shortsha>`. Do **not** create tags manually — the release workflow handles this.
