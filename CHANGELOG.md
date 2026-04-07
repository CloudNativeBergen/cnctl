# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `cnctl admin sponsors email <id>` — send templated emails to sponsors
  - Interactive fuzzy-search template picker, pre-sorted by relevance
  - `--template <slug>` for non-interactive template selection
  - `--message` for direct message (skip templates)
  - `--edit` to open `$EDITOR` before sending
  - `--dry-run` to preview without sending
  - `--json` for machine-readable output
  - Automatic variable substitution (`{{{SPONSOR_NAME}}}`, etc.)
- Template variable engine (`template.rs`) with unresolved variable detection
- Browser-based OAuth login (GitHub / LinkedIn) with conference selection
- `cnctl login` / `cnctl logout` / `cnctl status` commands
- `cnctl admin proposals list` — list all talk proposals
- `cnctl admin proposals get <id>` — show proposal details with speakers, topics, and reviews
- `cnctl admin sponsors list` — list sponsor pipeline with status and tier
- `cnctl admin sponsors get <id>` — show sponsor details with contacts, billing, and notes
- Colored terminal output for status fields
- CI workflow (clippy, fmt, test) across Linux, macOS, and Windows
- Release workflow with cross-compilation for 5 targets
