use std::error::Error;
use tsh::{
    db::migrate_to_latest,
    load_config,
    tui::app::{App, LlmConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config("./example_config.toml")?;
    let llm_config = LlmConfig::from(&config);
    migrate_to_latest(&config.db_path).await?;

    let mut tsh_app = App::new(config.db_path, llm_config);
    tsh_app.run().await?;

    Ok(())
}
