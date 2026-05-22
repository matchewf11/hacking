use reqwest::{
    blocking::{Client, ClientBuilder},
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
        Ok(())
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {

    }
}
