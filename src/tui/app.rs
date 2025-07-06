use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
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

enum Feature {
    EnterZettel,
    SearchZettels,
}

pub struct App {
    exit: bool,
    llm_client: LlmClient,
    features: Vec<Feature>,
    selected_feature: Option<usize>,
    activated_feature: Option<Feature>,
}

impl App {
    pub fn new(llm_client: LlmClient) -> Self {
        let exit = false;
        let features = vec![Feature::EnterZettel, Feature::SearchZettels];
        let selected_feature = Some(0);
        Self {
            exit: exit,
            llm_client: llm_client,
            features: features,
            selected_feature: selected_feature,
            activated_feature: None,
        }
    }
}

pub async fn run(app: &mut App) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| view(f, app.selected_feature))?;

        if let Some(msg) = handle_event(app)? {
            update(app, msg).await;
        }

        if let Some(feature) = &app.activated_feature {
            match feature {
                Feature::EnterZettel => {
                    add_zettel(&mut app.llm_client, &vec![]).await?;
                    app.activated_feature = None;
                    ratatui::restore();
                    terminal = ratatui::init();
                }
                Feature::SearchZettels => {
                    let mut search_model = SearchFeature::default();
                    app.activated_feature = None;
                    tui::search::run(&mut search_model)?;
                    ratatui::restore();
                    terminal = ratatui::init();
                }
            }
        }

        if app.exit {
            ratatui::restore();
            break;
        }
    }

    Ok(())
}

enum Message {
    QuitApp,
    MoveDown,
    MoveUp,
    EnterFeature(Feature),
}

async fn update(model: &mut App, msg: Message) {
    match msg {
        Message::QuitApp => {
            model.exit = true;
        }
        Message::MoveDown => {
            model.selected_feature = match model.selected_feature {
                Some(feature_idx) => {
                    if feature_idx == (model.features.len() - 1) {
                        Some(feature_idx)
                    } else {
                        Some(feature_idx + 1)
                    }
                }
                None => Some(0),
            };
        }
        Message::MoveUp => {
            model.selected_feature = match model.selected_feature {
                Some(feature_idx) => {
                    if feature_idx == 0 {
                        Some(feature_idx)
                    } else {
                        Some(feature_idx - 1)
                    }
                }
                None => Some(model.features.len() - 1),
            };
        }
        Message::EnterFeature(feature) => {
            model.activated_feature = Some(feature);
        }
    }
}

fn handle_event(model: &mut App) -> color_eyre::Result<Option<Message>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, &model.selected_feature));
            }
        }
    }

    Ok(None)
}

fn handle_key(key: KeyEvent, selected_feature: &Option<usize>) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::QuitApp),
        KeyCode::Down => Some(Message::MoveDown),
        KeyCode::Up => Some(Message::MoveUp),
        KeyCode::Enter => {
            if let Some(feature_idx) = selected_feature {
                let feature = match feature_idx {
                    // TODO: refactor this
                    0 => Feature::EnterZettel,
                    1 => Feature::SearchZettels,
                    _ => return None,
                };
                Some(Message::EnterFeature(feature))
            } else {
                None
            }
        }
        _ => None,
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
