use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::db::{find_thoughts_by_embedding, get_db, store_atomic_thought, store_combined_thought};
use crate::llm::embed;
use crate::model::Thought;

pub mod model;
pub mod db;
pub mod llm;


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
) -> Result<String, Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    let temp_file_path = temp_file.path().to_owned();

    if let Some(content) = initial_content {
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?; // ensure all data is written to disk before Neovim opens
    }

    println!("Opening Neovim at: {}", temp_file_path.display());
    println!("Edit the content and save/quit Neovim (e.g., :wq or :x) to continue...");

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


pub async fn add_thought() -> Result<(), Box<dyn Error>> {
    let initial_thought = get_user_input();

    match open_and_edit_neovim_buffer(Some(&initial_thought)) {
        Ok(edited_content) => {
            println!("\nNeovim closed. Edited content retrieved:");
            println!("```");
            println!("{}", edited_content);
            println!("```");

            if let Ok(embedding) = embed(&edited_content).await {
                let mut conn = get_db("my_thoughts.db").await?;
                let tx = conn.transaction()?;
                match store_atomic_thought(&tx, &edited_content, embedding).await {
                    Ok(_) => {
                        tx.commit()?;
                        println!("Application finished successfully.");
                    },
                    Err(e) => {
                        tx.rollback()?;
                        eprintln!("Error storing content: {}", e);
                    },
                }
            }
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    Ok(())
}

pub async fn combine_thoughts(thoughts: Vec<Thought>) -> Result<String, Box<dyn Error>> {
    let combined_thoughts = thoughts
        .iter()
        .map(|thought| thought.content.as_str())
        .collect::<Vec<&str>>()
        .join("\n\n");

    Ok(combined_thoughts)
}



pub async fn add_combined_thought() -> Result<(), Box<dyn Error>> {
    println!("What topic would you like to write about?");
    let query = get_user_input();
    let thoughts = find_thoughts(&query).await?;
    let buffer_content = combine_thoughts(thoughts.clone()).await?;
    match open_and_edit_neovim_buffer(Some(&buffer_content)) {
        Ok(edited_content) => {
            println!("\nNeovim closed. Edited content retrieved:");
            println!("```");
            println!("{}", edited_content);
            println!("```");

            let parent_ids: Vec<i64> = thoughts.iter().map(|t| t.id).collect();

            let mut conn = get_db("my_thoughts.db").await?;
            let tx = conn.transaction()?;

            if let Ok(embedding) = embed(&edited_content).await {
                match store_combined_thought(&tx, &edited_content, embedding, parent_ids).await {
                    Ok(_) => {
                        tx.commit()?;
                        println!("Application finished successfully.");
                    },
                    Err(e) => {
                        tx.rollback()?;
                        eprintln!("Error storing content: {}", e)
                    },
                }
            }
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    Ok(())
}

pub async fn find_thoughts(query: &str) -> Result<Vec<Thought>, rusqlite::Error> {
    let query_embedding = match embed(query).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error embedding query: {}", e);
            return Err(rusqlite::Error::InvalidQuery);
        }
    };

    let mut conn = get_db("my_thoughts.db").await?;
    let tx = conn.transaction()?;
    let thoughts: Vec<Thought> = find_thoughts_by_embedding(&tx, query_embedding).await?;

    Ok(thoughts)
}
