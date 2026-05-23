use wurl::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = wurl::start().await?;
    Ok(())
}
