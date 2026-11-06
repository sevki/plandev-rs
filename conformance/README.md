# UI conformance

The UI is the spec, so conformance means the unmodified UI works.

Tier 1 (fast, Rust): every operation in `plandev-contract` must validate
against our GraphQL schema. Lands with the skeleton server.

Tier 2 (real UI): `conformance/run-ui-e2e.sh` pins aerie-ui at the commit the
manifest was extracted from, starts our server, and runs the UI's own
Playwright suite. Pass `-g <pattern>` to run a subset.

Status: tier 1 passes for all 257 operations. Tier 2 has not been run yet (the server returns empty data for everything). Some upstream
specs also need a real mission-model jar (`e2e-tests/data/*.jar`), so they are
gated on the model-ingestion milestone. Track passing specs in
`conformance/expected-passing.txt` and fail CI on regressions.
