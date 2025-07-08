use std::error::Error;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::{
    api::find_zettels,
    model::Zettel,
    tui::{
        app::{ActiveScreenType, AppCommand, Screen},
        main_menu::MainMenuScreen,
    },
};

pub struct IterateZettelScreen {
    input_mode: InputMode,
    search_query: String,
    search_results: Vec<Zettel>,
}

enum InputMode {
    Insert,
    Normal,
}

enum IterateScreenMessage {
    BackToMainMenu,
    EnterInsertMode,
    ExitInsertMode,
    InsertChar(char),
    DeleteChar,
    SubmitQuery(String),
}

impl IterateZettelScreen {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_results: vec![],
        }
    }

    fn handle_key_event_internal(&mut self, key: KeyEvent) -> Option<IterateScreenMessage> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Enter => {
                    Some(IterateScreenMessage::SubmitQuery(self.search_query.clone()))
                }
                KeyCode::Char('i') => Some(IterateScreenMessage::EnterInsertMode),
                KeyCode::Char('q') | KeyCode::Esc => Some(IterateScreenMessage::BackToMainMenu),
                _ => None,
            },
            InputMode::Insert => match key.code {
                KeyCode::Char(c) => Some(IterateScreenMessage::InsertChar(c)),
                KeyCode::Backspace => Some(IterateScreenMessage::DeleteChar),
                KeyCode::Enter => {
                    Some(IterateScreenMessage::SubmitQuery(self.search_query.clone()))
                }
                KeyCode::Esc => Some(IterateScreenMessage::ExitInsertMode),
                _ => None,
            },
        }
    }

    async fn update(&mut self, message: IterateScreenMessage) -> Result<(), Box<dyn Error>> {
        match message {
            IterateScreenMessage::EnterInsertMode => {
                self.input_mode = InputMode::Insert;
            }
            IterateScreenMessage::ExitInsertMode => {
                self.input_mode = InputMode::Normal;
            }
            IterateScreenMessage::InsertChar(c) => {
                self.search_query.push(c);
            }
            IterateScreenMessage::DeleteChar => {
                self.search_query.pop();
            }
            IterateScreenMessage::SubmitQuery(query) => {
                self.search_results = find_zettels(&query).await?;
            }
            _ => {}
        };
        Ok(())
    }
}

impl Screen for IterateZettelScreen {
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<AppCommand>, Box<dyn Error>> {
        if let Some(msg) = self.handle_key_event_internal(key) {
            match msg {
                IterateScreenMessage::BackToMainMenu => Ok(Some(AppCommand::SwitchScreen(
                    ActiveScreenType::Main(MainMenuScreen::new()),
                ))),
                _ => {
                    self.update(msg).await?;
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(0)],
        )
        .split(f.area());

        let search_box_style: Style = match self.input_mode {
            InputMode::Insert => Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            InputMode::Normal => Style::default(),
        };

        let search_box = Paragraph::new(self.search_query.to_string())
            .style(search_box_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Query"),
            );

        let search_results: Vec<ListItem> = self
            .search_results
            .iter()
            .map(|zettel| ListItem::new(zettel.content.to_string()))
            .collect();

        let search_results_list = List::new(search_results);

        f.render_widget(search_box, layout[0]);
        f.render_widget(search_results_list, layout[1]);
    }
}
