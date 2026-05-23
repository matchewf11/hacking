use std::{collections::HashMap, time::Instant};

use colored_json::ToColoredJson;
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

use crate::{Error, cli::RequestFlags, parser::json::parse_json};

// ── Response types ────────────────────────────────────────────────────────────

/// The full parsed response from a single HTTP request.
pub(crate) struct ResponseData {
    pub(crate) status: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) cookies: HashMap<String, ParsedCookie>,
    pub(crate) body_raw: String,
    pub(crate) body_json: Option<Value>,
    pub(crate) duration_ms: u128,
}

pub(crate) struct ParsedCookie {
    pub(crate) value: String,
    pub(crate) attributes: HashMap<String, Option<String>>,
}

// ── Wurler ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Wurler {
    client: Client,
}

impl Default for Wurler {
    fn default() -> Self {
        Self::new()
    }
}

impl Wurler {
    pub fn new() -> Self {
        Wurler {
            client: Client::new(),
        }
    }

    /// Send an HTTP request and return a fully-parsed [`ResponseData`].
    ///
    /// `url` must be scheme-absolute (`http://` / `https://`); callers are
    /// responsible for resolving base URLs and appending paths before calling.
    pub(crate) async fn send(
        &self,
        method: Method,
        url: &str,
        flags: &RequestFlags,
    ) -> Result<ResponseData, Error> {
        let req = self.build_request(method, url, flags);

        let started = Instant::now();
        let resp = req
            .send()
            .await
            .map_err(|e| Error::Http(url.to_string(), e))?;
        let duration_ms = started.elapsed().as_millis();

        let status = resp.status().as_u16();

        // Collect lowercased headers before consuming the body.
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_lowercase(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();

        // Parse every Set-Cookie header into a structured cookie.
        let cookies: HashMap<String, ParsedCookie> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(parse_set_cookie)
            .collect();

        let body_raw = resp.text().await.map_err(Error::ReadBody)?;
        let body_json = serde_json::from_str::<Value>(&body_raw).ok();

        Ok(ResponseData {
            status,
            headers,
            cookies,
            body_raw,
            body_json,
            duration_ms,
        })
    }

    /// Apply all [`RequestFlags`] (query params, headers, cookies, JSON body)
    /// to a fresh [`RequestBuilder`].  A single place for all request assembly.
    fn build_request(&self, method: Method, url: &str, flags: &RequestFlags) -> RequestBuilder {
        let mut req = self.client.request(method, url);

        // ── Query params: --query key=value ──────────────────────────────────
        let query_pairs: Vec<(&str, &str)> = flags
            .query
            .iter()
            .filter_map(|s| s.split_once('='))
            .collect();
        if !query_pairs.is_empty() {
            req = req.query(&query_pairs);
        }

        // ── Request headers: --headers key:value ─────────────────────────────
        for h in &flags.headers {
            if let Some((k, v)) = h.split_once(':') {
                req = req.header(k.trim(), v.trim());
            }
        }

        // ── Cookies: --cookies key=value → Cookie: k1=v1; k2=v2 ─────────────
        let cookie_str = flags
            .cookies
            .iter()
            .filter_map(|c| c.split_once('=').map(|(k, v)| format!("{k}={v}")))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
            req = req.header("Cookie", cookie_str);
        }

        // ── JSON body: --json key=value (dot-notation, auto-typed) ───────────
        if !flags.json.is_empty() {
            let refs: Vec<&str> = flags.json.iter().map(String::as_str).collect();
            req = req
                .header("Content-Type", "application/json")
                .body(parse_json(&refs).to_string());
        }

        req
    }

    /// Pretty-print a JSON value to stdout with color.
    pub(crate) fn pretty_print(val: &Value) {
        println!(
            "{}",
            serde_json::to_string_pretty(val)
                .unwrap()
                .to_colored_json_auto()
                .unwrap()
        );
    }
}

// ── Cookie parsing ────────────────────────────────────────────────────────────

/// Parse a raw `Set-Cookie` header into a `(name, ParsedCookie)` pair.
fn parse_set_cookie(raw: &str) -> (String, ParsedCookie) {
    let mut parts = raw.splitn(2, ';');
    let name_val = parts.next().unwrap_or("").trim();
    let attrs_raw = parts.next().unwrap_or("");

    let (name, value) = name_val
        .split_once('=')
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .unwrap_or_else(|| (name_val.to_string(), String::new()));

    let mut attributes: HashMap<String, Option<String>> = HashMap::new();
    for part in attrs_raw.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('=') {
            Some((k, v)) => {
                attributes.insert(k.trim().to_lowercase(), Some(v.trim().to_string()));
            }
            None => {
                attributes.insert(part.to_lowercase(), None);
            }
        }
    }

    (name, ParsedCookie { value, attributes })
}

#[cfg(test)]
mod tests {
    use crate::Wurler;
    use crate::cli::RequestFlags;
    use reqwest::Method;

