---
name: malky-outbox-pattern
description: Implements the Transactional Outbox Pattern for any API. Generates the outbox table migration, Debezium connector configuration (with field-by-field explanation), Schema Registry Avro schemas, and Kafka topic naming for a given aggregate. Use when asked to "add outbox", "implement the outbox pattern", "set up CDC for an aggregate", or "add Kafka events to a domain".
argument-hint: (no arguments needed — starts an interactive session to gather context)
allowed-tools: Read, Write, Edit, Glob, Bash
---

# Skill: Outbox Pattern Implementation

## Context

You are a backend architecture expert implementing the **Transactional Outbox Pattern**.
You work inside any API project (Rust, Java, Python, Go, etc.) — the pattern is language-agnostic.
Your job is to gather context, then generate all outbox infrastructure artifacts with detailed explanations.

You do NOT generate application business logic. You generate:
- The outbox table SQL migration
- The Debezium connector JSON configuration (with field-by-field annotation)
- The Avro schemas for the Schema Registry
- The Kafka topic naming document

---

## Hard Rules — enforce these throughout

- The outbox insert MUST be in the **same database transaction** as the business write. Never allow an exception.
- `event_data` MUST contain the **complete DDD aggregate** at the moment of the event — never a partial delta.
- Every event carries a unique `event_id` (UUID). Consumers must be idempotent.
- Kafka topics are partitioned by `aggregate_id` to guarantee ordering per aggregate.
- Schema compatibility is always **FORWARD_TRANSITIVE** in the Schema Registry.
- The application NEVER writes directly to Kafka. Only Debezium does.

---

## Phase 1 — Gather Context

Ask the user for the following information. Present all questions at once in a numbered list.

1. **Company / namespace** — used for Avro schema namespaces and Snowflake database names. (e.g. `acme`, `norauto`)
2. **Domain name** — the bounded context. (e.g. `commerce`, `logistics`, `iam`) snake_case.
3. **Aggregate name** — the DDD aggregate being published. (e.g. `order`, `shipment`, `user`) snake_case.
4. **Event types** — the list of `event_type` values for this aggregate. Use `{AGGREGATE}_{VERB}` SCREAMING_SNAKE_CASE convention. (e.g. `ORDER_CREATED`, `ORDER_CANCELLED`, `ORDER_SHIPPED`). If the user is unsure, propose sensible defaults based on standard CRUD operations.
5. **Aggregate fields** — the fields of the aggregate (for the Avro schema). Ask for: field name, type (string / int / float / boolean / timestamp / uuid / array / object), nullable (yes/no).
6. **Environment** — target environment. (e.g. `public`, `staging`, `dev`)
7. **Kafka cluster identifier** — short identifier for the target Kafka cluster. (e.g. `c1`, `kafka-prod`)
8. **Schema version** — starting schema version. Default: `v1`.
9. **Database** — PostgreSQL (default). Note if different.

Tell the user: *"Don't worry about being perfect — I'll propose defaults and we can refine."*

Once the user provides answers (even partial), proceed to Phase 2.

---

## Phase 2 — Confirm & Derive Names

Derive all the names from the inputs and present them to the user for confirmation before generating anything.

Present a summary table:

```
## Derived Names — please confirm

| Artifact                        | Value                                                                 |
|---------------------------------|-----------------------------------------------------------------------|
| Outbox table                    | {domain}_{aggregate}_outbox                                           |
| Debezium connector name         | {domain}-{aggregate}-outbox-connector                                 |
| Kafka topic (events)            | {env}.{domain}.{aggregate}_event.{cluster}.{version}                  |
| Kafka topic (compacted)         | {env}.{domain}.{aggregate}.{cluster}.{version}                        |
| Avro schema (event envelope)    | com.{company}.{domain}.event.{Aggregate}Event                         |
| Avro schema (aggregate)         | com.{company}.{domain}.{Aggregate}                                    |
| Snowflake table (if applicable) | {COMPANY} / {DOMAIN}.{ENV} / {AGGREGATE}                              |
```

Ask: *"Do these names look correct? Any adjustments?"*

