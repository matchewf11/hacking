use hurl::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    hurl::start().await;
    Ok(())
}
