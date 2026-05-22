use reqwest::{Client, header};
use serde_json::Value;

use crate::Error;

// serde json
// request -> assume json -> pretty print
// take in enum and pretty print based off html, xml, json

// take string and req

pub struct Hurler {
    client: Client,
}

impl Hurler {
    pub fn new() -> Self {
        Hurler {
            client: Client::new(),
        }
    }

    pub async fn get(&self, path: String) -> Result<(), Error> {
        let res = self
            .client
            .get(&path)
            .header(header::ACCEPT, "applications/json")
            .send()
            .await
            .map_err(|e| Error::Get(path, e))?;

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(|e| Error::Json(e)).unwrap()).unwrap();
        println!("{}", json);
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

        let json: Value =
            serde_json::from_str(&res.text().await.map_err(|e| Error::Json(e)).unwrap()).unwrap();
        println!("{}", json);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() {
        let h = Hurler::new();
        h.get("https://jsonplaceholder.typicode.com/todos/1".to_string())
            .await
            .unwrap();
    }

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
}
