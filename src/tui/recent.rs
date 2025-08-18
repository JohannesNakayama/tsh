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
        common::InputMode,
        main_menu::MainMenuScreen,
    },
};

enum View {
    ListView,
    TagView,
    TagSearchView,
}

pub struct RecentScreen {
    db_path: String,
    llm_config: LlmConfig,
    view: View,
    list_view_state: ListViewState,
    tag_view_state: Option<TagViewState>,
    tag_search_view_state: Option<TagSearchViewState>,
}

struct ListViewState {
    zettels: Vec<Zettel>,
    selected_idx: Option<usize>,
    display_state: ListState,
}

struct TagViewState {
    zettel_id: i64,
    tags: Vec<ZettelTag>,
    selected_idx: Option<usize>,
    input_mode: InputMode,
    input: String,
}

struct TagSearchViewState {
    input_mode: InputMode,
    input: String,
}

enum RecentScreenMessage {
    SwitchView(View),
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
    EnterTagSearchInsertMode,
    ExitTagSearchInsertMode,
    InsertTagSearchInputChar(char),
    DeleteTagSearchInputChar,
}

impl RecentScreen {
    pub async fn new(db_path: String, llm_config: LlmConfig) -> Result<Self, Box<dyn Error>> {
        let n_recent_zettels = get_n_recent_zettels(&db_path, 100).await?;
        Ok(Self {
            db_path,
            llm_config,
            view: View::ListView,
            list_view_state: ListViewState {
                zettels: n_recent_zettels.clone(),
                selected_idx: if n_recent_zettels.is_empty() {
                    None
                } else {
                    Some(0)
                },
                display_state: ListState::default(),
            },
            tag_view_state: None,
            tag_search_view_state: None,
        })
    }

