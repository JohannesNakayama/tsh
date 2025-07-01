use tsh::{add_combined_thought, add_thought, db::migrate_to_latest, llm::LlmClient};
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    migrate_to_latest("my_thoughts.db").await?;

    // TODO: load from config, not env
    let api_base = std::env::var("API_BASE")?;
    let api_key = std::env::var("API_KEY")?;
    let embedding_model = std::env::var("EMBEDDINGS_MODEL")?;
    let chat_model = std::env::var("CHAT_MODEL")?;

    let mut llm_client = LlmClient::new(api_base, api_key, embedding_model, chat_model);

    // add_thought().await?;
    // chat().await?;
    add_combined_thought(&mut llm_client).await?;

    Ok(())
}
