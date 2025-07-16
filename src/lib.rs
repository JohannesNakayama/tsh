use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::db::{get_db, store_article};
use crate::model::{Article, Zettel};

pub mod db;
pub mod llm;
pub mod model;
pub mod tui {
    pub mod app;
    pub mod common;
    pub mod iterate;
    pub mod main_menu;
    pub mod recent;
}
pub mod api;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub data_dir: Option<String>,
    pub api_base: String,
    pub api_key: String,
    pub embeddings_model: String,
}

pub fn load_config(cfg_path: Option<String>) -> Result<AppConfig, Box<dyn Error>> {
    let path = match cfg_path {
        Some(path) => path,
        None => {
            let home_dir = std::env::var("HOME")?;
            let path = format!("{}/.config/tsh/config.toml", home_dir);
            path
        }
    };
    fs::read_to_string(path)
        .map_err(|e| e.into())
        .and_then(|content| toml::from_str(&content).map_err(|e| Box::new(e) as Box<dyn Error>))
        .map(|config: AppConfig| config)
}

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

pub fn combine_zettel_contents(zettels: Vec<Zettel>) -> String {
    zettels
        .iter()
        .map(|zettel| zettel.content.as_str())
        .collect::<Vec<&str>>()
        .join("\n\n")
}

pub async fn promote_zettel(
    zettel: Zettel,
    title: &str,
    db_path: &str,
) -> Result<Article, rusqlite::Error> {
    let mut conn = get_db(db_path).await?;
    let tx = conn.transaction()?;
    let article = store_article(&tx, zettel.id, title, &zettel.content).await?;
    tx.commit()?;
    Ok(article)
}
