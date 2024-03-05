use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Domain {
    pub id: i32,
    pub host: String,
}
