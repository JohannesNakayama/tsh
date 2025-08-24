use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::error::Error;

use crate::{
    api::{
        add_tag_to_zettel, delete_tag_from_zettel, find_tags, get_n_recent_zettels, get_tags,
        get_zettels_by_tags,
    },
    model::{Zettel, ZettelTag},
    tui::{
        app::{ActiveScreenType, AppCommand, LlmConfig, Screen},
        common::{InputMode, ListWithState},
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
    zettels: ListWithState<Zettel>,
    tag_view_state: Option<TagViewState>,
    tag_search_view_state: Option<TagSearchViewState>,
}

struct TagViewState {
    zettel_id: i64,
    tags: ListWithState<ZettelTag>,
    input_mode: InputMode,
    input: String,
}

struct TagSearchViewState {
    tag_search_results: ListWithState<String>,
    selected_tags: Vec<String>,
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
    SubmitTagSearchQuery,
    TagSearchResultListMoveUp,
    TagSearchResultListMoveDown,
    TagSearchResultAddToSelected,
    SubmitSelectedTagsForFiltering,
}

impl RecentScreen {
    pub async fn new(db_path: String, llm_config: LlmConfig) -> Result<Self, Box<dyn Error>> {
        let recent_zettels = get_n_recent_zettels(&db_path, 100).await?;
        Ok(Self {
            db_path,
            llm_config,
            view: View::ListView,
            zettels: ListWithState::new(recent_zettels),
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
                    if let Some(idx) = self.zettels.curr_idx() {
                        let zettel = self.zettels.items[idx].clone();
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
                        KeyCode::Up => Some(RecentScreenMessage::TagSearchResultListMoveUp),
                        KeyCode::Down => Some(RecentScreenMessage::TagSearchResultListMoveDown),
                        KeyCode::Right => Some(RecentScreenMessage::TagSearchResultAddToSelected),
                        KeyCode::Enter => Some(RecentScreenMessage::SubmitSelectedTagsForFiltering),
                        _ => None,
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Esc => Some(RecentScreenMessage::ExitTagSearchInsertMode),
                        KeyCode::Char(c) => Some(RecentScreenMessage::InsertTagSearchInputChar(c)),
                        KeyCode::Backspace => Some(RecentScreenMessage::DeleteTagSearchInputChar),
                        KeyCode::Enter => Some(RecentScreenMessage::SubmitTagSearchQuery),
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
                    if let Some(idx) = self.zettels.curr_idx() {
                        self.view = View::TagView;
                        let zettel_id = self.zettels.items[idx].id;
                        let tags = get_tags(&self.db_path, zettel_id).await?;
                        self.tag_view_state = Some(TagViewState {
                            zettel_id,
                            tags: ListWithState::new(tags),
                            input: String::new(),
                            input_mode: InputMode::Normal,
                        });
                    }
                }
                View::TagSearchView => {
                    self.tag_view_state = None;
                    self.tag_search_view_state = Some(TagSearchViewState {
                        tag_search_results: ListWithState::new(vec![]),
                        selected_tags: vec![],
                        input_mode: InputMode::Normal,
                        input: String::new(),
                    });
                    self.view = View::TagSearchView;
                }
            },
            RecentScreenMessage::EnterTagInputInsertMode => {
                if let View::TagView = self.view {
                    if let Some(state) = &mut self.tag_view_state {
                        state.tags.unselect();
                        state.input.clear();
                        state.input_mode = InputMode::Insert;
                    }
                }
            }
            RecentScreenMessage::ExitTagInputInsertMode => {
                if let Some(state) = &mut self.tag_view_state {
                    state.tags.select_first();
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
                    let upd_tags = get_tags(&self.db_path, state.zettel_id).await?;
                    state.tags = ListWithState::new(upd_tags);
                    state.input = String::new();
                    state.input_mode = InputMode::Normal;
                }
            }
            RecentScreenMessage::DeleteTag => {
                if let Some(state) = &mut self.tag_view_state {
                    if let Some(zettel_tag) = state.tags.get_selected_item() {
                        delete_tag_from_zettel(
                            &self.db_path,
                            zettel_tag.zettel_id,
                            &zettel_tag.tag,
                        )
                        .await?;
                        let upd_tags = get_tags(&self.db_path, zettel_tag.zettel_id).await?;
                        state.tags = ListWithState::new(upd_tags);
                    }
                }
            }
            RecentScreenMessage::TagListMoveUp => {
                if let Some(state) = &mut self.tag_view_state {
                    state.tags.select_prev();
                }
            }
            RecentScreenMessage::TagListMoveDown => {
                if let Some(state) = &mut self.tag_view_state {
                    state.tags.select_next();
                }
            }
            RecentScreenMessage::ResultListMoveUp => {
                self.zettels.select_prev();
            }
            RecentScreenMessage::ResultListMoveDown => {
                self.zettels.select_next();
            }
            RecentScreenMessage::EnterTagSearchInsertMode => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.tag_search_results.clear_items();
                    state.input_mode = InputMode::Insert;
                    state.input = String::new();
                }
            }
            RecentScreenMessage::ExitTagSearchInsertMode => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.input = String::new();
                    state.input_mode = InputMode::Normal;
                    state.tag_search_results.select_first();
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
            RecentScreenMessage::SubmitTagSearchQuery => {
                if let Some(state) = &mut self.tag_search_view_state {
                    if !state.input.is_empty() {
                        let search_results = find_tags(&self.db_path, &state.input).await?;
                        state.tag_search_results = ListWithState::new(search_results);
                    }
                    state.input_mode = InputMode::Normal;
                }
            }
            RecentScreenMessage::TagSearchResultListMoveUp => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.tag_search_results.select_prev();
                }
            }
            RecentScreenMessage::TagSearchResultListMoveDown => {
                if let Some(state) = &mut self.tag_search_view_state {
                    state.tag_search_results.select_next();
                }
            }
            RecentScreenMessage::TagSearchResultAddToSelected => {
                if let Some(state) = &mut self.tag_search_view_state {
                    if let Some(selected_tag) = state.tag_search_results.get_selected_item() {
                        if !state.selected_tags.contains(&selected_tag) {
                            state.selected_tags.push(selected_tag);
                        }
                    }
                }
            }
            RecentScreenMessage::SubmitSelectedTagsForFiltering => {
                if let Some(state) = &mut self.tag_search_view_state {
                    let zettels_by_tag =
                        get_zettels_by_tags(&self.db_path, state.selected_tags.clone()).await?;
                    self.zettels = ListWithState::new(zettels_by_tag);
                    self.tag_view_state = None;
                    self.tag_search_view_state = None;
                    self.view = View::ListView;
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

        let zettels_list_items: Vec<ListItem> = self
            .zettels
            .items
            .clone()
            .iter()
            .enumerate()
            .map(|(i, zettel)| {
                let mut item = ListItem::from(zettel);
                if let Some(idx) = self.zettels.curr_idx() {
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

        let zettels_list = List::new(zettels_list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title("Zettels"),
        );

        let preview = if let Some(zettel) = self.zettels.get_selected_item() {
            Paragraph::new(zettel.content.to_string())
        } else {
            Paragraph::default()
        }
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title("Preview"),
        );

        f.render_stateful_widget(zettels_list, layout[0], &mut self.zettels.list_state);
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
    let area = popup_area(f.area(), 60, 40);

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::default().add_modifier(Modifier::BOLD))
        .title("Tag");

    let inner_area = block.inner(area);

    let layout = Layout::default()
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

    let zettel_tags_list_items: Vec<ListItem> = state
        .tags
        .items
        .clone()
        .iter()
        .enumerate()
        .map(|(i, zettel_tag)| {
            let mut item = ListItem::from(zettel_tag);
            if let Some(idx) = state.tags.curr_idx() {
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

    let zettel_tags_list = List::new(zettel_tags_list_items);

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(input_field, layout[0]);
    f.render_widget(zettel_tags_list, layout[1]);
}

fn render_tag_search_view(f: &mut Frame, state: &mut TagSearchViewState) {
    let area = popup_area(f.area(), 60, 40);

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::default().add_modifier(Modifier::BOLD))
        .title("Tag Search");

    let inner_area = block.inner(area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let input_field = Paragraph::new(format!("> {}", state.input)).style(match state.input_mode {
        InputMode::Insert => Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        InputMode::Normal => Style::default().bg(Color::DarkGray),
    });

    let tag_search_results_list_items: Vec<ListItem> = state
        .tag_search_results
        .items
        .clone()
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let line = Line::styled(format!("#{}", tag), Style::default());
            let mut item = ListItem::new(line);
            if let Some(idx) = state.tag_search_results.curr_idx() {
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

    let tag_search_results_list = List::new(tag_search_results_list_items);

    let selected_tags_list_items: Vec<ListItem> = state
        .selected_tags
        .iter()
        .map(|tag| {
            let line = Line::styled(
                format!("#{}", tag),
                Style::default().add_modifier(Modifier::ITALIC),
            );
            ListItem::new(line)
        })
        .collect();

    let selected_tags_list = List::new(selected_tags_list_items);

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(input_field, layout[0]);
    f.render_widget(tag_search_results_list, inner_layout[0]);
    f.render_widget(selected_tags_list, inner_layout[1]);
}

// https://ratatui.rs/examples/apps/popup/
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
