use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to get {0}: {1}")]
    Get(String, reqwest::Error),
    #[error("unable to post {0}: {1}")]
    Post(String, reqwest::Error),
    #[error("unable to parse res to json: {0}")]
    Json(reqwest::Error),
}
