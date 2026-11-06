use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{routing::get_service, Router};
use plandev_server::{gateway, schema};
use tower_http::cors::CorsLayer;

fn arg(name: &str, default: u16) -> u16 {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[tokio::main]
async fn main() {
    let schema = schema::build().expect("schema builds");
    let gql = Router::new()
        .route_service("/v1/graphql", get_service(GraphQLSubscription::new(schema.clone())).post_service(GraphQL::new(schema)))
        .layer(CorsLayer::permissive());
    let gw = gateway::router().layer(CorsLayer::permissive());
    let (gp, wp) = (arg("--port", 8080), arg("--gateway-port", 9000));
    let a = tokio::net::TcpListener::bind(("0.0.0.0", gp)).await.unwrap();
    let b = tokio::net::TcpListener::bind(("0.0.0.0", wp)).await.unwrap();
    println!("graphql on :{gp}, gateway on :{wp}");
    tokio::join!(async { axum::serve(a, gql).await.unwrap() }, async { axum::serve(b, gw).await.unwrap() });
}
