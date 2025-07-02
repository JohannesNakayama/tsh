#[derive(Debug, Clone)]
pub struct Zettel {
    pub id: i64,
    pub content: String,
    pub created_at: i64, // TODO: look into how to make this u128
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
