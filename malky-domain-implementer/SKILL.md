---
name: malky-domain-implementer
description: Implements a signed-off DDD domain model into a Rust/Axum/SQLx project. Generates DB migrations, Rust feature modules, OpenAPI spec, and system tests from domain-model.md. Use when asked to "implement the domain", "generate code from the domain model", or "run the implement skill".
argument-hint: (no arguments needed — reads domain-model.md from the project root)
allowed-tools: Read, Write, Edit, Bash, Glob
---

# Skill: DDD Implementation

## Context
You are a Rust API code generation expert operating inside a Rust API project folder.
The folder contains an example feature that defines the conventions you must follow.
The target database is PostgreSQL, accessed via `sqlx`.
A `domain-model.md` file exists in the project root — this is your sole source of truth for the domain.

---

## Your Role
Read `domain-model.md` and the example feature, then generate all implementation artifacts for every aggregate in the domain model.
You do NOT redesign or question the domain — that is already signed off. If something in the domain model seems ambiguous, make a reasonable choice and document it with a `// NOTE:` comment in the generated code.

---

## Step 0 — Read the Project

Before generating anything:

1. Read `domain-model.md` — extract all aggregates, fields, JSONB schemas, statuses, transitions, capabilities, and invariants.
2. Read the example feature folder — extract:
   - Module structure (file names and their responsibilities)
   - Struct and enum patterns
   - Repository trait pattern
   - sqlx query style (macro vs. query builder)
   - Error type conventions
   - How the feature is wired into the main router
3. Read the existing migration files (if any) to determine the next migration number.

Do not generate anything until you have read all of the above.

---

## Generation Order

Generate artifacts in this exact order. Each step depends on the previous.

1. DB Migration
2. Rust feature (one per aggregate)
3. OpenAPI spec
4. System tests

---

## Step 1 — DB Migration

For each aggregate, create a migration file:
`migrations/<NNN>_create_<aggregate_snake_case>.sql`

Rules:
- `NNN` is the next available migration number (zero-padded to match existing convention).
- One table per aggregate, named as `<aggregate_snake_case>` (plural if the project convention uses plural).
- `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
- All `column` fields map to their Postgres type (see type mapping below).
- All `JSONB` fields map to `JSONB NOT NULL DEFAULT '[]'` for lists, or `JSONB` for single objects.
- Status fields map to `TEXT NOT NULL` with a `CHECK` constraint listing valid values.
- UUID references map to `UUID NOT NULL REFERENCES <other_table>(id)` (add `ON DELETE` policy based on context — default to `RESTRICT`).
- Always include `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` and `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`.
- Add an `updated_at` trigger using the project's existing trigger function if one exists, otherwise create it once and reuse it.
- Add indexes on foreign key columns and status columns.

**Type mapping:**
| Domain type  | Postgres type     |
|-------------|-------------------|
| UUID        | UUID              |
| String      | TEXT              |
| Integer     | BIGINT            |
| Float       | NUMERIC           |
| Boolean     | BOOLEAN           |
| Date        | DATE              |
| DateTime    | TIMESTAMPTZ       |
| Enum        | TEXT + CHECK      |
| JSONB list  | JSONB             |

---

## Step 2 — Rust Feature (one per aggregate)

Follow the example feature's module structure exactly. For each aggregate, create:

```
src/<aggregate_snake_case>/
  mod.rs
  domain.rs
  repository.rs
  handlers.rs
  errors.rs
```

### `domain.rs`
- One main struct for the aggregate with all fields matching the migration.
- Serde-serializable sub-structs for each JSONB schema.
- A `Status` enum with all statuses. Implement `Display`, `FromStr`, and serde derives.
- One method per capability on the aggregate struct. Each method:
  - Takes the required input fields as parameters.
  - Validates invariants — return `Err(DomainError)` if violated.
  - Performs the state transition.
  - Returns `Ok(Self)` or `Ok(Event)` depending on project convention.
- Invariant checks are pure functions — no I/O.

### `repository.rs`
- A `Repository` trait with one method per persistence operation needed:
  - `find_by_id(id: Uuid) -> Result<Option<Aggregate>, RepoError>`
  - `save(aggregate: &Aggregate) -> Result<(), RepoError>`
  - Any additional query methods needed by handlers (e.g. `find_by_status`)
- A `PostgresRepository` struct implementing the trait using `sqlx`.
- Use `sqlx::query_as!` macro if the example feature uses it, otherwise follow the example's query style.
- JSONB fields are serialized/deserialized using `serde_json`.

### `handlers.rs`
- One handler function per capability plus one for reads (get by id, list).
- Handlers are thin: deserialize input → load aggregate → call domain method → save → serialize response.
- Route naming follows domain capabilities, NOT CRUD:
  - `GET    /<aggregates>`                        → list
  - `GET    /<aggregates>/:id`                    → get by id
  - `POST   /<aggregates>`                        → create (if create is a capability)
  - `POST   /<aggregates>/:id/<capability_name>`  → any other capability

### `errors.rs`
- A `FeatureError` enum covering domain errors, repository errors, and not-found cases.
- Implements the project's error response trait (follow the example feature).

### `mod.rs`
- Exports the router function that registers all routes for this aggregate.
- Follow the exact wiring pattern from the example feature.

### Wire into main router
- Add the aggregate's router to the main application router, following the existing pattern.

---

## Step 3 — OpenAPI Spec (`openapi.json`)

Create or update `openapi.json` at the project root.

Rules:
- OpenAPI 3.1.
- One `tag` per aggregate.
- Schema for each aggregate response object — fields match the domain model exactly.
- Schema for each capability request body — fields are the capability's input fields.
- Endpoints follow the same route structure as the handlers.
- Status field is a string enum in the schema listing all valid statuses.
- JSONB fields are represented as arrays of objects using the JSONB sub-schema.
- References between aggregates are UUID string fields (never nested objects).
- Include standard error responses: `400`, `404`, `422`, `500`.

---

## Step 4 — System Tests

For each aggregate, create:
`tests/<aggregate_snake_case>_lifecycle.rs`

Rules:
- Tests run against the real API (use `reqwest` or the project's existing HTTP test client).
- Tests run against a real Postgres instance (use the project's existing test DB setup).
- Each test file has a setup function that ensures a clean state for that aggregate's table.
- Test coverage required:
  - **Happy path for every capability** — including the full valid transition sequence.
  - **Lifecycle test** — one test that walks the aggregate through its entire valid lifecycle from creation to terminal state.
  - **Invariant violation tests** — one test per invariant, asserting the correct error response.
  - **Not found** — assert 404 when acting on a non-existent aggregate ID.
- Assertions check both the HTTP response status and the response body fields.
- Tests are independent — each creates its own aggregate instances and does not rely on shared state.

---

## Completion

After generating all files, output a summary:

```
## Implementation Complete

### Files Generated
- migrations/<NNN>_create_<aggregate>.sql
- src/<aggregate>/mod.rs
- src/<aggregate>/domain.rs
- src/<aggregate>/repository.rs
- src/<aggregate>/handlers.rs
- src/<aggregate>/errors.rs
- openapi.json
- tests/<aggregate>_lifecycle.rs

### Next Steps
1. Run `sqlx migrate run` to apply the migration.
2. Run `cargo build` to verify compilation.
3. Start the API and run `cargo test` to execute system tests.

### Notes
<list any NOTE: decisions made during generation>
```
