use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{ListItem, ListState},
};

use crate::model::{Zettel, ZettelTag};

impl From<&Zettel> for ListItem<'_> {
    fn from(zettel: &Zettel) -> Self {
        let lines = vec![
            Line::styled(
                format!("{}: {}", zettel.id, zettel.get_datetime_string()),
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .fg(Color::LightBlue),
            ),
            Line::styled(zettel.get_shim(), Style::default()),
        ];
        ListItem::new(lines)
    }
}

impl From<&ZettelTag> for ListItem<'_> {
    fn from(tag: &ZettelTag) -> Self {
        let line = Line::styled(
            format!("#{}", tag.tag),
            Style::default().add_modifier(Modifier::ITALIC),
        );
        ListItem::new(line)
    }
}

pub enum InputMode {
    Insert,
    Normal,
}

pub struct ListWithState<T> {
    pub items: Vec<T>,
    pub list_state: ListState,
}

impl<T> ListWithState<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>) -> Self {
        let mut state = ListState::default();
        if items.len() > 0 {
            state.select_first();
        }
        ListWithState {
            items,
            list_state: state,
        }
    }

    pub fn get_selected_item(&mut self) -> Option<T> {
        if let Some(idx) = self.list_state.selected() {
            if !self.items.is_empty() && (idx < self.items.len()) {
                Some(self.items[idx].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear_items(&mut self) {
        self.unselect();
        self.items.clear();
    }

    pub fn select_next(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if idx + 1 < self.items.len() {
                self.list_state.select_next();
            }
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(_) = self.list_state.selected() {
            self.list_state.select_previous();
        }
    }

    pub fn select_first(&mut self) {
        if self.items.len() > 0 {
            self.list_state.select_first();
        }
    }

    pub fn unselect(&mut self) {
        self.list_state.select(None);
    }

    pub fn curr_idx(&mut self) -> Option<usize> {
        self.list_state.selected()
    }
}
