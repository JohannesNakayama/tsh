use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem},
};

use crate::tui::app::{AppCommand, Screen};

pub struct MainMenuScreen {
    features: Vec<Feature>,
    selected_feature: Option<usize>,
    // activated_feature: Option<Feature>,
}

enum Feature {
    EnterZettel,
    SearchZettels,
    DevelopZettel,
}

pub enum MainMenuMessage {
    QuitApp,
    MoveDown,
    MoveUp,
    // EnterFeature(Feature),
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self {
            features: vec![
                Feature::EnterZettel,
                Feature::SearchZettels,
                Feature::DevelopZettel,
            ],
            selected_feature: Some(0),
            // activated_feature: None,
        }
    }

    fn handle_key_event_internal(&self, key: KeyEvent) -> Option<MainMenuMessage> {
        match key.code {
            KeyCode::Down => Some(MainMenuMessage::MoveDown),
            KeyCode::Up => Some(MainMenuMessage::MoveUp),
            KeyCode::Char('q') => Some(MainMenuMessage::QuitApp),
            _ => None,
        }
    }

    fn update(&mut self, msg: MainMenuMessage) {
        match msg {
            MainMenuMessage::MoveDown => {
                self.selected_feature = match self.selected_feature {
                    Some(feature_idx) => {
                        if feature_idx == (self.features.len() - 1) {
                            Some(feature_idx)
                        } else {
                            Some(feature_idx + 1)
                        }
                    }
                    None => Some(0),
                };
            }
            MainMenuMessage::MoveUp => {
                self.selected_feature = match self.selected_feature {
                    Some(feature_idx) => {
                        if feature_idx == 0 {
                            Some(feature_idx)
                        } else {
                            Some(feature_idx - 1)
                        }
                    }
                    None => Some(self.features.len() - 1),
                };
            }
            // MainMenuMessage::EnterFeature(feature) => {
            //     model.activated_feature = Some(feature);
            // }
            _ => {}
        }

        // TODO:
        // FOR REFERENCE
        // if let Some(feature) = &app.activated_feature {
        //     ratatui::restore();
        //     match feature {
        //         Feature::EnterZettel => {
        //             // Open an empty Zettel in neovim buffer, no need for new screen
        //             add_zettel(&mut app.llm_client, &vec![]).await?;
        //         },
        //         Feature::SearchZettels => {
        //             let mut search_model = SearchFeature::new(app.llm_client.clone());
        //             tui::search::run(&mut search_model).await?;
        //         },
        //         Feature::DevelopZettel => {
        //             let mut develop_model = DevelopFeature::new();
        //             tui::develop::run(&mut develop_model)?;
        //         },
        //     };
        //     app.activated_feature = None;
        //     terminal = ratatui::init();
        // }
    }
}

impl Screen for MainMenuScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<AppCommand> {
        if let Some(msg) = self.handle_key_event_internal(key) {
            match msg {
                MainMenuMessage::QuitApp => Some(AppCommand::Quit),
                _ => {
                    self.update(msg);
                    None
                }
            }
        } else {
            None
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let layout = Layout::new(Direction::Vertical, [Constraint::Length(4)]).split(f.area());

        // TODO: refactor
        let menu_items: Vec<ListItem> = vec![
            Feature::EnterZettel,
            Feature::SearchZettels,
            Feature::DevelopZettel,
        ]
        .iter()
        .enumerate()
        .map(|(i, feature)| {
            let menu_entry = match feature {
                Feature::EnterZettel => "Enter Zettel",
                Feature::SearchZettels => "Search Zettels",
                Feature::DevelopZettel => "Develop Zettel",
            };
            let mut menu_item = ListItem::new(menu_entry);
            if Some(i) == self.selected_feature {
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
