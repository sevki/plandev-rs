//! HTTP gateway endpoints the UI calls outside GraphQL.

use axum::{routing::{get, post}, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use serde_json::{json, Value};

/// The UI only decodes the JWT (it never verifies it), so the skeleton issues
/// an unsigned token carrying the Hasura claims the UI reads.
fn mint_token(username: &str) -> String {
    let header = B64.encode(json!({"alg":"none","typ":"JWT"}).to_string());
    let claims = json!({
        "username": username,
        "iat": 0,
        "exp": 4_102_444_800u64,
        "https://hasura.io/jwt/claims": {
            "x-hasura-allowed-roles": ["user", "viewer", "aerie_admin"],
            "x-hasura-default-role": "aerie_admin",
            "x-hasura-user-id": username,
        }
    });
    format!("{header}.{}.", B64.encode(claims.to_string()))
}

async fn login(Json(body): Json<Value>) -> Json<Value> {
    let user = body.get("username").and_then(Value::as_str).unwrap_or("anonymous");
    Json(json!({"success": true, "message": "", "token": mint_token(user)}))
}

async fn session() -> Json<Value> {
    Json(json!({"success": true, "message": ""}))
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/login", post(login))
        .route("/auth/session", get(session))
}
