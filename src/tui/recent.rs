use std::error::Error;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    api::{add_tag_to_zettel, delete_tag_from_zettel, get_n_recent_zettels, get_tags},
    model::{Zettel, ZettelTag},
    tui::{
        app::{ActiveScreenType, AppCommand, LlmConfig, Screen},
        main_menu::MainMenuScreen,
    },
};

enum View {
    ListView,
    TagView,
}

enum TagInputMode {
    Insert,
    Normal,
}

pub struct RecentScreen {
    recent_zettels: Vec<Zettel>,
    selected_zettel: Option<usize>,
    selected_zettel_tags: Vec<ZettelTag>,
    db_path: String,
    llm_config: LlmConfig,
    list_state: ListState,
    view: View,
    tag_input_mode: TagInputMode,
    tag_input: String,
    tag_selected_idx: Option<usize>,
}

enum RecentScreenMessage {
    SwitchToTagView,
    SwitchToListView,
    EnterTagInputInsertMode,
    ExitTagInputInsertMode,
    InsertTagInputChar(char),
    DeleteTagInputChar,
    SubmitTag,
    DeleteTag,
    TagListMoveUp,
    TagListMoveDown,
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
            selected_zettel_tags: vec![],
            db_path,
            llm_config,
            list_state: ListState::default(),
            view: View::ListView,
            tag_input_mode: TagInputMode::Normal,
            tag_input: String::new(),
            tag_selected_idx: None,
        })
    }

    fn handle_key_event_internal(&mut self, key: KeyEvent) -> Option<RecentScreenMessage> {
        match self.view {
            View::ListView => match key.code {
                KeyCode::Char('q') => Some(RecentScreenMessage::BackToMainMenu),
                KeyCode::Char('t') => Some(RecentScreenMessage::SwitchToTagView),
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
            },
            View::TagView => match self.tag_input_mode {
                TagInputMode::Normal => match key.code {
                    KeyCode::Char('q') => Some(RecentScreenMessage::SwitchToListView),
                    KeyCode::Char('i') => Some(RecentScreenMessage::EnterTagInputInsertMode),
                    KeyCode::Up => Some(RecentScreenMessage::TagListMoveUp),
                    KeyCode::Down => Some(RecentScreenMessage::TagListMoveDown),
                    KeyCode::Char('d') => Some(RecentScreenMessage::DeleteTag),
                    _ => None,
                },
                TagInputMode::Insert => match key.code {
                    KeyCode::Char(c) => Some(RecentScreenMessage::InsertTagInputChar(c)),
                    KeyCode::Backspace => Some(RecentScreenMessage::DeleteTagInputChar),
                    KeyCode::Enter => Some(RecentScreenMessage::SubmitTag),
                    KeyCode::Esc => Some(RecentScreenMessage::ExitTagInputInsertMode),
                    _ => None,
                },
            },
        }
    }

    async fn update(&mut self, message: RecentScreenMessage) -> Result<(), Box<dyn Error>> {
        match message {
            RecentScreenMessage::SwitchToTagView => {
                if let Some(idx) = self.selected_zettel {
                    let zettel_id = self.recent_zettels[idx].id;
                    self.selected_zettel_tags = get_tags(&self.db_path, zettel_id).await?;
                    self.tag_input.clear();
                    self.tag_input_mode = TagInputMode::Normal;
                    if let None = self.tag_selected_idx {
                        if self.selected_zettel_tags.len() > 0 {
                            self.tag_selected_idx = Some(0);
                        }
                    }
                    self.view = View::TagView;
                }
            }
            RecentScreenMessage::SwitchToListView => {
                self.tag_input.clear();
                self.tag_input_mode = TagInputMode::Normal;
                self.view = View::ListView;
            }
            RecentScreenMessage::EnterTagInputInsertMode => {
                if let View::TagView = self.view {
                    self.tag_selected_idx = None;
                    self.tag_input.clear();
                    self.tag_input_mode = TagInputMode::Insert;
                }
            }
            RecentScreenMessage::ExitTagInputInsertMode => {
                if let View::TagView = self.view {
                    if let None = self.tag_selected_idx {
                        if self.selected_zettel_tags.len() > 0 {
                            self.tag_selected_idx = Some(0);
                        }
                    }
                    self.tag_input.clear();
                    self.tag_input_mode = TagInputMode::Normal;
                }
            }
            RecentScreenMessage::InsertTagInputChar(c) => {
                self.tag_input.push(c);
            }
            RecentScreenMessage::DeleteTagInputChar => {
                self.tag_input.pop();
            }
            RecentScreenMessage::SubmitTag => {
                if let Some(idx) = self.selected_zettel {
                    let zettel_id = self.recent_zettels[idx].id;
                    add_tag_to_zettel(&self.db_path, zettel_id, self.tag_input.clone()).await?;
                    self.selected_zettel_tags = get_tags(&self.db_path, zettel_id).await?;
                    self.tag_input = String::new();
                    self.tag_input_mode = TagInputMode::Normal;
                    if self.selected_zettel_tags.len() > 0 {
                        self.tag_selected_idx = Some(0);
                    }
                }
            }
            RecentScreenMessage::DeleteTag => {
                if let Some(tag_idx) = self.tag_selected_idx {
                    let selected_tag = self.selected_zettel_tags[tag_idx].clone();
                    delete_tag_from_zettel(
                        &self.db_path,
                        selected_tag.zettel_id,
                        &selected_tag.tag,
                    )
                    .await?;
                    self.selected_zettel_tags =
                        get_tags(&self.db_path, selected_tag.zettel_id).await?;
                    if self.selected_zettel_tags.len() > 0 {
                        self.tag_selected_idx = Some(0);
                    }
                }
            }
            RecentScreenMessage::TagListMoveUp => {
                if let Some(idx) = self.tag_selected_idx {
                    self.tag_selected_idx = Some(idx.saturating_sub(1));
                }
            }
            RecentScreenMessage::TagListMoveDown => {
                if let Some(idx) = self.tag_selected_idx {
                    if idx + 1 < self.selected_zettel_tags.len() {
                        self.tag_selected_idx = Some(idx + 1);
                    }
                }
            }
            RecentScreenMessage::ResultListMoveUp => {
                if let Some(idx) = self.selected_zettel {
                    self.selected_zettel = Some(idx.saturating_sub(1));
                    self.list_state.select(self.selected_zettel);
                }
            }
            RecentScreenMessage::ResultListMoveDown => {
                if let Some(idx) = self.selected_zettel {
                    if idx + 1 < self.recent_zettels.len() {
                        self.selected_zettel = Some(idx + 1);
                        self.list_state.select(self.selected_zettel);
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

        f.render_stateful_widget(search_results_list, layout[0], &mut self.list_state);
        f.render_widget(preview, layout[1]);

        if let View::TagView = self.view {
            let tag_list_items: Vec<ListItem> = self
                .selected_zettel_tags
                .iter()
                .enumerate()
                .map(|(i, zettel_tag)| {
                    let mut item = ListItem::from(zettel_tag);
                    if let Some(idx) = self.tag_selected_idx {
                        if i == idx {
                            item = item.style(
                                Style::default()
                                    .fg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD),
                            );
                        }
                    }
                    item
                })
                .collect();

            let tag_list = List::new(tag_list_items);

            let block = Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::default().add_modifier(Modifier::BOLD))
                .title("Tag");

            let area = popup_area(f.area(), 60, 40);

            let inner_area = block.inner(area);

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(0)])
                .split(inner_area);

            let input_field =
                Paragraph::new(format!("> {}", self.tag_input)).style(match self.tag_input_mode {
                    TagInputMode::Insert => Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                        .fg(Color::LightGreen),
                    TagInputMode::Normal => Style::default().bg(Color::DarkGray),
                });

            f.render_widget(Clear, area);
            f.render_widget(block, area);
            f.render_widget(input_field, popup_layout[0]);
            f.render_widget(tag_list, popup_layout[1]);
        }
    }
}

// https://ratatui.rs/examples/apps/popup/
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
