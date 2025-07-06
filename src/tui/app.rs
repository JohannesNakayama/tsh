use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem},
};
use std::error::Error;

use crate::{
    add_zettel,
    llm::LlmClient,
    tui::{self, search::SearchFeature},
};

pub enum Feature {
    EnterZettel,
    SearchZettels,
}

pub struct MainMenu {
    exit: bool,
    llm_client: LlmClient,
    features: Vec<Feature>,
    selected_feature: Option<usize>,
    terminal: DefaultTerminal,
}

impl MainMenu {
    pub fn new(llm_client: LlmClient) -> Self {
        let exit = false;
        let features = vec![Feature::EnterZettel, Feature::SearchZettels];
        let selected_feature = Some(0);
        let terminal = ratatui::init();
        Self {
            exit: exit,
            llm_client: llm_client,
            features: features,
            selected_feature: selected_feature,
            terminal: terminal,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        // The application's main loop
        loop {
            // draw frame
            let selected_feature = self.selected_feature;
            self.terminal.draw(|f| view(f, selected_feature))?;

            // handle events
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Down => {
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
                    KeyCode::Up => {
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
                    // Execute feature
                    KeyCode::Enter => {
                        if let Some(selected_feature) = self.selected_feature {
                            let feature = &self.features[selected_feature];
                            match feature {
                                Feature::EnterZettel => {
                                    add_zettel(&mut self.llm_client, &vec![]).await?;
                                    let selected_feature = self.selected_feature;
                                    self.terminal.draw(|f| view(f, selected_feature))?;
                                    ratatui::restore();
                                    self.terminal = ratatui::init();
                                }
                                Feature::SearchZettels => {
                                    let mut search_model = SearchFeature::default();
                                    tui::search::run(&mut search_model)?;
                                    ratatui::restore();
                                    self.terminal = ratatui::init();
                                }
                            };
                        }
                    }
                    _ => {}
                }
            }

            if self.exit {
                ratatui::restore();
                break;
            }
        }
        Ok(())
    }
}

fn view(frame: &mut Frame, selected_feature: Option<usize>) {
    let main_menu_layout =
        Layout::new(Direction::Vertical, [Constraint::Length(4)]).split(frame.area());

    let menu_items: Vec<ListItem> = vec![Feature::EnterZettel, Feature::SearchZettels]
        .iter()
        .enumerate()
        .map(|(i, feature)| {
            let menu_entry = match feature {
                Feature::EnterZettel => "Enter Zettel",
                Feature::SearchZettels => "Search Zettels",
            };
            let mut menu_item = ListItem::new(menu_entry);
            if Some(i) == selected_feature {
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

    frame.render_widget(menu, main_menu_layout[0]);
}
