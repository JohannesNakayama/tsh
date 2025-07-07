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

pub trait Screen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<AppCommand>;
    fn draw(&mut self, frame: &mut Frame);
}

pub struct App {
    should_quit: bool,
    // llm_client: LlmClient,
    current_screen: ActiveScreenType,
}

impl App {
    pub fn new(_llm_client: LlmClient) -> Self {
        Self {
            should_quit: false,
            // llm_client: llm_client,
            current_screen: ActiveScreenType::Main(MainMenuScreen::new()),
        }
    }

    fn handle_event(&mut self) -> color_eyre::Result<Option<AppCommand>> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match &mut self.current_screen {
                        ActiveScreenType::Main(screen) => {
                            return Ok(screen.handle_key_event(key));
                        }
                        // TODO: add handling for other screens
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
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = ratatui::init();

        loop {
            if let Some(command) = self.handle_event()? {
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
