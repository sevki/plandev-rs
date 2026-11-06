//! The wire contract the PlanDev UI expects from its backend.
//!
//! Clean-room provenance: `data/ui-operations.json` was extracted from the
//! UI's own `src/utilities/gql.ts` (MIT licensed, aerie-ui @ bd8312f). Each
//! entry is one operation the UI issues; the server is built against this
//! manifest, never against backend source.

use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Variable {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RootField {
    pub alias: Option<String>,
    pub field: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Operation {
    pub key: String,
    pub kind: Kind,
    pub name: Option<String>,
    pub vars: Vec<Variable>,
    pub roots: Vec<RootField>,
    /// Full GraphQL document text, usable for conformance tests.
    pub text: String,
}

/// All operations the UI issues.
pub fn operations() -> Vec<Operation> {
    serde_json::from_str(include_str!("../data/ui-operations.json"))
        .expect("embedded contract manifest is valid")
}

/// Distinct root fields per operation kind (the surface a server must serve).
pub fn root_fields(kind: Kind) -> BTreeSet<String> {
    operations()
        .into_iter()
        .filter(|o| o.kind == kind)
        .flat_map(|o| o.roots.into_iter().map(|r| r.field))
        .collect()
}

/// Names of Hasura-style tables referenced by root fields
/// (`plan`, `plan_by_pk`, `insert_plan_one`, `update_plan_by_pk`, ... -> `plan`).
pub fn tables() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for o in operations() {
        for r in o.roots {
            let f = r.field.as_str();
            if f.starts_with("__") || f.chars().any(|c| c.is_uppercase()) {
                continue; // introspection / camelCase custom actions
            }
            let f = f
                .strip_prefix("insert_")
                .or_else(|| f.strip_prefix("update_"))
                .or_else(|| f.strip_prefix("delete_"))
                .unwrap_or(f);
            let f = f
                .strip_suffix("_by_pk")
                .or_else(|| f.strip_suffix("_aggregate"))
                .or_else(|| f.strip_suffix("_one"))
                .or_else(|| f.strip_suffix("_many"))
                .unwrap_or(f);
            out.insert(f.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_loads() {
        let ops = operations();
        assert!(ops.len() > 250);
        assert!(root_fields(Kind::Subscription).contains("plan_by_pk"));
        assert!(root_fields(Kind::Mutation).contains("insert_plan_one"));
        assert!(tables().contains("activity_directive"));
    }
}
