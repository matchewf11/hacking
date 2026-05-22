use reqwest::{
    blocking::{Client, ClientBuilder},
};
// serde json
// request -> assume json -> pretty print
// take in enum and pretty print based off html, xml, json

// take string and req

pub struct Hurler {
    client: Client,
}

pub enum Error {

}

impl Hurler {
    pub fn new() -> Self {
        Hurler {
            client: Client::new(),
        }
    }

    pub fn get(path: String) -> Result<(), Error> {
        Ok(())
    }
}