Once confirmed, proceed to Phase 3.

---

## Phase 3 — Generate Artifacts

Generate all four artifacts in order. For each one, write the file AND provide an inline explanation section.

### Artifact 1 — Outbox Table Migration

File: `migrations/{NNN}_create_{domain}_{aggregate}_outbox.sql`

Detect the next migration number by reading existing files in `migrations/`. If no migrations directory exists, start at `001`.

```sql
-- ============================================================
-- Outbox table for aggregate: {aggregate} (domain: {domain})
--
-- Purpose: This table is the "outbox" — the write side of the
-- Transactional Outbox Pattern. The application writes to this
-- table in the SAME transaction as the business tables. Debezium
-- reads it via CDC (WAL) and publishes to Kafka.
--
-- NEVER read from this table in application code for business
-- logic. It is a CDC source only.
-- ============================================================

CREATE TABLE {domain}_{aggregate}_outbox (

    -- event_id: Primary key. A UUID generated by the database.
    -- This is the idempotency key for downstream consumers.
    -- Consumers must store processed event_ids to avoid reprocessing
    -- in at-least-once delivery scenarios (Kafka rebalancing, etc.).
    event_id        UUID          PRIMARY KEY DEFAULT gen_random_uuid(),

    -- aggregate_type: The logical name of the DDD aggregate.
    -- Debezium's EventRouter SMT uses this to route events to the
    -- correct Kafka topic. Maps to: {env}.{domain}.{aggregate}_event.{cluster}.{version}
    aggregate_type  VARCHAR(100)  NOT NULL,

    -- aggregate_id: The business primary key of the aggregate instance.
    -- This becomes the Kafka message KEY, which guarantees ordering:
    -- all events for the same aggregate are in the same Kafka partition.
    -- Use the natural business ID (UUID, order_id, etc.).
    aggregate_id    VARCHAR(255)  NOT NULL,

    -- event_type: The semantic type of the event in SCREAMING_SNAKE_CASE.
    -- Convention: {AGGREGATE}_{VERB}  e.g. ORDER_CREATED, ORDER_CANCELLED.
    -- This field is preserved in the Avro envelope and used by consumers
    -- to decide how to process the event (routing, filtering, etc.).
    event_type      VARCHAR(100)  NOT NULL,

    -- event_date: Timestamp when the event was created (application time).
    -- Note: this is NOT the Kafka publish time. Use this for business
    -- ordering and auditing. Debezium adds its own processing timestamp
    -- in the Kafka message metadata.
    event_date      TIMESTAMPTZ   NOT NULL DEFAULT now(),

    -- event_data: The complete DDD aggregate serialized as JSON.
    -- RULE: This must always be the FULL aggregate state at the moment
    -- of the event — never a partial delta. Consumers must be able to
    -- reconstruct the aggregate state from this field alone.
    -- Debezium serializes this JSON value as an Avro 'string' field
    -- using the Confluent wire format (magic byte + schema ID + payload).
    event_data      JSONB         NOT NULL
);

-- Index on aggregate_id for Debezium CDC query performance and
-- for any operational queries (e.g. "did we emit an event for order X?").
CREATE INDEX ON {domain}_{aggregate}_outbox (aggregate_id);

-- Index on event_date for monitoring queries:
-- "how many events are pending?" / "what is the oldest unprocessed event?"
CREATE INDEX ON {domain}_{aggregate}_outbox (event_date);
```

After writing the file, print:

```
### Outbox Table — Key Points
- One outbox table per aggregate (not a shared global outbox).
- Insert into this table in the SAME transaction as your business INSERT/UPDATE.
- Do not add foreign key constraints from this table to business tables — it must
  be insertable even if the business tables are in flux (e.g. soft deletes).
- Debezium will DELETE rows after publishing (with the EventRouter SMT delete.handling.mode=rewrite).
  The table size should stay small. Monitor it: > 500 rows signals a CDC lag issue.
```

---

### Artifact 2 — Debezium Connector Configuration

File: `debezium/{domain}-{aggregate}-connector.json`

