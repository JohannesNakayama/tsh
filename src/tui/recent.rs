use std::error::Error;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::{
    api::get_n_recent_zettels,
    model::Zettel,
    tui::{
        app::{ActiveScreenType, AppCommand, LlmConfig, Screen},
        main_menu::MainMenuScreen,
    },
};

pub struct RecentScreen {
    recent_zettels: Vec<Zettel>,
    selected_zettel: Option<usize>,
    db_path: String,
    llm_config: LlmConfig,
}

enum RecentScreenMessage {
    BackToMainMenu,
    ResultListMoveUp,
    ResultListMoveDown,
    IterateZettel(Zettel),
}

impl RecentScreen {
    pub async fn new(db_path: String, llm_config: LlmConfig) -> Result<Self, Box<dyn Error>> {
        let n_recent_zettels = get_n_recent_zettels(&db_path, 100).await?;
        Ok(Self {
            recent_zettels: n_recent_zettels.clone(),
            selected_zettel: if n_recent_zettels.is_empty() {
                None
            } else {
                Some(0)
            },
            db_path,
            llm_config,
        })
    }

    fn handle_key_event_internal(&mut self, key: KeyEvent) -> Option<RecentScreenMessage> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(RecentScreenMessage::BackToMainMenu),
            KeyCode::Up => Some(RecentScreenMessage::ResultListMoveUp),
            KeyCode::Down => Some(RecentScreenMessage::ResultListMoveDown),
            KeyCode::Enter => {
                if let Some(idx) = self.selected_zettel {
                    let zettel = self.recent_zettels[idx].clone();
                    Some(RecentScreenMessage::IterateZettel(zettel))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    async fn update(&mut self, message: RecentScreenMessage) -> Result<(), Box<dyn Error>> {
        match message {
            RecentScreenMessage::ResultListMoveUp => {
                if let Some(idx) = self.selected_zettel {
                    self.selected_zettel = Some(idx.saturating_sub(1));
                }
            }
            RecentScreenMessage::ResultListMoveDown => {
                if let Some(idx) = self.selected_zettel {
                    if idx + 1 < self.recent_zettels.len() {
                        self.selected_zettel = Some(idx + 1);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Screen for RecentScreen {
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<AppCommand>, Box<dyn Error>> {
        if let Some(msg) = self.handle_key_event_internal(key) {
            match msg {
                RecentScreenMessage::BackToMainMenu => {
                    Ok(Some(AppCommand::SwitchScreen(ActiveScreenType::Main(
                        MainMenuScreen::new(self.db_path.clone(), self.llm_config.clone()),
                    ))))
                }
                RecentScreenMessage::IterateZettel(zettel) => {
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
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        let recent_zettels: Vec<ListItem> = self
            .recent_zettels
            .iter()
            .enumerate()
            .map(|(i, zettel)| {
                let mut item = ListItem::from(zettel);
                if let Some(idx) = self.selected_zettel {
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

        let search_results_list = List::new(recent_zettels).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Recent Zettels"),
        );

        let preview_paragraph = match self.selected_zettel {
            Some(idx) => {
                let selected_zettel = &self.recent_zettels[idx];
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

        f.render_widget(search_results_list, layout[0]);
        f.render_widget(preview, layout[1]);
    }
}
