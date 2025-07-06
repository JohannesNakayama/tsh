use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::{error::Error, time::Duration};

enum InputMode {
    Insert,
    Normal,
}

pub struct SearchFeature {
    input: String,
    input_mode: InputMode,
    exit: bool,
}

impl SearchFeature {
    pub fn default() -> Self {
        SearchFeature {
            input: String::new(),
            input_mode: InputMode::Normal,
            exit: false,
        }
    }
}

pub fn run(model: &mut SearchFeature) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| view(f, &model))?;

        if let Some(message) = handle_event(model)? {
            update(model, message);
        }

        if model.exit {
            break;
        }
    }

    Ok(())
}

enum Message {
    ExitSearchFeature,
    EnterInsertMode,
    ExitInsertMode,
    InsertCharacter(char),
    DeleteCharacter,
}

fn update(model: &mut SearchFeature, msg: Message) {
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
    };
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
            KeyCode::Esc => Some(Message::ExitSearchFeature),
            _ => None,
        },
        InputMode::Insert => match key.code {
            KeyCode::Esc => Some(Message::ExitInsertMode),
            KeyCode::Char(c) => Some(Message::InsertCharacter(c)),
            KeyCode::Backspace => Some(Message::DeleteCharacter),
            _ => None,
        },
    }
}

fn view(frame: &mut Frame, model: &SearchFeature) {
    let search_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(frame.area());

    let search_input = Paragraph::new(model.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Input"),
        );

    frame.render_widget(search_input, search_layout[0]);
}
