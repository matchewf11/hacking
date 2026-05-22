use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to get {0}")]
    Get(String),
}
