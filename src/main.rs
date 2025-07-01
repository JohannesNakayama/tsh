use tsh::add_combined_thought;
use std::error::Error;

mod model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // add_thought().await?;
    // chat().await?;
    add_combined_thought().await?;

    Ok(())
}
