use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};
use std::{error::Error, time::Duration};

use crate::{find_zettels, llm::LlmClient};

pub struct SearchFeature {
    llm_client: LlmClient,
    input: String,
    input_mode: InputMode,
    exit: bool,
    search_results: Vec<String>, // TODO: we also need ids etc. here to later on select and combine
                                 // the search results
}

impl SearchFeature {
    pub fn new(llm_client: LlmClient) -> Self {
        SearchFeature {
            llm_client: llm_client,
            input: String::new(),
            input_mode: InputMode::Normal,
            exit: false,
            search_results: vec![],
        }
    }
}

pub async fn run(model: &mut SearchFeature) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| view(f, &model))?;

        if let Some(message) = handle_event(model)? {
            update(model, message).await?;
        }

        if model.exit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}

enum InputMode {
    Insert,
    Normal,
}

enum Message {
    ExitSearchFeature,
    EnterInsertMode,
    ExitInsertMode,
    InsertCharacter(char),
    DeleteCharacter,
    SubmitQuery,
}

fn view(frame: &mut Frame, model: &SearchFeature) {
    let search_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(frame.area());

    let style = match model.input_mode {
        InputMode::Normal => Style::default().fg(Color::White),
        InputMode::Insert => Style::default().fg(Color::LightGreen),
    };

    let search_input = Paragraph::new(model.input.as_str()).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Input"),
    );

    let search_results = model
        .search_results
        .iter()
        .map(|result| ListItem::new(result.as_str()))
        .collect::<Vec<_>>();

    let search_results_list = List::new(search_results);

    frame.render_widget(search_input, search_layout[0]);
    frame.render_widget(search_results_list, search_layout[1]);
}

async fn update(model: &mut SearchFeature, msg: Message) -> Result<(), Box<dyn Error>> {
    match msg {
        Message::ExitSearchFeature => {
            model.exit = true;
        }
        Message::EnterInsertMode => {
            model.input_mode = InputMode::Insert;
        }
        Message::ExitInsertMode => {
            model.input_mode = InputMode::Normal;
        }
        Message::InsertCharacter(c) => {
            model.input.push(c);
        }
        Message::DeleteCharacter => {
            model.input.pop();
        }
        Message::SubmitQuery => {
            let zettels = find_zettels(&mut model.llm_client, model.input.as_str()).await?;
            model.search_results = zettels
                .iter()
                .map(|zettel| zettel.content.clone())
                .collect();
            model.input.clear();
            model.input_mode = InputMode::Normal;
        }
    };

    Ok(())
}

fn handle_event(model: &mut SearchFeature) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, &model.input_mode));
            }
        }
    }

    Ok(None)
}

fn handle_key(key: KeyEvent, input_mode: &InputMode) -> Option<Message> {
    match input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('i') => Some(Message::EnterInsertMode),
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::ExitSearchFeature),
            KeyCode::Enter => Some(Message::SubmitQuery),
            _ => None,
        },
        InputMode::Insert => match key.code {
            KeyCode::Esc => Some(Message::ExitInsertMode),
            KeyCode::Char(c) => Some(Message::InsertCharacter(c)),
            KeyCode::Backspace => Some(Message::DeleteCharacter),
            KeyCode::Enter => Some(Message::SubmitQuery),
            _ => None,
        },
    }
}