Generate the full connector JSON with an inline comment block above each field explaining its role and impact.

```json
{
  "name": "{domain}-{aggregate}-outbox-connector",

  "config": {
    // ── Connector class ──────────────────────────────────────────────────────
    // Use the PostgreSQL CDC connector. It reads the WAL (Write-Ahead Log)
    // using the pgoutput logical replication plugin (built into Postgres 10+).
    // No extra Postgres extensions required.
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",

    // ── Database connection ──────────────────────────────────────────────────
    "database.hostname": "${DB_HOST}",
    "database.port": "5432",
    "database.user": "${DB_USER}",
    "database.password": "${DB_PASSWORD}",
    "database.dbname": "${DB_NAME}",

    // ── Replication slot ────────────────────────────────────────────────────
    // A replication slot is a cursor in the WAL. Postgres holds WAL segments
    // until this slot has consumed them. RISK: if Debezium stops for a long
    // time, WAL accumulates and disk fills up. Monitor slot lag in production.
    // Name must be unique per connector in the Postgres cluster.
    "database.server.name": "{domain}_{aggregate}_server",
    "slot.name": "{domain}_{aggregate}_slot",
    "plugin.name": "pgoutput",

    // ── Table filtering ─────────────────────────────────────────────────────
    // Only watch the outbox table. This minimizes WAL parsing overhead.
    // Format: {schema}.{table}  — always use the public schema unless you
    // have a multi-schema setup.
    "table.include.list": "public.{domain}_{aggregate}_outbox",

    // ── Heartbeat ───────────────────────────────────────────────────────────
    // When the outbox table has no activity, the WAL slot does not advance.
    // The heartbeat forces a WAL write every N ms so the slot stays current.
    // Required in low-traffic environments to prevent WAL accumulation.
    "heartbeat.interval.ms": "10000",

    // ── EventRouter SMT (Single Message Transform) ───────────────────────────
    // The EventRouter transforms a raw CDC row-change event into a
    // business-domain Kafka event. It:
    //   1. Routes to the correct topic based on aggregate_type
    //   2. Sets the Kafka message KEY to aggregate_id
    //   3. Extracts event_data as the Kafka message VALUE
    //   4. Optionally deletes the outbox row after publishing (rewrite mode)
    "transforms": "outbox",
    "transforms.outbox.type": "io.debezium.transforms.outbox.EventRouter",

    // ── EventRouter field mappings ───────────────────────────────────────────
    // These map outbox table column names to the EventRouter's expected fields.
    // If your column names match the defaults (event_id, aggregate_type, etc.)
    // these explicit mappings are redundant but are listed for clarity.
    "transforms.outbox.table.field.event.id": "event_id",
    "transforms.outbox.table.field.event.key": "aggregate_id",
    "transforms.outbox.table.field.event.type": "event_type",
    "transforms.outbox.table.field.event.timestamp": "event_date",
    "transforms.outbox.table.field.event.payload": "event_data",

    // ── Topic routing ────────────────────────────────────────────────────────
    // The EventRouter routes to a topic derived from aggregate_type.
    // With route.by.field = aggregate_type, the topic name is the value
    // stored in the aggregate_type column of each row.
    // Store the FULL topic name in aggregate_type for explicit control:
    //   e.g. "{env}.{domain}.{aggregate}_event.{cluster}.{version}"
    // OR use the topic.regex.replace pattern below for dynamic construction.
    "transforms.outbox.route.by.field": "aggregate_type",
    "transforms.outbox.route.topic.replacement": "{env}.{domain}.{aggregate}_event.{cluster}.{version}",

    // ── Payload serialization (Avro + Schema Registry) ───────────────────────
    // The Confluent Avro converter serializes messages using the Confluent
    // wire format: [0x00][4-byte schema ID][Avro binary payload].
    // The schema ID is resolved from the Schema Registry at startup and
    // cached. On schema evolution, Debezium fetches the new schema ID.
    "value.converter": "io.confluent.connect.avro.AvroConverter",
    "value.converter.schema.registry.url": "${SCHEMA_REGISTRY_URL}",

    // schema.registry.value.subject.name.strategy controls how the subject
    // name is derived in the Schema Registry.
    // TopicNameStrategy (default): subject = {topic}-value
    //   → simple, one schema per topic
    // RecordNameStrategy: subject = {full Avro record name}
    //   → allows the same schema across multiple topics
    // Use TopicNameStrategy unless you have a specific reason to share schemas.
    "value.converter.value.subject.name.strategy": "io.confluent.kafka.serializers.subject.TopicNameStrategy",

    // ── Key serialization ────────────────────────────────────────────────────
    // The Kafka message key is aggregate_id (a string). Use StringConverter
    // to avoid wrapping it in Avro — this keeps consumer code simple and
    // allows topic compaction to work correctly (tombstone matching by key).
    "key.converter": "org.apache.kafka.connect.storage.StringConverter",

    // ── Delete handling ──────────────────────────────────────────────────────
    // rewrite: Debezium emits the event payload BEFORE deleting the row,
    // then emits a tombstone (null value) with the same key. The tombstone
    // is used by the compacted topic to signal aggregate deletion.
    // Alternative: none — rows are never deleted (outbox table grows forever).
    "transforms.outbox.table.op.invalid.behavior": "warn",
    "transforms.outbox.route.tombstone.on.empty.payload": "false",

    // ── Snapshot mode ────────────────────────────────────────────────────────
    // initial: On first start, Debezium snapshots all existing rows, then
    //          switches to streaming mode. Use this if you have pre-existing
    //          outbox rows that must be published.
    // never:   Skip snapshot entirely. Use this for a fresh outbox table.
    // schema_only: Snapshot schema but not data. Useful for restart scenarios.
    "snapshot.mode": "never",

    // ── Offset storage ───────────────────────────────────────────────────────
    // Debezium stores its WAL position (LSN) in Kafka Connect's offset store.
    // On restart it resumes from the last committed LSN — no events are lost
    // as long as the WAL slot has not been invalidated.
    // The topic below is the internal Kafka topic for offset storage.
    // It is NOT related to your business topics.
    "offset.storage.topic": "debezium.offsets",
    "offset.flush.interval.ms": "10000",

    // ── Error handling ───────────────────────────────────────────────────────
    // On deserialization errors, log and skip (warn) rather than failing the
    // entire connector. In production, consider dead-letter queue (DLQ) instead.
    "errors.tolerance": "none",
    "errors.log.enable": "true",
    "errors.log.include.messages": "true"
  }
}
```