    fn wurler() -> Wurler {
        Wurler::new()
    }

    fn flags(base: &str) -> RequestFlags {
        RequestFlags {
            base: base.into(),
            path: None,
            query: vec![],
            headers: vec![],
            cookies: vec![],
            json: vec![],
        }
    }

    // GET /users
    #[tokio::test]
    async fn get_users() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        assert_eq!(res.status, 200);
        assert!(res.body_json.unwrap()["data"].as_array().unwrap().len() > 0);
    }

    // GET /users filtered by id
    #[tokio::test]
    async fn get_users_filtered_by_id() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            query: vec!["id=1".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        let data = json["data"].as_array().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["id"], 1);
    }

    // GET /users filtered by name
    #[tokio::test]
    async fn get_users_filtered_by_name() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            query: vec!["name=freddy".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        let data = json["data"].as_array().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["name"], "freddy");
    }

    // GET /users filtered by active
    #[tokio::test]
    async fn get_users_filtered_by_active() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            query: vec!["is_active=false".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        let data = json["data"].as_array().unwrap();
        assert!(data.iter().all(|u| u["is_active"] == false));
    }

    // GET /users with Accept header
    #[tokio::test]
    async fn get_users_json_accept() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            headers: vec!["Accept: application/json".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        assert!(res.body_json.unwrap()["data"].is_array());
    }

    // GET /users with X-Request-Id
    #[tokio::test]
    async fn get_users_with_request_id() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            headers: vec!["X-Request-Id: test-123".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        assert_eq!(res.body_json.unwrap()["meta"]["x_request_id"], "test-123");
    }

    // GET /users with Authorization
    #[tokio::test]
    async fn get_users_with_auth() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            headers: vec!["Authorization: Bearer my-token-123".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        assert_eq!(
            res.body_json.unwrap()["meta"]["authorization"],
            "Bearer my-token-123"
        );
    }

    // GET /users/{id}
    #[tokio::test]
    async fn get_user_by_id() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users/1".into()),
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users/1", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["data"]["id"], 1);
        assert_eq!(json["data"]["name"], "freddy");
    }

    // POST /users
    #[tokio::test]
    async fn post_user() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            json: vec![
                "name=golden_freddy".into(),
                "age=30".into(),
                "is_active=true".into(),
            ],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::POST, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["method"], "POST");
        assert_eq!(json["body"]["name"], "golden_freddy");
        assert_eq!(json["data"]["created"], true);
    }

    // PUT /users/{id}
    #[tokio::test]
    async fn put_user() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users/1".into()),
            json: vec![
                "name=dark_freddy".into(),
                "age=70".into(),
                "is_active=false".into(),
            ],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::PUT, "http://localhost:3000/users/1", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["method"], "PUT");
        assert_eq!(json["body"]["name"], "dark_freddy");
        assert_eq!(json["data"]["replaced"], true);
    }

    // PATCH /users/{id}
    #[tokio::test]
    async fn patch_user() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users/2".into()),
            json: vec!["age=99".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::PATCH, "http://localhost:3000/users/2", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["method"], "PATCH");
        assert_eq!(json["body"]["age"], 99);
        assert_eq!(json["data"]["patched"], true);
    }

    // DELETE /users/{id}
    #[tokio::test]
    async fn delete_user() {
        let w = wurler();
        let f = flags("localhost:3000");
        let res = w
            .send(Method::DELETE, "http://localhost:3000/users/3", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["method"], "DELETE");
        assert_eq!(json["data"]["deleted"], true);
        assert_eq!(json["data"]["id"], 3);
    }

    // Cookies
    #[tokio::test]
    async fn get_with_cookies() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            cookies: vec!["session=abc123".into(), "theme=dark".into()],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        let cookie_header = json["headers"]["cookie"].as_str().unwrap();
        assert!(cookie_header.contains("session=abc123"));
        assert!(cookie_header.contains("theme=dark"));
    }

    // Multiple query + header combo
    #[tokio::test]
    async fn get_users_multiple_filters_and_headers() {
        let w = wurler();
        let f = RequestFlags {
            path: Some("/users".into()),
            query: vec!["id=1".into(), "is_active=true".into()],
            headers: vec![
                "X-Request-Id: combo-test".into(),
                "Authorization: Bearer tok".into(),
            ],
            ..flags("localhost:3000")
        };
        let res = w
            .send(Method::GET, "http://localhost:3000/users", &f)
            .await
            .unwrap();
        let json = res.body_json.unwrap();
        assert_eq!(json["meta"]["x_request_id"], "combo-test");
        assert_eq!(json["query"]["id"], "1");
        assert_eq!(json["query"]["is_active"], "true");
    }

    #[tokio::test]
    async fn response_headers_collected() {
        let w = wurler();
        let res = w
            .send(
                Method::GET,
                "http://localhost:3000/users",
                &flags("localhost:3000"),
            )
            .await
            .unwrap();
        assert!(res.headers.contains_key("content-type"));
    }
}
