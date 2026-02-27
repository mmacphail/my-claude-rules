---
name: malky-api-scaffolder
description: Scaffolds a complete Rust API project — workspace Cargo.toml, Rust crate, infra, justfile, and README — by orchestrating malky-rust-scaffolder and malky-infra-scaffolder. Use when starting a brand-new project from scratch, asked to "create a new api project", "scaffold a full rust project", or "bootstrap a new api".
argument-hint: <project_name> [destination_dir]
allowed-tools: Bash
---

# API Project Scaffolder

Full project bootstrap: delegates to the two sub-scaffolders, then adds the workspace glue, justfile, and README.

## What gets created

```
<project_name>/
  Cargo.toml                    workspace root with [workspace.dependencies]
  justfile                      db-up/down, run, dev, build, test, check, fmt, lint
  README.md                     quick start + structure + API conventions
  .gitignore                    target/, .env, .sqlx/, editor dirs
  .env.example                  pre-filled DATABASE_URL / TEST_ADMIN_URL / RUST_LOG
  apps/
    <project_name>/             Rust crate (from malky-rust-scaffolder)
      Cargo.toml                uses { workspace = true } for all shared deps
      src/
        main.rs  lib.rs  config.rs  state.rs  error.rs  router.rs
        features/example/       model.rs  db.rs  handlers.rs  mod.rs
      migrations/
        20240101000001_create_items.sql
      tests/
        common/mod.rs           TestApp::spawn() with isolated test DB
        items.rs
  infra/                        (from malky-infra-scaffolder)
    docker-compose.yml          dev DB on :6432 (persistent volume)
    docker-compose.test.yml     test DB on :6433 (tmpfs, no logging)
```

## Dependencies

Requires both sibling skills to be installed:
- `~/.claude/skills/malky-rust-scaffolder/scaffold.py`
- `~/.claude/skills/malky-infra-scaffolder/scaffold.py`

The script errors clearly if either is missing.

## How to run

1. Get project name from the user (required). Get destination directory (optional, default `.`).
2. Run the scaffold script:

```bash
python3 ~/.claude/skills/malky-api-scaffolder/scaffold.py <project_name> [destination_dir]
```

3. Show the user the final output and next steps:

```bash
cd <project_name>
cp .env.example .env
just db-up
just run
```

4. Remind them to rename `apps/<project_name>/src/features/example/` to their first real resource and wire it into `mod.rs` + `router.rs`.

## What the workspace patch does

The rust scaffolder creates a standalone `apps/<name>/Cargo.toml` (with explicit versions).
This scaffolder overwrites it with a workspace-aware version (`{ workspace = true }` for all shared deps), which is how the real project is structured. The root `Cargo.toml` holds all shared version pins under `[workspace.dependencies]`.