After writing the file, print:

```
### Debezium Connector — Key Points
- Register this connector via POST to: http://{debezium-host}:8083/connectors
- Monitor task status via: GET http://{debezium-host}:8083/connectors/{name}/status
  A task in FAILED state means CDC is broken — events are NOT being published.
- The replication slot persists in Postgres. If you drop and recreate the connector,
  also drop the old slot: SELECT pg_drop_replication_slot('{domain}_{aggregate}_slot');
- aggregate_type in each outbox row controls Kafka topic routing. Store the full
  topic name there, or configure the route.topic.replacement pattern carefully.
```

---

### Artifact 3 — Avro Schemas

Write two schema files.

**File: `schemas/{domain}/{aggregate}_event.avsc`** — Event envelope schema (topic: events)

```json
{
  "type": "record",
  "name": "{Aggregate}Event",
  "namespace": "com.{company}.{domain}.event",
  "doc": "Event envelope for the {Aggregate} aggregate. Published to the events topic by Debezium via the Transactional Outbox Pattern. Contains the full aggregate state at the moment of the event.",
  "fields": [
    {
      "name": "event_id",
      "type": "string",
      "doc": "UUID of this event. Idempotency key for consumers."
    },
    {
      "name": "aggregate_type",
      "type": "string",
      "doc": "Always '{aggregate}'. Used by the EventRouter SMT for topic routing."
    },
    {
      "name": "aggregate_id",
      "type": "string",
      "doc": "Business primary key of the aggregate. Also the Kafka message key — guarantees ordering per aggregate."
    },
    {
      "name": "event_type",
      "type": {
        "type": "enum",
        "name": "{Aggregate}EventType",
        "symbols": [{event_type_list}],
        "doc": "Semantic event type. Consumers use this to route/filter events."
      },
      "doc": "The type of domain event that occurred."
    },
    {
      "name": "event_date",
      "type": {
        "type": "long",
        "logicalType": "timestamp-micros"
      },
      "doc": "Application timestamp (UTC) when the event was written to the outbox. NOT the Kafka publish time."
    },
    {
      "name": "event_data",
      "type": "string",
      "doc": "The complete {Aggregate} aggregate serialized as a JSON string. This is the full state, not a delta. Parse this string as JSON to hydrate the aggregate."
    }
  ]
}
```

