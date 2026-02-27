# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A collection of Claude Code skills for scaffolding production-ready Rust API projects (Axum 0.8 + SQLx 0.8 + PostgreSQL 16). There is no build system, test suite, or linter for the repo itself — it contains Python scaffold scripts and Rust template files.

## Architecture

Skills with an orchestration hierarchy:

```
malky-api-scaffolder (master orchestrator)
├── malky-rust-scaffolder  (Rust/Axum/SQLx API crate)
└── malky-infra-scaffolder (Docker Compose for dev/test Postgres)

malky-domain-designer     (interactive DDD session → domain-model.md)
malky-domain-implementer  (implements domain-model.md → migrations, Rust features, OpenAPI, tests)
malky-skill-creator       (standalone meta-skill for creating new skills)
```

**`malky-api-scaffolder`** is the primary entry point. Its `scaffold.py` calls the two sub-scaffolders, then adds workspace glue: root `Cargo.toml` (with `[workspace.dependencies]`), `justfile`, `README.md`, `.gitignore`. It also patches the Rust crate's `Cargo.toml` to use `{ workspace = true }` for shared deps.

**`malky-rust-scaffolder`** generates a standalone Rust crate with Axum routes, SQLx queries, migrations, and integration tests. Template files live in `malky-rust-scaffolder/resources/`.

**`malky-infra-scaffolder`** generates `infra/docker-compose.yml` (dev DB on port 6432) and `infra/docker-compose.test.yml` (test DB on port 6433, tmpfs). Templates live in `malky-infra-scaffolder/resources/`.

**`malky-domain-implementer`** has no scaffold.py — Claude does the work directly using Write/Edit tools. Given a signed-off `domain-model.md`, it generates DB migrations, Rust feature modules (domain/repository/handlers/errors), an `openapi.json`, and system tests for every aggregate.

**`malky-skill-creator`** has no scaffold.py — it is purely instructional. It describes the conventions for creating new skills.

## Skill structure convention

Each skill follows this layout:

```
<skill-name>/
├── SKILL.md        # YAML frontmatter (name, description, argument-hint, allowed-tools) + instructions
├── scaffold.py     # Python script that copies/patches resources (optional)
└── resources/      # Template files to copy into generated projects (optional)
```

Skills are installed at `~/.claude/skills/<skill-name>/`.

## Invoking the scaffolders

```bash
# Full project (workspace + crate + infra)
python3 ~/.claude/skills/malky-api-scaffolder/scaffold.py <project_name> [destination_dir]

# Rust crate only
python3 ~/.claude/skills/malky-rust-scaffolder/scaffold.py <project_name> [destination_dir]

# Infra only
python3 ~/.claude/skills/malky-infra-scaffolder/scaffold.py <project_name> [destination_dir]
```

## Conventions baked into generated projects

- **Feature modules**: `src/features/<resource>/` with `mod.rs`, `model.rs`, `db.rs`, `handlers.rs`
- **Soft deletes**: all tables have `deleted_at TIMESTAMPTZ`; all queries filter `WHERE deleted_at IS NULL`
- **Error envelope**: `{ "error": { "code": "NOT_FOUND", "message": "..." } }`
- **Pagination**: `{ "data": [...], "meta": { "total", "page", "per_page" } }`
- **Integration tests**: `TestApp::spawn()` creates an isolated `test_<uuid>` database per test
- **justfile recipes**: `db-up`, `run`, `dev`, `test`, `fmt`, `lint`, `build`, `check`
