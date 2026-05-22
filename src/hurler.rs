use crate::hurler::Error;
use reqwest::{
    blocking::{Client, ClientBuilder},
    header,
};
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
            .get(path)
            .header(header::ACCEPT, "applications/json")
            .send()
            .map_err(|e| Error::Get(path, e))?;
        println!(res);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {}
}
