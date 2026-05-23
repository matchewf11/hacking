use hurl::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = hurl::start().await?;
    Ok(())
}
