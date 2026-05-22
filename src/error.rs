use thisError::Error;

#[derive(Error, Debug)]
pub enum HurlError {
    #[error("unable to get {0}")]
    GetError(String),
}
