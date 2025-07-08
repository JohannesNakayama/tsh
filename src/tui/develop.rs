use std::{error::Error, time::Duration};

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    widgets::{List, ListItem},
};

pub struct DevelopFeature {
    exit: bool,
}

impl DevelopFeature {
    pub fn new() -> Self {
        DevelopFeature { exit: false }
    }
}

enum Message {
    ExitDevelopFeature,
}

pub fn run(model: &mut DevelopFeature) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| view(f, model))?;

        if let Some(message) = handle_event(model)? {
            update(model, message);
        }

        if model.exit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}

pub fn view(frame: &mut Frame, _: &DevelopFeature) {
    let develop_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(frame.area());

    let options = vec![
        ListItem::new("Option 1"),
        ListItem::new("Option 2"),
        ListItem::new("Option 3"),
    ];

    let menu = List::new(options);

    frame.render_widget(menu, develop_layout[0]);
}

fn update(model: &mut DevelopFeature, msg: Message) {
    match msg {
        Message::ExitDevelopFeature => {
            model.exit = true;
        }
    }
}

fn handle_event(_: &mut DevelopFeature) -> Result<Option<Message>, Box<dyn Error>> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }

    Ok(None)
}

fn handle_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::ExitDevelopFeature),
        _ => None,
    }
}
