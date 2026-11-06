# UI conformance

The UI is the spec, so conformance means the unmodified UI works.

Tier 1 (fast, Rust): every operation in `plandev-contract` must validate
against our GraphQL schema. Lands with the skeleton server.

Tier 2 (real UI): `conformance/run-ui-e2e.sh` pins aerie-ui at the commit the
manifest was extracted from, starts our server, and runs the UI's own
Playwright suite. Pass `-g <pattern>` to run a subset.

Status: the server does not exist yet, so tier 2 cannot run. Some upstream
specs also need a real mission-model jar (`e2e-tests/data/*.jar`), so they are
gated on the model-ingestion milestone. Track passing specs in
`conformance/expected-passing.txt` and fail CI on regressions.
