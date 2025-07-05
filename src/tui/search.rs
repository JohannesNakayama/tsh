use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    style::{Style, Color},
    widgets::{Block, BorderType, Borders, Paragraph}, DefaultTerminal, Frame};
use std::error::Error;

pub enum InputMode {
    Insert,
    Normal,
}


pub struct SearchFeatureModel {
    input: String,
    input_mode: InputMode,
    terminal: DefaultTerminal,
}

impl SearchFeatureModel {
    pub fn default() -> Self {
        let terminal = ratatui::init();
        SearchFeatureModel {
            input: String::new(),
            input_mode: InputMode::Normal,
            terminal: terminal
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.terminal.draw(|f| draw_search_page(f, &self.input))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('i') => {
                            self.input_mode = InputMode::Insert;
                        },
                        KeyCode::Esc => {
                            break;
                        },
                        _ => {},
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Normal;
                        },
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        },
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        _ => {},
                    },
                }
            }
        }

        Ok(())
    }
}


pub fn draw_search_page(frame: &mut Frame, search_input: &str) {
    let search_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3),
            Constraint::Min(0),
        ],
    )
        .split(frame.area());

    let search_input = Paragraph::new(search_input)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Input")
        );

    frame.render_widget(search_input, search_layout[0]);
}

