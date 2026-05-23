use colored_json::ToColoredJson;
use reqwest::{Client, RequestBuilder};
use serde_json::Value;

use crate::Error;

// Hurler object btw
pub struct Ho<'a> {
    path: String,
    body: Option<String>,
    queries: Option<Vec<(&'a str, &'a str)>>,
    // path args?
    headers: Option<Vec<(&'a str, &'a str)>>,
    // cookies?
    // etc??
}

impl<'a> Ho<'a> {
    pub fn new(
        path: String,
        body: Option<String>,
        queries: &'a [&'a str],
        headers: &'a [&'a str],
    ) -> Result<Self, Error> {
        let path = if !path.starts_with("http://") && !path.starts_with("https://") {
            format!("http://{}", path)
        } else {
            path
        };
        Ok(Ho {
            path,
            body,
            queries: Option::Some(Ho::get_queries(queries)?),
            headers: Option::Some(Ho::get_headers(headers)?),
        })
    }

    fn get_queries(queries: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, Error> {
        queries
            .iter()
            .map(|s| {
                let mut p = s.splitn(2, "=");
                let k = p.next().ok_or(Error::Query("missing key".to_string()))?;
                let v = p.next().ok_or(Error::Query("missing value".to_string()))?;
                Ok((k, v))
            })
            .collect()
    }

    fn get_headers(queries: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, Error> {
        queries
            .iter()
            .map(|s| {
                let mut p = s.splitn(2, ":");
                let k = p.next().ok_or(Error::Header("missing key".to_string()))?;
                let v = p.next().ok_or(Error::Header("missing value".to_string()))?;
                Ok((k, v))
            })
            .collect()
    }
}

pub struct Hurler {
    client: Client,
}

impl Default for Hurler {
    fn default() -> Self {
        Self::new()
    }
}

impl Hurler {
    pub fn new() -> Self {
        Hurler {
            client: Client::new(),
        }
    }

    pub async fn get(&self, ho: Ho<'_>) -> Result<serde_json::Value, Error> {
        let res = ho
            .headers
            .unwrap_or_default()
            .into_iter()
            .fold(
                self.client
                    .get(&ho.path)
                    .query(&ho.queries.unwrap_or_default()),
                |req: RequestBuilder, (k, v)| req.header(k, v),
            )
            .send()
            .await
            .map_err(|e| Error::Get(ho.path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(Error::Json).unwrap()).unwrap();
        self.pretty_p(&json);
        Ok(json)
    }

    pub async fn post(&self, ho: Ho<'_>) -> Result<serde_json::Value, Error> {
        let res = ho
            .headers
            .unwrap_or_default()
            .into_iter()
            .fold(
                self.client
                    .post(&ho.path)
                    .query(&ho.queries.unwrap_or_default())
                    .body(ho.body.unwrap_or_default()),
                |req: RequestBuilder, (k, v)| req.header(k, v),
            )
            .send()
            .await
            .map_err(|e| Error::Post(ho.path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(Error::Json).unwrap()).unwrap();
        self.pretty_p(&json);
        Ok(json)
    }

    pub async fn patch(&self, ho: Ho<'_>) -> Result<serde_json::Value, Error> {
        let res = ho
            .headers
            .unwrap_or_default()
            .into_iter()
            .fold(
                self.client
                    .patch(&ho.path)
                    .query(&ho.queries.unwrap_or_default())
                    .body(ho.body.unwrap_or_default()),
                |req: RequestBuilder, (k, v)| req.header(k, v),
            )
            .send()
            .await
            .map_err(|e| Error::Patch(ho.path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(Error::Json).unwrap()).unwrap();
        self.pretty_p(&json);
        Ok(json)
    }

    pub async fn put(&self, ho: Ho<'_>) -> Result<serde_json::Value, Error> {
        let res = ho
            .headers
            .unwrap_or_default()
            .into_iter()
            .fold(
                self.client
                    .put(&ho.path)
                    .query(&ho.queries.unwrap_or_default())
                    .body(ho.body.unwrap_or_default()),
                |req: RequestBuilder, (k, v)| req.header(k, v),
            )
            .send()
            .await
            .map_err(|e| Error::Put(ho.path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(Error::Json).unwrap()).unwrap();
        self.pretty_p(&json);
        Ok(json)
    }

    pub async fn delete(&self, ho: Ho<'_>) -> Result<serde_json::Value, Error> {
        let res = ho
            .headers
            .unwrap_or_default()
            .into_iter()
            .fold(
                self.client
                    .delete(&ho.path)
                    .query(&ho.queries.unwrap_or_default())
                    .body(ho.body.unwrap_or_default()),
                |req: RequestBuilder, (k, v)| req.header(k, v),
            )
            .send()
            .await
            .map_err(|e| Error::Delete(ho.path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(Error::Json).unwrap()).unwrap();
        self.pretty_p(&json);
        Ok(json)
    }

    // head
    // options
    // connect
    // trace

    #[inline]
    fn pretty_p(&self, res: &serde_json::Value) {
        println!(
            "{}",
            serde_json::to_string_pretty(res)
                .unwrap()
                .to_colored_json_auto()
                .unwrap()
        );
    }
}
