use chrono::DateTime;

#[derive(Debug, Clone)]
pub struct Zettel {
    pub id: i64,
    pub content: String,
    pub created_at: i64, // TODO: look into how to make this u128
}

impl Zettel {
    pub fn get_shim(&self) -> String {
        if self.content.len() < 77 {
            self.content.to_string()
        } else {
            format!("{}...", self.content[..77].to_string())
        }
    }

    // TODO: refactor once timestamps are retrieved as u128
    // TODO: display in current timezone?
    pub fn get_datetime_string(&self) -> String {
        let timestamp_seconds = self.created_at / 1000;
        let datetime = DateTime::from_timestamp(timestamp_seconds, 0).unwrap();
        datetime.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct ZettelEdge {
    pub node_id: i64,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub id: i64,
    pub zettel_id: i64,
    pub title: String,
    pub content: String,
    pub created_at: i64, // TODO: look into how to make this u128
}
