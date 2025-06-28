use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use async_openai::Client;
use sqlx::prelude::FromRow;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{query, query_as, Sqlite, SqlitePool, Transaction};
use tempfile::NamedTempFile;

use crate::model::{Edge, Thought};

mod model;

/// Opens Neovim with a temporary buffer, optionally populated with initial data.
/// It waits for Neovim to close, then returns the final content of the buffer.
///
/// # Arguments
/// * `initial_content` - An optional string slice to populate the buffer with.
///
/// # Returns
/// A `Result` which is `Ok(String)` containing the buffer's final content on success,
/// or `Err(Box<dyn std::error::Error>)` if an error occurs.
fn open_and_edit_neovim_buffer(
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

fn get_user_input() -> String {
    print!("What are you thinking about?");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub async fn get_db_connection(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn store_thought(tx: &mut Transaction<'_, Sqlite>, content: &str) -> Result<Thought, sqlx::Error> {
    let thought: Thought = query_as("
            insert into thought (content) values ($1) returning *
        ")
        .bind(content)
        .fetch_one(&mut **tx)
        .await?;
    query("
            insert into edge (node_id) values ($1)
        ")
        .bind(thought.id)
        .execute(&mut **tx)
        .await?;
    Ok(thought)
}

async fn add_thought() -> Result<(), Box<dyn Error>> {
    let pool: SqlitePool = get_db_connection("sqlite://my_thoughts.db").await?;

    let initial_thought = get_user_input();

    match open_and_edit_neovim_buffer(Some(&initial_thought)) {
        Ok(edited_content) => {
            println!("\nNeovim closed. Edited content retrieved:");
            println!("```");
            println!("{}", edited_content);
            println!("```");

            let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;
            match store_thought(&mut tx, &edited_content).await {
                Ok(_) => println!("Application finished successfully."),
                Err(e) => eprintln!("Error storing content: {}", e),
            }
            tx.commit().await?;
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    Ok(())
}

fn embed(thought: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    Ok(vec![0.1, 0.2, 0.3])
}

async fn chat(prompt: &str) -> Result<(), Box<dyn Error>> {
    let api_key = "ollama";
    let api_base = "http://localhost:11434/v1";

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base)
        );

    let model = "llama3.2:1b";

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model(model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant with a French accent.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    println!("\nResponse:\n");
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // add_thought().await?;

    // let thought = "abcde";

    // match embed(thought) {
    //     Ok(result) => println!("{:?}", result),
    //     Err(e) => eprintln!("{:?}", e),
    // }

    let thought = get_user_input();

    chat(&thought).await?;

    Ok(())
}
