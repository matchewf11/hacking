use axum::{
    Json, Router,
    extract::{Path, Query},
    http::HeaderMap,
    routing::{delete, get, patch, post, put},
};
use serde_json::{Value, json};
use std::collections::HashMap;

fn users() -> Value {
    json!([
        {"id": 1, "name": "freddy", "age": 69, "is_active": true, "roles": ["admin", "user"]},
        {"id": 2, "name": "foxy", "age": 34, "is_active": true, "roles": ["user"]},
        {"id": 420, "name": "chika", "age": 48, "is_active": false, "roles": ["admin"]},
        {"id": 69, "name": "bonnie", "age": 1003223, "is_active": true, "roles": ["admin", "funtimes"]},
    ])
}

fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", patch(patch_user))
        .route("/users/{id}", delete(delete_user));
    let routes = [
        ("GET", "/users"),
        ("GET", "/users/{id}"),
        ("POST", "/users"),
        ("PUT", "/users/{id}"),
        ("PATCH", "/users/{id}"),
        ("DELETE", "/users/{id}"),
    ];

    for (method, path) in routes {
        println!("  {:<8} http://localhost:3000{}", method, path);
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on :3000");
    axum::serve(listener, app).await.unwrap();
    let routes = [
        ("GET", "/users"),
        ("GET", "/users/{id}"),
        ("POST", "/users"),
        ("PUT", "/users/{id}"),
        ("PATCH", "/users/{id}"),
        ("DELETE", "/users/{id}"),
    ];

    println!("listening on :3000\n");
    for (method, path) in routes {
        println!("  {:<8} http://localhost:3000{}", method, path);
    }
}

async fn get_users(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Json<Value> {
    Json(json!({
        "method": "GET",
        "headers": headers_to_map(&headers),
        "query": query,
        "data": users(),
    }))
}

async fn get_user(Path(id): Path<u64>, headers: HeaderMap) -> Json<Value> {
    let user = users()
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["id"] == id)
        .cloned()
        .unwrap_or(json!(null));

    Json(json!({
        "method": "GET",
        "headers": headers_to_map(&headers),
        "data": user,
    }))
}

async fn create_user(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "method": "POST",
        "headers": headers_to_map(&headers),
        "body": body,
        "data": {"id": 4, "created": true},
    }))
}

async fn update_user(
    Path(id): Path<u64>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "method": "PUT",
        "headers": headers_to_map(&headers),
        "body": body,
        "data": {"id": id, "replaced": true},
    }))
}

async fn patch_user(
    Path(id): Path<u64>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "method": "PATCH",
        "headers": headers_to_map(&headers),
        "body": body,
        "data": {"id": id, "patched": true},
    }))
}

async fn delete_user(Path(id): Path<u64>, headers: HeaderMap) -> Json<Value> {
    Json(json!({
        "method": "DELETE",
        "headers": headers_to_map(&headers),
        "data": {"id": id, "deleted": true},
    }))
}
