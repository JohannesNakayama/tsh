use std::{error::Error, fs::create_dir_all};
use tsh::{
    db::migrate_to_latest,
    load_config,
    tui::app::{App, LlmConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let maybe_config_path: Option<String> = if args.len() == 2 {
        Some(args[1].clone())
    } else {
        None
    };

    let config = load_config(maybe_config_path)?;
    let llm_config = LlmConfig::from(&config);

    let data_dir = match config.data_dir {
        Some(path) => path,
        None => {
            let home_dir = std::env::var("HOME")?;
            format!("{}/.local/share/tsh/", home_dir)
        }
    };
    create_dir_all(&data_dir)?;
    let db_path = format!("{}/zettelkasten.db", data_dir);
    migrate_to_latest(&db_path).await?;

    let mut tsh_app = App::new(db_path, llm_config);
    tsh_app.run().await?;

    Ok(())
}
