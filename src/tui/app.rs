use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyEvent},
};
use std::error::Error;

use crate::{llm::LlmClient, tui::main_menu::MainMenuScreen};

pub enum ActiveScreenType {
    Main(MainMenuScreen),
}

pub enum AppCommand {
    Quit,
}

#[trait_variant::make(ScreenMulti: Send)]
pub trait Screen {
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<AppCommand>, Box<dyn Error>>;
    fn draw(&mut self, frame: &mut Frame);
}

pub struct App {
    should_quit: bool,
    // llm_client: LlmClient,
    current_screen: ActiveScreenType,
}

impl App {
    pub fn new(llm_client: LlmClient) -> Self {
        // TODO: refactor (use reference of llm client instead)
        // TODO: understand lifetimes properly...
        Self {
            should_quit: false,
            // llm_client: llm_client.clone(),
            current_screen: ActiveScreenType::Main(MainMenuScreen::new(llm_client.clone())),
        }
    }

    async fn handle_event(&mut self) -> Result<Option<AppCommand>, Box<dyn Error>> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match &mut self.current_screen {
                        ActiveScreenType::Main(screen) => {
                            let maybe_action = screen.handle_key_event(key).await?;
                            return Ok(maybe_action);
                        } // TODO: add handling for other screens
                    }
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
        }
    }

    pub fn process_app_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::Quit => {
                self.should_quit = true;
            } // TODO: open neovim buffer here, restore and re-init terminal
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = ratatui::init();

        loop {
            if let Some(command) = self.handle_event().await? {
                self.process_app_command(command);
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
