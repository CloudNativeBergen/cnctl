# AGENTS.md — cnctl CLI

Rust CLI (`cnctl`) for managing Cloud Native Days Norway conferences. Cargo project inside `cli/` of monorepo.

## RTK (Rust Token Killer)
**Always prefix terminal commands with `rtk`** (e.g., `rtk cargo test`, `rtk git status`). This safely filters output to save context tokens. Use even in command chains (`rtk git add . && rtk git commit -m "m"`).

## Build & Test
```sh
rtk mise run check   # clippy + fmt-check + test (parallel)
rtk mise run clippy  # pedantic lints, warnings as errors
rtk mise run fmt     # format with rustfmt
rtk mise run test    # unit + E2E
```

## Architecture & Conventions
- **Commands**: `src/commands/<domain>/` (keep thin)
- **Types**: `src/types/`
- **Client**: `src/client.rs` (tRPC HTTP client)
- **Display**: `src/display/` (rendering/colors)
- **Template**: `src/template.rs` (`{{{VAR}}}` substitution)
- **Rules**: Clippy pedantic enabled (no warnings). Return `anyhow::Result<T>`, avoid `unwrap()`.
- **Serde**: `#[serde(rename_all = "camelCase")]`, `#[serde(rename = "_id")]`, `#[serde(default)]`. Enum fallbacks: `#[serde(other)] Unknown` + custom Deserialize/Default.
- **Tests**: `#[cfg(test)]` for unit, `tests/e2e.rs` (wiremock) for E2E.

## Commits & Releases
- Commits **must** use Conventional Commits (e.g., `feat(sponsors): add X`, `fix: Y`).
- Release notes include: `feat`, `fix`, `perf`, `refactor`, `docs`.
- Releases/tags are fully automated via GitHub Actions on `main` push. Do **not** tag manually.
