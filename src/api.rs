use std::error::Error;

use crate::{
    combine_zettel_contents,
    db::{find_zettels_by_embedding, get_db, store_zettel},
    llm::LlmClient,
    model::Zettel,
    open_and_edit_neovim_buffer,
    tui::app::LlmConfig,
};

pub async fn add_zettel(
    db_path: &str,
    llm_config: &LlmConfig,
    parents: &Vec<Zettel>,
) -> Result<(), Box<dyn Error>> {
    let mut llm_client = LlmClient::from(llm_config);

    match open_and_edit_neovim_buffer(Some(combine_zettel_contents(parents.to_vec()).as_str())) {
        Ok(edited_content) => {
            // Don't save if:
            // - only one parent and content unchanged
            // - empty zettel
            let one_parent_and_content_unchanged =
                (parents.len() == 1) && (edited_content == parents.first().unwrap().content);
            if one_parent_and_content_unchanged || edited_content.is_empty() {
                return Ok(());
            }

            if let Ok(embedding) = llm_client.embed(&edited_content).await {
                let parent_ids: Vec<i64> = parents.iter().map(|zettel| zettel.id).collect();

                let mut conn = get_db(db_path).await?;
                let tx = conn.transaction()?;
                match store_zettel(&tx, &edited_content, embedding.clone(), parent_ids).await {
                    Ok(_) => {
                        tx.commit()?;
                    }
                    Err(e) => {
                        tx.rollback()?;
                        eprintln!("Error storing content: {}", e); // TODO: logging
                    }
                }
            }
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    Ok(())
}

pub async fn find_zettels(
    db_path: &str,
    llm_config: &LlmConfig,
    query: &str,
) -> Result<Vec<Zettel>, Box<dyn Error>> {
    let mut llm_client = LlmClient::from(llm_config);

    let query_embedding = llm_client.embed(query).await?;

    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let zettels: Vec<Zettel> = find_zettels_by_embedding(&tx, query_embedding).await?;
    tx.commit()?;

    Ok(zettels)
}
