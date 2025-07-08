use std::error::Error;
use tsh::{db::migrate_to_latest, tui::app::App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: load from config, not env
    let db_url = std::env::var("DATABASE_URL")?;

    migrate_to_latest(&db_url).await?;

    color_eyre::install()?; // TODO: where best to call this?
    let mut tsh_app = App::new();
    tsh_app.run().await?;

    Ok(())
}
