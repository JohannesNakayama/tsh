use ratatui::{
    crossterm::event::{self, Event, KeyCode}, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{List, ListItem}, DefaultTerminal, Frame
};
use tsh::{add_zettel, llm::LlmClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: load from config, not env
    // let db_url = std::env::var("DATABASE_URL")?;
    let api_base = std::env::var("API_BASE")?;
    let api_key = std::env::var("API_KEY")?;
    let embedding_model = std::env::var("EMBEDDINGS_MODEL")?;
    let chat_model = std::env::var("CHAT_MODEL")?;

    // TODO: is it a good idea to run this every time?
    // migrate_to_latest(&db_url).await?;

    let llm_client = LlmClient::new(api_base, api_key, embedding_model, chat_model);

    // let mut conn = get_db(&db_url).await?;
    // let tx = conn.transaction()?;
    // let parent = find_zettel_by_id(&tx, 1).await?;
    // tx.commit()?;
    // add_zettel(&mut llm_client, &vec![]).await?;
    // chat().await?;
    // add_combined_zettel(&mut llm_client).await?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let _app_result = App::new(llm_client).run(terminal).await?;
    ratatui::restore();

    Ok(())
}


pub enum Feature {
    EnterZettel,
    SearchZettels,
}


pub struct App {
    exit: bool,
    llm_client: LlmClient,
    features: Vec<Feature>,
    selected_feature: Option<usize>,
}


impl App {
    fn new(llm_client: LlmClient) -> Self {
        let exit = false;
        let features = vec![Feature::EnterZettel, Feature::SearchZettels];
        let selected_feature = None;
        Self {
            exit,
            llm_client,
            features,
            selected_feature
        }
    }

    async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
        // The application's main loop
        loop {
            // draw frame
            terminal.draw(|f| self.draw(f))?;

            // handle events
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    },
                    KeyCode::Down => {
                        self.selected_feature = match self.selected_feature {
                            Some(feature_idx) => if feature_idx == (self.features.len() - 1) {
                                Some(feature_idx)
                            } else {
                                Some(feature_idx + 1)
                            },
                            None => Some(0),
                        };
                    },
                    KeyCode::Up => {
                        self.selected_feature = match self.selected_feature {
                            Some(feature_idx) => if feature_idx == 0 {
                                Some(feature_idx)
                            } else {
                                Some(feature_idx - 1)
                            },
                            None => Some(self.features.len() - 1),
                        };
                    },
                    // Execute feature
                    KeyCode::Enter => {
                        if let Some(selected_feature) = self.selected_feature {
                            let feature = &self.features[selected_feature];
                            match feature {
                                Feature::EnterZettel => {
                                    add_zettel(&mut self.llm_client, &vec![]).await?;
                                    terminal.draw(|f| self.draw(f))?; // redraw ui after vim buffer
                                                                      // exits
                                },
                                _ => {},
                            };
                        }
                    }
                    _ => {},
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let main_menu_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(4),
                Constraint::Min(0),
            ],
        )
            .split(frame.area());

        let menu_items: Vec<ListItem> = vec![Feature::EnterZettel, Feature::SearchZettels]
            .iter()
            .enumerate()
            .map(|(i, feature)| {
                let menu_entry = match feature {
                    Feature::EnterZettel => "Enter Zettel",
                    Feature::SearchZettels => "Search Zettels",
                };
                let mut menu_item = ListItem::new(menu_entry);
                if Some(i) == self.selected_feature {
                    menu_item = menu_item.style(Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD));
                }
                menu_item
            })
            .collect();

        let menu = List::new(menu_items);

        // let enter_zettel_menu_item = Paragraph::new("Enter Zettel")
        //     .style(Style::default().fg(Color::Blue))
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_type(BorderType::Rounded)
        //     );

        // let search_zettels_menu_item = Paragraph::new("Search Zettels")
        //     .style(Style::default().fg(Color::Red))
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_type(BorderType::Rounded)
        //     );

        // let remix_zettels_menu_item = Paragraph::new("Remix Zettels")
        //     .style(Style::default().fg(Color::Green))
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_type(BorderType::Rounded)
        //     );

        frame.render_widget(menu, main_menu_layout[0]);
        // frame.render_widget(search_zettels_menu_item, main_menu_layout[1]);
        // frame.render_widget(remix_zettels_menu_item, main_menu_layout[2]);
    }
}
