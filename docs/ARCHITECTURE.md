# PlanDev Rust: clean-room architecture

Goal: a backend that the unmodified PlanDev UI (aerie-ui) can talk to.
The UI is the spec. We never read the upstream backend source; the contract is
extracted from what the UI sends and expects back.

## Contract (done)

`plandev-contract/data/ui-operations.json` holds all 257 operations the UI
issues (143 mutations, 71 subscriptions, 43 queries), extracted from the UI's
`gql.ts`. Roughly 254 distinct root fields, over ~100 Hasura-style tables, plus
camelCase custom actions (`simulate`, `schedule`, `uploadDictionary`, ...).

Non-GraphQL surface (from the UI's `reqGateway` calls): `POST /auth/login`,
`GET /auth/session`, `/file`, `/modelExtraction`, `/importPlan`,
`/uploadActivities`, `/uploadDataset`, `/uploadExternalSource`,
`/importSequenceTemplate`, `/secrets`. Subscriptions use the `graphql-ws`
protocol on the same `/v1/graphql` path. Auth is `Authorization: Bearer`,
`x-hasura-role`, `x-hasura-user-id`.

## Layers

1. `plandev-contract`: manifest + conformance checks.
2. Hasura-compatible GraphQL engine: a metadata-driven schema (async-graphql
   `dynamic`) that, per registered table, emits the object type, `bool_exp`,
   `order_by`, `set_input`/`insert_input`, `*_by_pk`, `*_aggregate`,
   `insert_*_one`, `update_*_by_pk`, `delete_*`, and live subscriptions.
3. Storage: SQLite in memory first, behind a repository trait.
4. Domain services: plans/activities (changelog, anchors, snapshots, merge),
   models, simulation, constraints, scheduling, expansion, sequences.
5. Custom actions and the HTTP gateway.

## Open design tension: Toasty

Toasty models are static derive structs. The Hasura layer needs runtime
`where`/`order_by` over any table. Options: (a) a thin dynamic SQL layer over
the same SQLite connection for reads, Toasty for typed domain writes;
(b) Toasty everywhere with a macro that generates per-model filter translation.
Decision pending.

## Testing

See `conformance/README.md`: manifest-vs-schema checks plus the UI's own Playwright e2e suite run against this server.

## Milestones

1. Skeleton: server boots, login/session, schema serves every contract root
   field from empty tables; UI loads without errors.
2. Models, plans, activity directives CRUD, tags, presets, views, subscriptions.
3. Mission model ingestion (needs a decision on how models are supplied).
4. Simulation, constraints, scheduling, expansion.
