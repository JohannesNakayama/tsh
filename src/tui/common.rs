use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::ListItem,
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
