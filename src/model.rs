#[derive(Debug, Clone)]
pub struct Thought {
    pub id: i64,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub node_id: i64,
    pub parent_id: Option<i64>,
}