    fn handle_key_event_internal(&mut self, key: KeyEvent) -> Option<RecentScreenMessage> {
        match self.view {
            View::ListView => match key.code {
                KeyCode::Char('q') => Some(RecentScreenMessage::BackToMainMenu),
                KeyCode::Char('t') => Some(RecentScreenMessage::SwitchView(View::TagView)),
                KeyCode::Char('s') => Some(RecentScreenMessage::SwitchView(View::TagSearchView)),
                KeyCode::Up => Some(RecentScreenMessage::ResultListMoveUp),
                KeyCode::Down => Some(RecentScreenMessage::ResultListMoveDown),
                KeyCode::Enter => {
                    if let Some(idx) = self.list_view_state.selected_idx {
                        let zettel = self.list_view_state.zettels[idx].clone();
                        Some(RecentScreenMessage::IterateZettel(zettel))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            View::TagView => match &self.tag_view_state {
                Some(state) => match state.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => Some(RecentScreenMessage::SwitchView(View::ListView)),
                        KeyCode::Char('i') => Some(RecentScreenMessage::EnterTagInputInsertMode),
                        KeyCode::Up => Some(RecentScreenMessage::TagListMoveUp),
                        KeyCode::Down => Some(RecentScreenMessage::TagListMoveDown),
                        KeyCode::Char('d') => Some(RecentScreenMessage::DeleteTag),
                        _ => None,
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Char(c) => Some(RecentScreenMessage::InsertTagInputChar(c)),
                        KeyCode::Backspace => Some(RecentScreenMessage::DeleteTagInputChar),
                        KeyCode::Enter => Some(RecentScreenMessage::SubmitTag),
                        KeyCode::Esc => Some(RecentScreenMessage::ExitTagInputInsertMode),
                        _ => None,
                    },
                },
                None => None,
            },
            View::TagSearchView => match &self.tag_search_view_state {
                Some(state) => match state.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => Some(RecentScreenMessage::SwitchView(View::ListView)),
                        KeyCode::Char('i') => Some(RecentScreenMessage::EnterTagSearchInsertMode),
                        _ => None,
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Esc => Some(RecentScreenMessage::ExitTagSearchInsertMode),
                        KeyCode::Char(c) => Some(RecentScreenMessage::InsertTagSearchInputChar(c)),
                        KeyCode::Backspace => Some(RecentScreenMessage::DeleteTagSearchInputChar),
                        _ => None,
                    },
                },
                None => None,
            },
        }
    }

    async fn update(&mut self, message: RecentScreenMessage) -> Result<(), Box<dyn Error>> {
        match message {
            RecentScreenMessage::SwitchView(view) => match view {
                View::ListView => {
                    self.tag_view_state = None;
                    self.tag_search_view_state = None;
                    self.view = View::ListView;
                }
                View::TagView => {
                    if let Some(idx) = self.list_view_state.selected_idx {
                        self.view = View::TagView;
                        let zettel_id = self.list_view_state.zettels[idx].id;
                        let tags = get_tags(&self.db_path, zettel_id).await?;
                        let selected_idx = if tags.len() > 0 { Some(0) } else { None };
                        self.tag_view_state = Some(TagViewState {
                            zettel_id,
                            tags,
                            input: String::new(),
                            input_mode: InputMode::Normal,
                            selected_idx,
                        });
                    }
                }
                View::TagSearchView => {
                    self.tag_view_state = None;
                    self.tag_search_view_state = Some(TagSearchViewState {
                        input_mode: InputMode::Normal,
                        input: String::new(),
                    });
                    self.view = View::TagSearchView;
                }
            },
            RecentScreenMessage::EnterTagInputInsertMode => {
                if let View::TagView = self.view {
                    if let Some(state) = &mut self.tag_view_state {
                        state.selected_idx = None;
                        state.input.clear();
                        state.input_mode = InputMode::Insert;
                    }
                }
            }
            RecentScreenMessage::ExitTagInputInsertMode => {
                if let Some(state) = &mut self.tag_view_state {
                    if state.tags.len() > 0 {
                        state.selected_idx = Some(0);
                    }
                    state.input.clear();
                    state.input_mode = InputMode::Normal;
                }
            }
            RecentScreenMessage::InsertTagInputChar(c) => {
                if let Some(state) = &mut self.tag_view_state {
                    state.input.push(c);
                }
            }
            RecentScreenMessage::DeleteTagInputChar => {
                if let Some(state) = &mut self.tag_view_state {
                    state.input.pop();
                }
            }
            RecentScreenMessage::SubmitTag => {
                if let Some(state) = &mut self.tag_view_state {
                    add_tag_to_zettel(&self.db_path, state.zettel_id, state.input.clone()).await?;
                    state.tags = get_tags(&self.db_path, state.zettel_id).await?;
                    state.input = String::new();
                    state.input_mode = InputMode::Normal;
                    if state.tags.len() > 0 {
                        state.selected_idx = Some(0);
                    }
                }
            }
            RecentScreenMessage::DeleteTag => {
                if let Some(state) = &mut self.tag_view_state {
                    if let Some(idx) = state.selected_idx {
                        let selected_tag = state.tags[idx].clone();
                        delete_tag_from_zettel(
                            &self.db_path,
                            selected_tag.zettel_id,
                            &selected_tag.tag,
                        )
                        .await?;
                        state.tags = get_tags(&self.db_path, selected_tag.zettel_id).await?;
                        if state.tags.len() > 0 {
                            state.selected_idx = Some(0);
                        } else {
                            state.selected_idx = None;
                        }
                    }
                }
            }
            RecentScreenMessage::TagListMoveUp => {
                if let Some(state) = &mut self.tag_view_state {
                    if let Some(idx) = state.selected_idx {
                        state.selected_idx = Some(idx.saturating_sub(1));
                    }
                }
            }
            RecentScreenMessage::TagListMoveDown => {
                if let Some(state) = &mut self.tag_view_state {
                    if let Some(idx) = state.selected_idx {
                        if idx + 1 < state.tags.len() {
                            state.selected_idx = Some(idx + 1);
                        }
                    }
                }
            }
            RecentScreenMessage::ResultListMoveUp => {
                if let Some(idx) = self.list_view_state.selected_idx {
                    self.list_view_state.selected_idx = Some(idx.saturating_sub(1));
                    self.list_view_state
                        .display_state
                        .select(self.list_view_state.selected_idx);
                }
            }
            RecentScreenMessage::ResultListMoveDown => {
                if let Some(idx) = self.list_view_state.selected_idx {
                    if idx + 1 < self.list_view_state.zettels.len() {
                        self.list_view_state.selected_idx = Some(idx + 1);
                        self.list_view_state
                            .display_state
                            .select(self.list_view_state.selected_idx);
                    }
                }
            }
            RecentScreenMessage::EnterTagSearchInsertMode => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.input_mode = InputMode::Insert;
                    state.input = String::new();
                }
            }
            RecentScreenMessage::ExitTagSearchInsertMode => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.input = String::new();
                    state.input_mode = InputMode::Normal;
                }
            }
            RecentScreenMessage::InsertTagSearchInputChar(c) => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.input.push(c);
                }
            }
            RecentScreenMessage::DeleteTagSearchInputChar => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.input.pop();
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
            .list_view_state
            .zettels
            .iter()
            .enumerate()
            .map(|(i, zettel)| {
                let mut item = ListItem::from(zettel);
                if let Some(idx) = self.list_view_state.selected_idx {
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

        let preview_paragraph = match self.list_view_state.selected_idx {
            Some(idx) => {
                let selected_zettel = &self.list_view_state.zettels[idx];
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

        f.render_stateful_widget(
            search_results_list,
            layout[0],
            &mut self.list_view_state.display_state,
        );
        f.render_widget(preview, layout[1]);

        match self.view {
            View::TagView => {
                if let Some(state) = &mut self.tag_view_state {
                    render_tag_view(f, state);
                }
            }
            View::TagSearchView => {
                if let Some(state) = &mut self.tag_search_view_state {
                    render_tag_search_view(f, state);
                }
            }
            _ => {}
        }
    }
}

fn render_tag_view(f: &mut Frame, state: &mut TagViewState) {
    let tag_list_items: Vec<ListItem> = state
        .tags
        .iter()
        .enumerate()
        .map(|(i, zettel_tag)| {
            let mut item = ListItem::from(zettel_tag);
            if let Some(idx) = state.selected_idx {
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

    let input_field = Paragraph::new(format!("> {}", state.input)).style(match state.input_mode {
        InputMode::Insert => Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        InputMode::Normal => Style::default().bg(Color::DarkGray),
    });

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(input_field, popup_layout[0]);
    f.render_widget(tag_list, popup_layout[1]);
}

fn render_tag_search_view(f: &mut Frame, state: &mut TagSearchViewState) {
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::default().add_modifier(Modifier::BOLD))
        .title("Tag Search");

    let area = popup_area(f.area(), 60, 40);

    let input_field = Paragraph::new(format!("> {}", state.input)).style(match state.input_mode {
        InputMode::Insert => Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        InputMode::Normal => Style::default().bg(Color::DarkGray),
    });

    let inner_area = block.inner(area);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(input_field, popup_layout[0]);
    // f.render_widget(tag_list, popup_layout[1]);
}

// https://ratatui.rs/examples/apps/popup/
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
