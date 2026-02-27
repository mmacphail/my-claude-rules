---
name: malky-infra-scaffolder
description: Scaffolds the infra/ directory and .env.example for a new Rust/PostgreSQL project. Use when setting up a new project's local infrastructure, asked to "scaffold infra", "set up docker compose", "create dev database config", or "set up postgres for a new project".
argument-hint: <project_name> [destination_dir]
allowed-tools: Bash
---

# Infra Scaffolder

Generates the local dev/test PostgreSQL infra and `.env.example` for a new project.

## What gets generated

```
<dest>/
  infra/
    docker-compose.yml        # dev DB on :6432, persistent named volume
    docker-compose.test.yml   # test DB on :6433, tmpfs (ephemeral, no logging)
  .env.example                # DATABASE_URL + DATABASE_TEST_ADMIN_URL + RUST_LOG
```

## Key conventions baked in

- Dev DB: user/password/dbname all = `<project_name>`, port **6432**
- Test DB: user/password = `<project_name>_test`, dbname = `<project_name>_test`, port **6433**
- Test DB uses `tmpfs` (wiped on container stop) and `log_statement=none` to keep test runs quiet
- `DATABASE_TEST_ADMIN_URL` points to the `postgres` system DB so `TestApp::spawn()` can `CREATE DATABASE test_<uuid>` dynamically
- Managed with `podman compose` (drop-in for `docker compose`)

## How to run

1. Get project name from the user (required). Get destination directory (optional, default `.`).
2. Run the scaffold script:

```bash
python3 ~/.claude/skills/malky-infra-scaffolder/scaffold.py <project_name> [destination_dir]
```

3. Show the user the output and remind them:

```bash
# Start dev DB
podman compose -f infra/docker-compose.yml up -d

# Start test DB (before running integration tests)
podman compose -f infra/docker-compose.test.yml up -d

# Copy env
cp .env.example .env
```

## Pairing with the Rust scaffolder

This skill is the infra counterpart to `malky-rust-scaffolder`. Run both when starting a new project:
1. `malky-rust-scaffolder` → creates the Rust crate
2. `malky-infra-scaffolder` → creates the compose files and `.env.example`

The `.env.example` generated here is the correct one to use — it supersedes the one placed by the rust scaffolder (which uses generic placeholder URLs).
