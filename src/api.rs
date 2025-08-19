use std::error::Error;

use crate::{
    combine_zettel_contents,
    db::{
        self, add_tag_if_not_exists, delete_tag_for_zettel_if_exists, find_n_recent_leaf_zettels,
        find_zettels_by_embedding, get_db, get_tags_for_zettel, store_zettel,
    },
    llm::LlmClient,
    model::{Zettel, ZettelTag},
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

pub async fn get_n_recent_zettels(db_path: &str, n: i64) -> Result<Vec<Zettel>, Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let zettels: Vec<Zettel> = find_n_recent_leaf_zettels(&tx, n).await?;
    tx.commit()?;

    Ok(zettels)
}

pub async fn add_tag_to_zettel(
    db_path: &str,
    zettel_id: i64,
    tag: String,
) -> Result<(), Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    add_tag_if_not_exists(&tx, zettel_id, &tag).await?;
    tx.commit()?;
    Ok(())
}

pub async fn get_tags(db_path: &str, zettel_id: i64) -> Result<Vec<ZettelTag>, Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let tags = get_tags_for_zettel(&tx, zettel_id).await?;
    tx.commit()?;
    Ok(tags)
}

pub async fn delete_tag_from_zettel(
    db_path: &str,
    zettel_id: i64,
    tag: &str,
) -> Result<(), Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    delete_tag_for_zettel_if_exists(&tx, zettel_id, tag).await?;
    tx.commit()?;
    Ok(())
}

pub async fn find_tags(db_path: &str, search_string: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let tags = db::find_tags_by_search_string(&tx, search_string).await?;
    tx.commit()?;
    Ok(tags)
}

pub async fn get_zettels_by_tags(
    db_path: &str,
    tags: Vec<String>,
) -> Result<Vec<Zettel>, Box<dyn Error>> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let zettels = db::find_zettels_by_tags(&tx, tags).await?;
    tx.commit()?;
    Ok(zettels)
}
