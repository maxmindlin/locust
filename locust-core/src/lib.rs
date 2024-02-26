use std::env;

use sqlx::{postgres::PgPoolOptions, Error, PgPool};

pub mod crud;
pub mod models;

pub async fn new_pool() -> Result<PgPool, Error> {
    let user = env::var("POSTGRES_USER").unwrap_or("postgres".into());
    let pwd = env::var("POSTGRES_PASSWORD").unwrap_or("password".into());
    let db = env::var("POSTGRES_DB").unwrap_or("postgres".into());
    let host = env::var("POSTGRES_HOST").unwrap_or("localhost".into());
    let port = env::var("POSTGRES_PORT")
        .unwrap_or("5432".into())
        .parse::<usize>()
        .expect("Invalid psql port");
    let conn_string = format!("postgresql://{}:{}@{}:{}/{}", user, pwd, host, port, db);

    let pool = PgPoolOptions::new().connect(&conn_string).await?;

    Ok(pool)
}
