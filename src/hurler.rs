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

impl Hurler {
    pub fn get(path: String) {

    }
}
