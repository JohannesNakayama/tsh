use std::error::Error;
use tsh::{
    db::migrate_to_latest, llm::LlmClient, tui::app::{self, App}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: load from config, not env
    let db_url = std::env::var("DATABASE_URL")?;
    let api_base = std::env::var("API_BASE")?;
    let api_key = std::env::var("API_KEY")?;
    let embedding_model = std::env::var("EMBEDDINGS_MODEL")?;
    let chat_model = std::env::var("CHAT_MODEL")?;

    migrate_to_latest(&db_url).await?;

    let llm_client = LlmClient::new(api_base, api_key, embedding_model, chat_model);

    color_eyre::install()?; // TODO: where best to call this?
    let mut tsh_app = App::new(llm_client);
    app::run(&mut tsh_app).await?;

    Ok(())
}
