use colored_json::ToColoredJson;
use reqwest::{Client, header};
use serde_json::Value;

use crate::Error;

// take in enum and pretty print based off html, json

pub struct Hurler {
    client: Client,
}

impl Hurler {
    pub fn new() -> Self {
        Hurler {
            client: Client::new(),
        }
    }

    pub async fn get(&self, path: String, queries: &[&str]) -> Result<(), Error> {
        let res = self
            .client
            .get(&path)
            .query(&self.get_queries(&queries)?)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Get(path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(|e| Error::Json(e)).unwrap()).unwrap();
        println!(
            "{}",
            serde_json::to_string_pretty(&json)
                .unwrap()
                .to_colored_json_auto()
                .unwrap()
        );
        Ok(())
    }

    pub async fn post(&self, path: String, body: String) -> Result<(), Error> {
        let res = self
            .client
            .post(&path)
            .body(body)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Post(path, e))?;

        self.pretty_p(&res.text().await.map_err(|e| Error::Json(e)).unwrap());
        Ok(())
    }

    pub async fn patch(&self, path: String, body: String) -> Result<(), Error> {
        let res = self
            .client
            .patch(&path)
            .body(body)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Patch(path, e))?;

        self.pretty_p(&res.text().await.map_err(|e| Error::Json(e)).unwrap());
        Ok(())
    }

    pub async fn put(&self, path: String, body: String) -> Result<(), Error> {
        let res = self
            .client
            .put(&path)
            .body(body)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Put(path, e))?;

        self.pretty_p(&res.text().await.map_err(|e| Error::Json(e)).unwrap());
        Ok(())
    }

    pub async fn delete(&self, path: String, body: String) -> Result<(), Error> {
        let res = self
            .client
            .delete(&path)
            // .body(body)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Delete(path, e))?;

        self.pretty_p(&res.text().await.map_err(|e| Error::Json(e)).unwrap());
        Ok(())
    }

    // head
    // options
    // connect
    // trace

    fn pretty_p(&self, res: &str) {
        let json: Value = serde_json::from_str(res).unwrap();
        println!(
            "{}",
            serde_json::to_string_pretty(&json)
                .unwrap()
                .to_colored_json_auto()
                .unwrap()
        );
    }

    fn get_queries<'a>(&self, queries: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, Error> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get() {
        let h = Hurler::new();
        h.get(
            "https://jsonplaceholder.typicode.com/todos/1".to_string(),
            &[],
        )
        .await
        .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_post() {
        let h = Hurler::new();
        h.post(
            "https://jsonplaceholder.typicode.com/posts".to_string(),
            r#"{"title": "foo", "body": "bar", "userId": 1}"#.to_string(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_patch() {
        let h = Hurler::new();
        h.patch(
            "https://jsonplaceholder.typicode.com/posts".to_string(),
            r#"{"title": "foo", "body": "bar", "userId": 1}"#.to_string(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_put() {
        let h = Hurler::new();
        h.put(
            "https://jsonplaceholder.typicode.com/posts".to_string(),
            r#"{"title": "foo", "body": "bar", "userId": 1}"#.to_string(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete() {
        let h = Hurler::new();
        h.delete(
            "https://jsonplaceholder.typicode.com/posts".to_string(),
            r#"{"title": "foo", "body": "bar", "userId": 1}"#.to_string(),
        )
        .await
        .unwrap();
    }
}
