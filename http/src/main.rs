use axum::{
    Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch, post, put},
};
use serde_json::{Value, json};
use std::collections::HashMap;

fn users() -> Vec<Value> {
    vec![
        json!({"id": 1, "name": "freddy", "age": 69, "is_active": true, "roles": ["admin", "user"]}),
        json!({"id": 2, "name": "foxy", "age": 34, "is_active": true, "roles": ["user"]}),
        json!({"id": 420, "name": "chika", "age": 48, "is_active": false, "roles": ["admin"]}),
        json!({"id": 69, "name": "bonnie", "age": 1003223, "is_active": true, "roles": ["admin", "funtimes"]}),
    ]
}

fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect()
}

fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false)
}

fn users_to_html(users: &[Value]) -> String {
    let rows: String = users
        .iter()
        .map(|u| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                u["id"], u["name"], u["age"], u["is_active"]
            )
        })
        .collect();

    format!(
        "<html><body><table border='1'>\
        <tr><th>ID</th><th>Name</th><th>Age</th><th>Active</th></tr>\
        {}</table></body></html>",
        rows
    )
}

fn filter_users(users: &[Value], query: &HashMap<String, String>) -> Vec<Value> {
    users
        .iter()
        .filter(|u| {
            if let Some(id) = query.get("id") {
                if u["id"].to_string() != *id {
                    return false;
                }
            }
            if let Some(name) = query.get("name") {
                if u["name"].as_str().unwrap_or("") != name.as_str() {
                    return false;
                }
            }
            if let Some(active) = query.get("is_active") {
                if u["is_active"].to_string() != *active {
                    return false;
                }
            }
            true
        })
        .cloned()
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on :3000\n");
    println!("  {:<8} http://localhost:3000{}", "GET", "/users");
    println!("           ?id=1&name=hurley&is_active=true");
    println!("           Accept: text/html | application/json");
    println!("           X-Request-Id: <any>");
    println!("           Authorization: Bearer <token>");
    println!("  {:<8} http://localhost:3000{}", "GET", "/users/{id}");
    println!("           Accept: text/html | application/json");
    println!("  {:<8} http://localhost:3000{}", "POST", "/users");
    println!(
        "           Body: {{\"name\": str, \"age\": num, \"is_active\": bool, \"roles\": [str]}}"
    );
    println!("  {:<8} http://localhost:3000{}", "PUT", "/users/{id}");
    println!(
        "           Body: {{\"name\": str, \"age\": num, \"is_active\": bool, \"roles\": [str]}}"
    );
    println!("  {:<8} http://localhost:3000{}", "PATCH", "/users/{id}");
    println!(
        "           Body: any subset of {{\"name\": str, \"age\": num, \"is_active\": bool, \"roles\": [str]}}"
    );
    println!("  {:<8} http://localhost:3000{}", "DELETE", "/users/{id}");
    println!("  {:<8} http://localhost:3000{}", "POST", "/users");
    println!("  {:<8} http://localhost:3000{}", "PUT", "/users/{id}");
    println!("  {:<8} http://localhost:3000{}", "PATCH", "/users/{id}");
    println!("  {:<8} http://localhost:3000{}", "DELETE", "/users/{id}");

    axum::serve(listener, app).await.unwrap();
}

async fn get_users(headers: HeaderMap, Query(query): Query<HashMap<String, String>>) -> Response {
    let filtered = filter_users(&users(), &query);

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none");
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none");

    if wants_html(&headers) {
        return Html(users_to_html(&filtered)).into_response();
    }

    Json(json!({
        "method": "GET",
        "headers": headers_to_map(&headers),
        "query": query,
        "meta": {
            "x_request_id": request_id,
            "authorization": auth,
            "count": filtered.len(),
        },
        "data": filtered,
    }))
    .into_response()
}

async fn get_user(Path(id): Path<u64>, headers: HeaderMap) -> Response {
    let user = users().into_iter().find(|u| u["id"] == id);

    match user {
        Some(u) => {
            if wants_html(&headers) {
                return Html(users_to_html(&[u.clone()])).into_response();
            }
            Json(json!({
                "method": "GET",
                "headers": headers_to_map(&headers),
                "data": u,
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "user not found"})),
        )
            .into_response(),
    }
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
