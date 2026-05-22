use reqwest::{
    blocking::{Client, ClientBuilder},
    header,
};
use serde_json::{Value, json};

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

    pub fn get(&self, path: String) -> Result<(), Error> {
        let res = self
            .client
            .get(&path)
            .header(header::ACCEPT, "applications/json")
            .send()
            .map_err(|e| Error::Get(path, e))?;

        let json: Value = serde_json::from_str(&res.text().unwrap()).unwrap();
        println!("{}", json);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {}
}
