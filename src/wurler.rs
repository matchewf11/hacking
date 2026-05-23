use std::{collections::HashMap, time::Instant};

use colored_json::ToColoredJson;
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

use crate::{
    Error,
    cli::RequestFlags,
    parser::json::parse_json,
};

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
        let resp = req.send().await.map_err(|e| Error::Http(url.to_string(), e))?;
        let duration_ms = started.elapsed().as_millis();

        let status = resp.status().as_u16();

        // Collect lowercased headers before consuming the body.
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_lowercase(), v.to_str().unwrap_or("").to_string()))
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
