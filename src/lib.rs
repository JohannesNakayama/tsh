use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::db::{find_zettels_by_embedding, get_db, store_article, store_zettel};
use crate::llm::LlmClient;
use crate::model::{Article, Zettel};

pub mod db;
pub mod llm;
pub mod model;

/// Opens Neovim with a temporary buffer, optionally populated with initial data.
/// It waits for Neovim to close, then returns the final content of the buffer.
///
/// # Arguments
/// * `initial_content` - An optional string slice to populate the buffer with.
///
/// # Returns
/// A `Result` which is `Ok(String)` containing the buffer's final content on success,
/// or `Err(Box<dyn std::error::Error>)` if an error occurs.
pub fn open_and_edit_neovim_buffer(
    initial_content: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let mut temp_file = NamedTempFile::new()?;
    let temp_file_path = temp_file.path().to_owned();

    if let Some(content) = initial_content {
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?; // ensure all data is written to disk before Neovim opens
    }

    // Spawn Neovim as a child process.
    // We direct stdin/stdout/stderr to inherit from the parent process so the user can interact.
    let mut child = Command::new("nvim")
        .arg(&temp_file_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Wait for the Neovim process to complete. This makes our Rust program block
    // until the user closes Neovim.
    let status = child.wait()?;

    if !status.success() {
        eprintln!("Neovim exited with an error: {:?}", status);
        return Err("Neovim process exited with an error".into());
    }

    // Read the modified content from the temporary file after Neovim has closed.
    let mut edited_content = String::new();
    let mut file = fs::File::open(&temp_file_path)?;
    file.read_to_string(&mut edited_content)?;

    Ok(edited_content)
}

pub fn get_user_input() -> String {
    print!("> ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// TODO: separate workflow from logic here
pub async fn add_zettel(
    llm_client: &mut LlmClient,
    parents: &Vec<Zettel>,
) -> Result<(), Box<dyn Error>> {
    match open_and_edit_neovim_buffer(Some(combine_zettel_contents(parents.to_vec()).as_str())) {
        Ok(edited_content) => {
            // println!("\nNeovim closed. Edited content retrieved:");
            // println!("```");
            // println!("{}", edited_content);
            // println!("```");

            if let Ok(embedding) = llm_client.embed(&edited_content).await {
                let parent_ids = parents.iter().map(|zettel| zettel.id).collect();

                let mut conn = get_db("my_thoughts.db").await?;
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

pub fn combine_zettel_contents(zettels: Vec<Zettel>) -> String {
    zettels
        .iter()
        .map(|zettel| zettel.content.as_str())
        .collect::<Vec<&str>>()
        .join("\n\n")
}

pub async fn add_combined_zettel(llm_client: &mut LlmClient) -> Result<(), Box<dyn Error>> {
    println!("What topic would you like to write about?");
    let query = get_user_input();
    let zettels = find_zettels(llm_client, &query).await?;
    let buffer_content = combine_zettel_contents(zettels.clone());
    match open_and_edit_neovim_buffer(Some(&buffer_content)) {
        Ok(edited_content) => {
            // println!("\nNeovim closed. Edited content retrieved:");
            // println!("```");
            // println!("{}", edited_content);
            // println!("```");

            let parent_ids: Vec<i64> = zettels.iter().map(|t| t.id).collect();

            let mut conn = get_db("my_thoughts.db").await?;
            let tx = conn.transaction()?;

            if let Ok(embedding) = llm_client.embed(&edited_content).await {
                match store_zettel(&tx, &edited_content, embedding, parent_ids).await {
                    Ok(_) => {
                        tx.commit()?;
                        // println!("Application finished successfully.");
                    }
                    Err(_) => {
                        tx.rollback()?;
                        // eprintln!("Error storing content: {}", e)
                    }
                }
            }
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    Ok(())
}

pub async fn find_zettels(
    llm_client: &mut LlmClient,
    query: &str,
) -> Result<Vec<Zettel>, rusqlite::Error> {
    let query_embedding = match llm_client.embed(query).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error embedding query: {}", e);
            return Err(rusqlite::Error::InvalidQuery);
        }
    };

    let mut conn = get_db("my_thoughts.db").await?;
    let tx = conn.transaction()?;
    let zettels: Vec<Zettel> = find_zettels_by_embedding(&tx, query_embedding).await?;
    tx.commit()?;

    Ok(zettels)
}

pub async fn promote_zettel(zettel: Zettel, title: &str) -> Result<Article, rusqlite::Error> {
    let mut conn = get_db("my_thoughts.db").await?;
    let tx = conn.transaction()?;
    let article = store_article(&tx, zettel.id, title, &zettel.content).await?;
    tx.commit()?;
    Ok(article)
}
