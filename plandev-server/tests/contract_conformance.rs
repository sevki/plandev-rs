//! Tier 1: every operation the UI issues must pass GraphQL validation against
//! our schema. Variables are not supplied, so "variable required" errors are
//! expected and ignored; any other error is a contract violation.

use async_graphql::Request;
use plandev_contract::operations;
use plandev_server::schema;

#[tokio::test]
async fn every_ui_operation_validates() {
    let schema = schema::build().expect("schema builds");
    let mut failures = Vec::new();
    for op in operations() {
        let req = Request::new(op.text.clone());
        let errors = if op.kind == plandev_contract::Kind::Subscription {
            use futures_util::StreamExt;
            let mut stream = schema.execute_stream(req);
            stream.next().await.map(|r| r.errors).unwrap_or_default()
        } else {
            schema.execute(req).await.errors
        };
        for e in errors {
            let m = e.message;
            if m.contains("is required") || (m.starts_with("Variable ") && m.ends_with("is not defined.")) {
                continue;
            }
            failures.push(format!("{}: {}", op.key, m));
        }
    }
    assert!(failures.is_empty(), "{} violations:\n{}", failures.len(), failures.join("\n"));
}