**File: `schemas/{domain}/{aggregate}.avsc`** — Aggregate schema (topic: compacted)

Generate fields based on the aggregate fields the user provided in Phase 1.

```json
{
  "type": "record",
  "name": "{Aggregate}",
  "namespace": "com.{company}.{domain}",
  "doc": "Current state of the {Aggregate} aggregate. Published to the log-compacted topic. Represents the latest known state for a given aggregate_id.",
  "fields": [
    // ... one field per aggregate field provided by the user
    // Use these Avro type mappings:
    // string / uuid      → "string"
    // int / bigint       → "long"
    // float / decimal    → "double" (or bytes+decimal logical type for precision)
    // boolean            → "boolean"
    // timestamp          → {"type": "long", "logicalType": "timestamp-micros"}
    // nullable field     → ["null", <type>] with "default": null
    // array of objects   → {"type": "array", "items": <sub-record>}
  ]
}
```

After writing both files, print:

```
### Avro Schemas — Key Points
- Schema compatibility mode: FORWARD_TRANSITIVE (configure in Schema Registry per subject).
  Command: curl -X PUT {SCHEMA_REGISTRY_URL}/config/{topic}-value \
    -H "Content-Type: application/json" \
    -d '{"compatibility": "FORWARD_TRANSITIVE"}'
- Breaking changes (field rename, field removal, type change) require a new topic version.
  Increment {version} in the topic name (v1 → v2) and run a dual-publish migration period.
- Non-breaking changes (adding an optional field with a default) can be done in-place
  on the same topic — just register the new schema version in the Registry.
- Register schemas before deploying the Debezium connector. The connector validates
  schema compatibility on startup and fails fast if incompatible.
```

---

### Artifact 4 — Kafka Topic Naming Document

File: `docs/kafka-topics-{domain}-{aggregate}.md`

```markdown
# Kafka Topics — {Aggregate} ({Domain})

## Topics

| Topic | Type | Retention | Compaction | Partitions | Purpose |
|---|---|---|---|---|---|
| `{env}.{domain}.{aggregate}_event.{cluster}.{version}` | Events | 7 days | No | 6 (or match DB partitions) | Raw event history — every event ever emitted |
| `{env}.{domain}.{aggregate}.{cluster}.{version}` | Compacted | Forever | Yes (log.cleanup.policy=compact) | 6 (match events topic) | Current aggregate state — latest event per key |

## Naming Convention

`{env}.{domain}.{aggregate}[_event].{cluster}.{version}`

| Segment | Value | Notes |
|---|---|---|
| `env` | `{env}` | Deployment environment |
| `domain` | `{domain}` | Bounded context (snake_case) |
| `aggregate` | `{aggregate}` | DDD aggregate name (snake_case) |
| `_event` suffix | Present on events topic, absent on compacted topic | Distinguishes raw history from current state |
| `cluster` | `{cluster}` | Kafka cluster identifier |
| `version` | `{version}` | Schema version — increment on breaking changes |

## Partitioning

- Both topics are partitioned by **`aggregate_id`** (the Kafka message key).
- This guarantees that all events for a single aggregate are in the same partition → strict ordering per aggregate.
- Ordering between different aggregates is NOT guaranteed and must not be assumed.
- Recommended partition count: 6 (allows consumer parallelism while maintaining ordering guarantee).

## Kafka Topic Configuration

```bash
# Events topic
kafka-topics.sh --create \
  --topic {env}.{domain}.{aggregate}_event.{cluster}.{version} \
  --partitions 6 \
  --replication-factor 3 \
  --config retention.ms=604800000 \
  --config cleanup.policy=delete

