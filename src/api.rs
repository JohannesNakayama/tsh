use std::error::Error;

use crate::{
    combine_zettel_contents,
    db::{find_zettels_by_embedding, get_db, store_zettel},
    llm::LlmClient,
    model::Zettel,
    open_and_edit_neovim_buffer,
    tui::app::LlmConfig,
};

// TODO: separate workflow from logic here
pub async fn add_zettel(
    db_path: &str,
    llm_config: &LlmConfig,
    parents: &Vec<Zettel>,
) -> Result<(), Box<dyn Error>> {
    let mut llm_client = LlmClient::from(llm_config);

    match open_and_edit_neovim_buffer(Some(combine_zettel_contents(parents.to_vec()).as_str())) {
        Ok(edited_content) => {
            if edited_content.is_empty() {
                return Ok(());
            }

            if let Ok(embedding) = llm_client.embed(&edited_content).await {
                let parent_ids = parents.iter().map(|zettel| zettel.id).collect();

                let mut conn = get_db(db_path).await?;
                let tx = conn.transaction()?;
                match store_zettel(&tx, &edited_content, embedding.clone(), parent_ids).await {
                    Ok(_) => {
                        tx.commit()?;
                        // println!("Application finished successfully.");
                    }
                    Err(_) => {
                        tx.rollback()?;
                        // eprintln!("Error storing content: {}", e);
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

// pub async fn add_combined_zettel(llm_client: &mut LlmClient) -> Result<(), Box<dyn Error>> {
//     println!("What topic would you like to write about?");
//     let query = get_user_input();
//     let zettels = find_zettels(llm_client, &query).await?;
//     let buffer_content = combine_zettel_contents(zettels.clone());
//     match open_and_edit_neovim_buffer(Some(&buffer_content)) {
//         Ok(edited_content) => {
//             // println!("\nNeovim closed. Edited content retrieved:");
//             // println!("```");
//             // println!("{}", edited_content);
//             // println!("```");

//             let parent_ids: Vec<i64> = zettels.iter().map(|t| t.id).collect();

//             let mut conn = get_db("my_thoughts.db").await?;
//             let tx = conn.transaction()?;

//             if let Ok(embedding) = llm_client.embed(&edited_content).await {
//                 match store_zettel(&tx, &edited_content, embedding, parent_ids).await {
//                     Ok(_) => {
//                         tx.commit()?;
//                         // println!("Application finished successfully.");
//                     }
//                     Err(_) => {
//                         tx.rollback()?;
//                         // eprintln!("Error storing content: {}", e)
//                     }
//                 }
//             }
//         }
//         Err(e) => eprintln!("Error interacting with Neovim: {}", e),
//     }

//     Ok(())
// }
