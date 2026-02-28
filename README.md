# my-claude-rules

A collection of Claude Code skills for scaffolding production-ready Rust API projects. These skills automate project setup with consistent patterns and best practices, so you can go from zero to a running API in seconds.

## Skills

### malky-api-scaffolder

The **master orchestrator**. Creates a complete Cargo workspace by calling the Rust and infra scaffolders, then wiring everything together with a root `Cargo.toml`, `justfile`, `.gitignore`, and `README.md`.

**Generated project structure:**

```
<project>/
├── Cargo.toml              # Workspace root with shared dependency versions
├── justfile                # Build / test / dev recipes
├── .env.example
├── apps/
│   └── api/                # Rust crate (Axum + SQLx)
│       ├── src/
│       │   ├── main.rs
│       │   ├── config.rs
│       │   ├── router.rs
│       │   ├── error.rs
│       │   ├── state.rs
│       │   └── features/
│       │       └── example/
│       ├── migrations/
│       └── tests/
└── infra/
    ├── docker-compose.yml       # Dev Postgres on :6432
    └── docker-compose.test.yml  # Test Postgres on :6433 (tmpfs)
```

### malky-rust-scaffolder

Generates a standalone Rust/Axum/SQLx REST API crate with:

- **Axum 0.8** HTTP framework with CORS and tracing middleware
- **SQLx 0.8** with compile-time query verification against PostgreSQL 16
- **Soft deletes** (`deleted_at` column) on all tables
- **Standardized error responses** — `{ "error": { "code": "...", "message": "..." } }`
- **Paginated list responses** — `{ "data": [...], "meta": { "total", "page", "per_page" } }`
- **Feature-module organization** — `src/features/<resource>/` with `model.rs`, `db.rs`, `handlers.rs`
- **Integration tests** with per-test isolated databases

### malky-infra-scaffolder

Creates Docker Compose files and a `.env.example` for local development:

| Service | Port | Storage |
|---------|------|---------|
| Dev Postgres | 6432 | Persistent named volume |
| Test Postgres | 6433 | tmpfs (ephemeral) |

### malky-domain-designer

Guides an interactive DDD design session and produces a `domain-model.md` file at the project root. Covers aggregates, fields, JSONB schemas, statuses, state transitions, capabilities, and invariants.

Use before `malky-domain-implementer` — this is the "design" half of the DDD workflow.

### malky-domain-implementer

Reads a signed-off `domain-model.md` and generates the full implementation for every aggregate:

- **DB migrations** — typed columns, JSONB, status `CHECK` constraints, triggers, indexes
- **Rust feature modules** — `domain.rs`, `repository.rs`, `handlers.rs`, `errors.rs`, `mod.rs` per aggregate, wired into the main router
- **`openapi.json`** — OpenAPI 3.1 spec with capability-named routes and JSONB sub-schemas
- **System tests** — lifecycle, happy-path, invariant-violation, and not-found tests per aggregate

Does not redesign the domain — ambiguous decisions are documented with `// NOTE:` comments.

### malky-outbox-pattern

Implements the **Transactional Outbox Pattern** for any API (language-agnostic). Runs an interactive session to gather context, then generates:

- **Outbox table migration** — `{domain}_{aggregate}_outbox` with a comment on every column explaining its CDC role and downstream impact
- **Debezium connector JSON** — fully annotated config covering the PostgreSQL CDC connector, EventRouter SMT, Avro + Schema Registry serialization, replication slot management, snapshot mode, and error handling
- **Avro schemas** — `{Aggregate}Event` (events topic envelope) and `{Aggregate}` (compacted topic state), with FORWARD_TRANSITIVE compatibility guidance
- **Kafka topic naming doc** — naming convention, `kafka-topics.sh` creation commands, partition rationale, Schema Registry subject mapping, and monitoring metrics
- **Integration checklist** — step-by-step guide for wiring the outbox insert into application service code

Follows the [outbox pattern spec](malky-outbox-pattern/SKILL.md) covering: dual-write problem, ACID guarantees, naming conventions, schema evolution, error resilience, and consumer idempotence.

### malky-skill-creator

A meta-skill that documents how to create new Claude Code skills following consistent conventions (naming, structure, SKILL.md format).

## Quick start

After scaffolding a project with `malky-api-scaffolder`:

```bash
cd <project>
cp .env.example .env
just db-up          # Start dev Postgres
just run            # Start the API
```

Other useful recipes:

```
just dev            # Hot-reload with cargo-watch
just test           # Run integration tests (starts test DB automatically)
just fmt            # Format code
just lint           # Clippy with warnings-as-errors
```

## Tech stack

- **Rust** + **Tokio** async runtime
- **Axum 0.8** — HTTP framework
- **SQLx 0.8** — Async Postgres driver with compile-time checked queries
- **PostgreSQL 16** — Database
- **Docker Compose** — Local infrastructure
- **just** — Task runner
