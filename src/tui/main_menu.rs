use std::error::Error;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::tui::{
    app::{ActiveScreenType, AppCommand, LlmConfig, Screen},
    iterate::IterateZettelScreen,
    recent::RecentScreen,
};

pub struct MainMenuScreen {
    selected_action: Action,
    db_path: String,
    llm_config: LlmConfig,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq, Eq)]
enum Action {
    #[default]
    #[strum(to_string = "Add")]
    AddZettel,
    #[strum(to_string = "Iterate")]
    IterateZettel,
    #[strum(to_string = "Recent")]
    RecentZettel,
}

impl Action {
    pub fn previous(self) -> Self {
        let current_idx: usize = self as usize;
        let previous_idx = current_idx.saturating_sub(1);
        Self::from_repr(previous_idx).unwrap_or(self)
    }

    pub fn next(self) -> Self {
        let current_idx: usize = self as usize;
        let next_idx = current_idx.saturating_add(1);
        Self::from_repr(next_idx).unwrap_or(self)
    }
}

enum MainMenuMessage {
    QuitApp,
    MoveDown,
    MoveUp,
    DoAction(Action),
}

impl MainMenuScreen {
    pub fn new(db_path: String, llm_config: LlmConfig) -> Self {
        Self {
            selected_action: Action::AddZettel,
            db_path,
            llm_config,
        }
    }

    fn handle_key_event_internal(&self, key: KeyEvent) -> Option<MainMenuMessage> {
        match key.code {
            KeyCode::Down => Some(MainMenuMessage::MoveDown),
            KeyCode::Up => Some(MainMenuMessage::MoveUp),
            KeyCode::Char('q') | KeyCode::Esc => Some(MainMenuMessage::QuitApp),
            KeyCode::Enter => Some(MainMenuMessage::DoAction(self.selected_action)),
            _ => None,
        }
    }

    async fn update(&mut self, msg: MainMenuMessage) -> Result<(), Box<dyn Error>> {
        match msg {
            MainMenuMessage::MoveDown => {
                self.selected_action = self.selected_action.next();
            }
            MainMenuMessage::MoveUp => {
                self.selected_action = self.selected_action.previous();
            }
            _ => {}
        };
        Ok(())
    }
}

impl Screen for MainMenuScreen {
    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<AppCommand>, Box<dyn Error>> {
        if let Some(msg) = self.handle_key_event_internal(key) {
            match msg {
                MainMenuMessage::QuitApp => Ok(Some(AppCommand::Quit)),
                MainMenuMessage::DoAction(action) => match action {
                    Action::AddZettel => Ok(Some(AppCommand::AddZettel(vec![]))),
                    Action::IterateZettel => {
                        Ok(Some(AppCommand::SwitchScreen(ActiveScreenType::Iterate(
                            IterateZettelScreen::new(self.db_path.clone(), self.llm_config.clone()),
                        ))))
                    }
                    Action::RecentZettel => {
                        Ok(Some(AppCommand::SwitchScreen(ActiveScreenType::Recent(
                            RecentScreen::new(self.db_path.clone(), self.llm_config.clone())
                                .await?,
                        ))))
                    }
                },
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
        let layout = Layout::new(Direction::Vertical, [Constraint::Length(4)]).split(f.area());

        let menu_items: Vec<ListItem> = Action::iter()
            .map(|action| {
                let menu_entry = format!("{action}");
                let mut menu_item = ListItem::new(menu_entry);
                if action == self.selected_action {
                    menu_item = menu_item.style(
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                menu_item
            })
            .collect();

        let menu = List::new(menu_items);

        f.render_widget(menu, layout[0]);
    }
}
