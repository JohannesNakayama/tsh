use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::str::FromStr;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

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

/// Stores the provided content into a simple file-based "database".
/// For a real application, you would replace this with actual database integration (e.g., SQLite, PostgreSQL).
///
/// # Arguments
/// * `content` - The string content to store.
/// * `db_file_path` - The path to the file where content will be appended.
///
/// # Returns
/// A `Result` indicating success (`Ok(())`) or an error (`Err(Box<dyn std::error.Error>)`).
fn store_content(content: &str, db_file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Open the database file in append mode, creating it if it doesn't exist.
    let mut file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist.
        .append(true)  // Append to the file.
        .open(db_file_path)?;

    // Get the current timestamp.
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Write the content with a separator and timestamp.
    writeln!(file, "--- Entry on {} ---", timestamp)?;
    writeln!(file, "{}", content)?;
    writeln!(file, "--------------------\n")?;

    println!("Content successfully stored in: {}", db_file_path.display());
    Ok(())
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

fn main() {
    // Define the path for our "database" file.
    let mut db_file_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    db_file_path.push("my_thoughts.txt");

    let initial_thought = get_user_input();

    match open_and_edit_neovim_buffer(Some(&initial_thought)) {
        Ok(edited_content) => {
            println!("\nNeovim closed. Edited content retrieved:");
            println!("```");
            println!("{}", edited_content);
            println!("```");

            // Store the edited content.
            match store_content(&edited_content, &db_file_path) {
                Ok(_) => println!("Application finished successfully."),
                Err(e) => eprintln!("Error storing content: {}", e),
            }
        }
        Err(e) => eprintln!("Error interacting with Neovim: {}", e),
    }

    // You can also open an empty buffer:
    // match open_and_edit_neovim_buffer(None) {
    //     Ok(new_thought) => {
    //         println!("\nNew thought captured:");
    //         println!("{}", new_thought);
    //         // Store the new thought
    //     }
    //     Err(e) => eprintln!("Error capturing new thought: {}", e),
    // }
}
