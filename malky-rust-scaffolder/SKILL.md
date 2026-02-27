---
name: malky-rust-scaffolder
description: Scaffolds a new Rust/Axum/SQLx REST API project from Alexandre's personal template. Use when starting a new Rust API project, when asked to "scaffold a rust api", "create a new rust project", or "start a new axum project".
argument-hint: <project_name> [destination_dir]
allowed-tools: Bash
---

# Rust API Scaffolder

Scaffolds a production-ready Rust API project using Axum 0.8, SQLx 0.8, and PostgreSQL.

## What gets generated

```
<project_name>/
  Cargo.toml                          # standalone (not workspace)
  .env.example
  src/
    main.rs                           # tokio entrypoint, pool, migrations, bind
    lib.rs
    config.rs                         # DATABASE_URL + API_PORT from env
    state.rs                          # AppState { db: PgPool }
    error.rs                          # AppError enum → JSON { error: { code, message } }
    router.rs                         # create_router(), /health, /api/v1/*
    features/
      mod.rs
      example/                        # rename this for your first resource
        mod.rs                        # router() fn + route declarations
        model.rs                      # structs, PaginationParams, ListResponse<T>
        db.rs                         # sqlx queries (find_all, find_by_id, insert, update, soft_delete)
        handlers.rs                   # Axum handlers (list, get, create, update, delete)
  migrations/
    20240101000001_create_items.sql   # example table with soft-delete pattern
  tests/
    common/mod.rs                     # TestApp::spawn() → fresh test_<uuid> DB per test
    items.rs                          # example integration tests
```

## Key conventions baked in

- All tables have `deleted_at TIMESTAMPTZ` — all queries filter `WHERE deleted_at IS NULL`
- Error envelope: `{ "error": { "code": "NOT_FOUND", "message": "..." } }`
- `sqlx::Error::RowNotFound` → 404; PG code `23505` → 409
- `runtime-tokio-rustls` TLS feature for sqlx (not the old `rustls`)
- List endpoints return `{ data: [...], meta: { total, page, per_page } }`
- Integration tests create isolated `test_<uuid>` databases and drop them on cleanup

## How to run

1. Get project name from the user (required). Get destination directory (optional, default `.`).
2. Run the scaffold script:

```bash
python3 ~/.claude/skills/malky-rust-scaffolder/scaffold.py <project_name> [destination_dir]
```

3. Show the user the output and next steps:
   - `cd <project_name>`
   - `cp .env.example .env` and fill in `DATABASE_URL`
   - `cargo build`
   - Rename `src/features/example/` to their first resource name
   - Update `src/features/mod.rs` and `src/router.rs` to use the new module name

## Adding a new feature after scaffolding

Tell the user the standard pattern:
1. Copy `src/features/example/` → `src/features/<name>/`
2. Rename `Item` → their model, update table name in `db.rs`
3. Add `pub mod <name>;` to `src/features/mod.rs`
4. Add `.merge(crate::features::<name>::router())` in `src/router.rs`
5. Add migration in `migrations/`
