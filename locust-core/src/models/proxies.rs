use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Proxy {
    pub id: i32,
    pub protocol: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct NewProxy {
    pub protocol: String,
    pub host: String,
    pub port: i16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub provider: String,
}
