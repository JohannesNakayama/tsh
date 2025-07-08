use std::error::Error;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::{
    api::find_zettels,
    model::Zettel,
    tui::{
        app::{ActiveScreenType, AppCommand, LlmConfig, Screen},
        main_menu::MainMenuScreen,
    },
};

pub struct IterateZettelScreen {
    input_mode: InputMode,
    search_query: String,
    search_results: Vec<Zettel>,
    selected_result: Option<usize>,
    db_path: String,
    llm_config: LlmConfig,
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
    ResultListMoveUp,
    ResultListMoveDown,
    IterateZettel(Zettel),
}

impl IterateZettelScreen {
    pub fn new(db_path: String, llm_config: LlmConfig) -> Self {
        Self {
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_results: vec![],
            selected_result: None,
            db_path,
            llm_config,
        }
    }

    fn handle_key_event_internal(&mut self, key: KeyEvent) -> Option<IterateScreenMessage> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('i') => Some(IterateScreenMessage::EnterInsertMode),
                KeyCode::Char('q') | KeyCode::Esc => Some(IterateScreenMessage::BackToMainMenu),
                KeyCode::Up => Some(IterateScreenMessage::ResultListMoveUp),
                KeyCode::Down => Some(IterateScreenMessage::ResultListMoveDown),
                KeyCode::Enter => {
                    if let Some(idx) = self.selected_result {
                        let zettel = self.search_results[idx].clone();
                        Some(IterateScreenMessage::IterateZettel(zettel))
                    } else {
                        None
                    }
                }
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
                self.search_results = find_zettels(&self.db_path, &self.llm_config, &query).await?;
                if self.search_results.len() != 0 {
                    self.selected_result = Some(0);
                }
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            IterateScreenMessage::ResultListMoveUp => {
                if let Some(idx) = self.selected_result {
                    self.selected_result = Some(idx.saturating_sub(1));
                }
            }
            IterateScreenMessage::ResultListMoveDown => {
                if let Some(idx) = self.selected_result {
                    if idx + 1 < self.search_results.len() {
                        self.selected_result = Some(idx + 1);
                    }
                }
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
                IterateScreenMessage::BackToMainMenu => {
                    Ok(Some(AppCommand::SwitchScreen(ActiveScreenType::Main(
                        MainMenuScreen::new(self.db_path.clone(), self.llm_config.clone()),
                    ))))
                }
                IterateScreenMessage::IterateZettel(zettel) => {
                    Ok(Some(AppCommand::AddZettel(vec![zettel])))
                }
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

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[1]);

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
            .enumerate()
            .map(|(i, zettel)| {
                let mut item = ListItem::from(zettel);
                if let Some(idx) = self.selected_result {
                    if i == idx {
                        item = item.style(
                            Style::default()
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                }
                item
            })
            .collect();

        let search_results_list = List::new(search_results).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Search Results"),
        );

        let preview_paragraph = match self.selected_result {
            Some(idx) => {
                let selected_zettel = &self.search_results[idx];
                Paragraph::new(selected_zettel.content.to_string())
            }
            None => Paragraph::default(),
        };

        let preview = preview_paragraph.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Preview"),
        );

        f.render_widget(search_box, layout[0]);
        f.render_widget(search_results_list, inner_layout[0]);
        f.render_widget(preview, inner_layout[1]);
    }
}

impl From<&Zettel> for ListItem<'_> {
    fn from(zettel: &Zettel) -> Self {
        let lines = vec![
            Line::styled(
                format!("{}: {}", zettel.id, zettel.created_at),
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .fg(Color::LightBlue),
            ),
            Line::styled(format!("{}", zettel.content), Style::default()),
        ];
        ListItem::new(lines)
    }
}
