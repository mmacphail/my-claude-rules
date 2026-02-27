# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Repo Is

A suite of four Claude Code skills that scaffold production-ready Rust/Axum/SQLx API projects. There is no buildable application here — this repo contains skill definitions (SKILL.md), Python scaffolding scripts (scaffold.py), and template resource files.

## Repository Structure

Each top-level directory is an independent Claude Code skill:

- **malky-api-scaffolder** — Master orchestrator. Calls the rust and infra scaffolders, then assembles a Cargo workspace with justfile, README, and .env.example. This is the primary entry point for creating new projects.
- **malky-rust-scaffolder** — Generates a standalone Rust/Axum/SQLx crate with feature-module organization, soft deletes, error envelopes, pagination, and per-test isolated databases.
- **malky-infra-scaffolder** — Generates Docker Compose files for dev Postgres (:6432, persistent volume) and test Postgres (:6433, tmpfs ephemeral).
- **create-malky-skill** — Meta-skill documenting how to create new skills following the conventions in this repo.

## How the Skills Relate

`malky-api-scaffolder` is the orchestrator — its `scaffold.py` calls `malky-rust-scaffolder/scaffold.py` and `malky-infra-scaffolder/scaffold.py` sequentially, then patches the output into a unified Cargo workspace. The rust and infra scaffolders can also be used independently.

All three scaffolders follow the same pattern:
1. Recursively copy files from their `resources/` directory to the target
2. Replace the `__APP_NAME__` placeholder with the actual project name
3. Print next-step instructions

The orchestrator additionally renames `apps/<name>/` to `apps/api/` and patches `Cargo.toml` to use `{ workspace = true }` dependencies.

## Working with Scaffolding Scripts

The scaffold scripts are plain Python 3 with no external dependencies. To test them:

```bash
python3 malky-rust-scaffolder/scaffold.py test_project /tmp
python3 malky-infra-scaffolder/scaffold.py test_project /tmp
python3 malky-api-scaffolder/scaffold.py test_project /tmp
```

The api scaffolder expects the rust and infra scaffolders to be at `~/.claude/skills/malky-rust-scaffolder/` and `~/.claude/skills/malky-infra-scaffolder/` respectively.

## Skill Conventions

Each skill directory contains:
- `SKILL.md` — Skill metadata and instructions (read by Claude Code at invocation time)
- `scaffold.py` — The scaffolding script (except `create-malky-skill` which has no script)
- `resources/` — Template files copied to the target project

Skill naming: lowercase, hyphens, max 64 chars, gerund form preferred for the description.

## Conventions Enforced in Generated Projects

These patterns are baked into the templates and should be maintained when modifying resource files:

- **Soft deletes**: Every table has `deleted_at TIMESTAMPTZ`; all queries filter `WHERE deleted_at IS NULL`
- **Error envelope**: `{ "error": { "code": "NOT_FOUND", "message": "..." } }`
- **List response**: `{ "data": [...], "meta": { "total", "page", "per_page" } }`
- **Feature modules**: `src/features/<resource>/` containing `model.rs`, `db.rs`, `handlers.rs`, `mod.rs`
- **API base path**: `/api/v1`
- **Test isolation**: Each integration test gets its own `test_<uuid>` database, created and dropped automatically
- **Ports**: Dev Postgres 6432, Test Postgres 6433, API default 3001
