use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to get {0}: {1}")]
    Get(String, reqwest::Error),
    #[error("unable to post {0}: {1}")]
    Post(String, reqwest::Error),
    #[error("unable to put {0}: {1}")]
    Put(String, reqwest::Error),
    #[error("unable to patch {0}: {1}")]
    Patch(String, reqwest::Error),
    #[error("unable to delete {0}: {1}")]
    Delete(String, reqwest::Error),
    #[error("unable to parse res to json: {0}")]
    Json(reqwest::Error),
    #[error("unable to read queries: {0}")]
    Query(String),
    #[error("unable to read headers: {0}")]
    Header(String),
}
