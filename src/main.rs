use tsh::{add_combined_thought, add_thought, db::migrate_to_latest};
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    migrate_to_latest("my_thoughts.db").await?;

    // add_thought().await?;
    // chat().await?;
    add_combined_thought().await?;

    Ok(())
}
