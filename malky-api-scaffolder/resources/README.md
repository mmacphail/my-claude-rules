# __APP_NAME__

A REST API built with Rust, Axum 0.8, SQLx 0.8, and PostgreSQL 16.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Podman](https://podman.io/) (or Docker)
- [just](https://just.systems/) — `cargo install just`
- Optional: [cargo-watch](https://crates.io/crates/cargo-watch) for hot-reload — `cargo install cargo-watch`

## Quick start

```bash
# 1. Start the dev database
just db-up

# 2. Configure environment
cp .env.example .env

# 3. Run
just run
```

The API listens on `http://localhost:3001`.
Health check: `curl http://localhost:3001/health`

## Commands

| Recipe | Description |
|---|---|
| `just db-up` | Start dev Postgres on `:6432` |
| `just db-down` | Stop dev Postgres |
| `just test-db-up` | Start test Postgres on `:6433` (ephemeral) |
| `just test-db-down` | Stop test Postgres |
| `just run` | Run the API |
| `just dev` | Run with hot-reload (requires cargo-watch) |
| `just build` | Build the workspace |
| `just test` | Run all integration tests |
| `just check` | Fast compile check |
| `just fmt` | Format code |
| `just lint` | Clippy (warnings as errors) |

## Project structure

```
apps/api/src/
  main.rs          entrypoint — pool, migrations, bind :3001
  lib.rs
  config.rs        Config from DATABASE_URL / API_PORT env vars
  state.rs         AppState { db: PgPool }
  error.rs         AppError enum → { "error": { "code", "message" } }
  router.rs        create_router() — /health + /api/v1/*
  features/
    example/       rename this for your first resource
      mod.rs       router() fn
      model.rs     structs, PaginationParams, ListResponse<T>
      db.rs        sqlx queries (soft-delete pattern)
      handlers.rs  Axum handlers (list, get, create, update, delete)
infra/
  docker-compose.yml       dev DB (persistent volume)
  docker-compose.test.yml  test DB (tmpfs, wiped on stop)
migrations/
  *.sql            run automatically on startup via sqlx::migrate!
tests/
  common/mod.rs    TestApp::spawn() — fresh test_<uuid> DB per test
  *.rs             integration tests
```

## API conventions

- Base path: `/api/v1`
- Error envelope: `{ "error": { "code": "NOT_FOUND", "message": "..." } }`
- List envelope: `{ "data": [...], "meta": { "total", "page", "per_page" } }`
- Soft deletes: all tables have `deleted_at TIMESTAMPTZ`, all queries filter `WHERE deleted_at IS NULL`

## Adding a new resource

1. Copy `apps/api/src/features/example/` → `apps/api/src/features/<name>/`
2. Rename `Item` → your model, update table name in `db.rs`
3. Add `pub mod <name>;` in `src/features/mod.rs`
4. Merge the router in `src/router.rs`
5. Add a migration in `migrations/`
