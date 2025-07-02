#[derive(Debug, Clone)]
pub struct Zettel {
    pub id: i64,
    pub content: String,
    pub created_at: i64, // TODO: look into how to make this u128
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub node_id: i64,
    pub parent_id: Option<i64>,
}
