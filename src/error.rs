use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to get {0}: {1}")]
    Get(String, reqwest::Error),
}
