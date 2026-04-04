# Contributing

Thanks for your interest in contributing to cnctl!

## Getting started

1. Fork and clone the repository
2. Install [Rust 1.85+](https://rustup.rs/)
3. Run the checks: `cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test`

## Development workflow

1. Create a branch from `main`
2. Make your changes
3. Ensure all checks pass: `mise run check` (or run clippy, fmt, and test individually)
4. Submit a pull request

## Code style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Pedantic clippy lints are enforced
- Format with `cargo fmt` (edition 2024)
- Keep commands thin — business logic in domain modules, display logic in `display/`

## Testing

- Unit tests live alongside source code (`#[cfg(test)]` modules)
- Integration tests live in `tests/e2e.rs` using [wiremock](https://crates.io/crates/wiremock)
- All tests must pass before merging

## Commit messages

Use clear, concise commit messages. Reference issues where applicable.

## Releases

Releases are automated via GitHub Actions. Tag a version to trigger a build:

```sh
git tag v0.1.0
git push origin v0.1.0
```

This cross-compiles binaries for Linux, macOS, and Windows.