# Compacted topic
kafka-topics.sh --create \
  --topic {env}.{domain}.{aggregate}.{cluster}.{version} \
  --partitions 6 \
  --replication-factor 3 \
  --config cleanup.policy=compact \
  --config min.cleanable.dirty.ratio=0.1 \
  --config delete.retention.ms=86400000
```

## Schema Subjects (Schema Registry)

| Subject | Schema | Compatibility |
|---|---|---|
| `{env}.{domain}.{aggregate}_event.{cluster}.{version}-value` | `com.{company}.{domain}.event.{Aggregate}Event` | FORWARD_TRANSITIVE |
| `{env}.{domain}.{aggregate}.{cluster}.{version}-value` | `com.{company}.{domain}.{Aggregate}` | FORWARD_TRANSITIVE |

## Monitoring

| Metric | Source | Alert Threshold |
|---|---|---|
| Debezium consumer lag | Kafka Connect / Prometheus | > 1000 messages or > 60 s |
| `{domain}_{aggregate}_outbox` table size | PostgreSQL | > 500 rows |
| Schema Registry incompatibility | Schema Registry | Any — immediate P1 |
```
```

---

## Phase 4 — Application Integration Checklist

After generating all artifacts, print an integration checklist tailored to the user's detected tech stack (or generic if unknown).

```
## Integration Checklist for {Aggregate} Outbox

### In your application service / use case:

- [ ] Open a database transaction
- [ ] Perform your business INSERT/UPDATE on the aggregate table(s)
- [ ] Build the complete aggregate as a JSON object (event_data)
- [ ] INSERT into {domain}_{aggregate}_outbox:
      - aggregate_type = '{aggregate}'  (or full topic name if using route.by.field)
      - aggregate_id   = <business primary key>
      - event_type     = '<EVENT_TYPE>'  (e.g. '{AGGREGATE}_CREATED')
      - event_data     = <aggregate as JSON>
- [ ] Commit the transaction
- [ ] On any error: rollback — NO event is emitted (the outbox row was never committed)

### Infrastructure:
- [ ] Apply migration: {NNN}_create_{domain}_{aggregate}_outbox.sql
- [ ] Create Kafka topics (see docs/kafka-topics-{domain}-{aggregate}.md)
- [ ] Register Avro schemas in Schema Registry (set FORWARD_TRANSITIVE compatibility)
- [ ] Deploy Debezium connector: POST debezium/{domain}-{aggregate}-connector.json
- [ ] Verify connector task status is RUNNING (not FAILED)
- [ ] Write an integration test that:
      - Posts an event to your API
      - Asserts the outbox row is created (or consumed) in Postgres
      - Consumes the Kafka events topic and validates the Avro-decoded payload

### Monitoring:
- [ ] Add alert: Debezium lag > 60s
- [ ] Add alert: outbox table size > 500 rows
- [ ] Add alert: Schema Registry compatibility failure on deploy
```

---

## Error Handling

If the user asks about error scenarios, explain using this table:

| Failure | What happens | Recovery |
|---|---|---|
| DB transaction fails | Rollback — outbox row never written — no event emitted | None needed; consistent by design |
| Debezium loses DB connection | WAL slot holds position — no events lost — auto-resumes from last LSN | Automatic |
| Kafka broker unavailable | Debezium retries with backoff — WAL slot holds data | Automatic |
| Schema incompatibility | Connector fails on startup — no events published | Fix schema, redeploy connector |
| Consumer crashes mid-processing | At-least-once delivery — consumer must use event_id for idempotency | Consumer implements dedup |
| Outbox table not cleaned (rewrite mode off) | Table grows indefinitely — monitor size | Enable delete.handling.mode=rewrite in EventRouter |
