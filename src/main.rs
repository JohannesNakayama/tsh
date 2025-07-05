use std::error::Error;
use tsh::{llm::LlmClient, tui::app::MainMenu};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: load from config, not env
    // let db_url = std::env::var("DATABASE_URL")?;
    let api_base = std::env::var("API_BASE")?;
    let api_key = std::env::var("API_KEY")?;
    let embedding_model = std::env::var("EMBEDDINGS_MODEL")?;
    let chat_model = std::env::var("CHAT_MODEL")?;

    // TODO: is it a good idea to run this every time?
    // migrate_to_latest(&db_url).await?;

    let llm_client = LlmClient::new(api_base, api_key, embedding_model, chat_model);

    // let mut conn = get_db(&db_url).await?;
    // let tx = conn.transaction()?;
    // let parent = find_zettel_by_id(&tx, 1).await?;
    // tx.commit()?;
    // add_zettel(&mut llm_client, &vec![]).await?;
    // chat().await?;
    // add_combined_zettel(&mut llm_client).await?;

    color_eyre::install()?; // TODO: where best to call this?
    let _app_result = MainMenu::new(llm_client).run().await?;

    Ok(())
}
