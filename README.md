# PlanDev Rust

This repository is the Rust port of selected PlanDev libraries. The workspace
currently contains:

- `merlin-framework`: model-action commands and microsecond-duration primitives.
- `scheduler-driver`: planning horizons, activities, and validated in-memory
  plans built on the Merlin time type.

Run the workspace tests with:

```sh
cargo test --workspace
```

These crates establish the initial Rust API and data-model foundations. They do
not yet provide the Java projects' full simulation runtime, goal/constraint
system, or scheduling solver.
