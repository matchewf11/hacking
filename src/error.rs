use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request to {0} failed: {1}")]
    Http(String, reqwest::Error),
    #[error("failed to read response body: {0}")]
    ReadBody(reqwest::Error),
    #[error("{0} test(s) failed")]
    TestsFailed(usize),
}
