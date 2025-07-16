use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyEvent},
};
use std::error::Error;

use crate::{
    AppConfig,
    api::add_zettel,
    model::Zettel,
    tui::{iterate::IterateZettelScreen, main_menu::MainMenuScreen, recent::RecentScreen},
};

pub enum ActiveScreenType {
    Main(MainMenuScreen),
    Iterate(IterateZettelScreen),
    Recent(RecentScreen),
}

pub enum AppCommand {
    Quit,
    AddZettel(Vec<Zettel>),
    SwitchScreen(ActiveScreenType),
}

#[trait_variant::make(ScreenMulti: Send)]
pub trait Screen {
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<AppCommand>, Box<dyn Error>>;
    fn draw(&mut self, frame: &mut Frame);
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_base: String,
    pub api_key: String,
    pub embeddings_model: String,
}

impl From<&AppConfig> for LlmConfig {
    fn from(config: &AppConfig) -> Self {
        LlmConfig {
            api_base: config.api_base.clone(),
            api_key: config.api_key.clone(),
            embeddings_model: config.embeddings_model.clone(),
        }
    }
}

pub struct App {
    should_quit: bool,
    current_screen: ActiveScreenType,
    db_path: String,
    llm_config: LlmConfig,
}

impl App {
    pub fn new(db_path: String, llm_config: LlmConfig) -> Self {
        Self {
            should_quit: false,
            current_screen: ActiveScreenType::Main(MainMenuScreen::new(
                db_path.clone(),
                llm_config.clone(),
            )),
            db_path,
            llm_config,
        }
    }

    async fn handle_event(&mut self) -> Result<Option<AppCommand>, Box<dyn Error>> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    return match &mut self.current_screen {
                        ActiveScreenType::Main(screen) => {
                            let maybe_action = screen.handle_key_event(key).await?;
                            Ok(maybe_action)
                        }
                        ActiveScreenType::Iterate(screen) => {
                            let maybe_action = screen.handle_key_event(key).await?;
                            Ok(maybe_action)
                        }
                        ActiveScreenType::Recent(screen) => {
                            let maybe_action = screen.handle_key_event(key).await?;
                            Ok(maybe_action)
                        }
                    };
                }
            }
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.current_screen {
            ActiveScreenType::Main(screen) => {
                screen.draw(frame);
            }
            ActiveScreenType::Iterate(screen) => {
                screen.draw(frame);
            }
            ActiveScreenType::Recent(screen) => {
                screen.draw(frame);
            }
        }
    }

    pub fn process_app_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::Quit => {
                self.should_quit = true;
            }
            AppCommand::SwitchScreen(screen_type) => match screen_type {
                ActiveScreenType::Main(screen) => {
                    self.current_screen = ActiveScreenType::Main(screen);
                }
                ActiveScreenType::Iterate(screen) => {
                    self.current_screen = ActiveScreenType::Iterate(screen);
                }
                ActiveScreenType::Recent(screen) => {
                    self.current_screen = ActiveScreenType::Recent(screen);
                }
            },
            _ => {}
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = ratatui::init();

        loop {
            if let Some(command) = self.handle_event().await? {
                match command {
                    AppCommand::AddZettel(parents) => {
                        // TODO: maybe use embedded neovim to avoid flickering (-> nvim-rs)
                        // Open an empty Zettel in neovim buffer
                        ratatui::restore();
                        add_zettel(&self.db_path, &self.llm_config, &parents).await?;
                        self.current_screen = ActiveScreenType::Main(MainMenuScreen::new(
                            self.db_path.clone(),
                            self.llm_config.clone(),
                        ));
                        terminal = ratatui::init();
                    }
                    _ => {
                        self.process_app_command(command);
                    }
                }
            }

            if self.should_quit {
                break;
            }

            terminal.draw(|f| self.draw(f))?;
        }

        ratatui::restore();

        Ok(())
    }
}
