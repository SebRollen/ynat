use anyhow::Result;

use ynat::App;

#[tokio::main]
async fn main() -> Result<()> {
    let token = ynat_auth::authenticate().await?;

    // Logging is initialized in App::run() with buffer support
    App::new(token).run().await?;

    Ok(())
}
