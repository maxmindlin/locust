use std::env;

use sqlx::{postgres::PgPoolOptions, Error, PgPool};
use urlencoding::encode;

pub mod crud;
pub mod models;

pub async fn new_pool() -> Result<PgPool, Error> {
    let conn_string = get_conn_string();
    let pool = PgPoolOptions::new().connect(&conn_string).await?;

    Ok(pool)
}

pub fn get_conn_string() -> String {
    let user = env::var("POSTGRES_USER").unwrap_or("postgres".into());
    let pwd = env::var("POSTGRES_PASSWORD").unwrap_or("password".into());
    let pwd = encode(&pwd);
    let db = env::var("POSTGRES_DB").unwrap_or("postgres".into());
    let host = env::var("POSTGRES_HOST").unwrap_or("localhost".into());
    let port = env::var("POSTGRES_PORT")
        .unwrap_or("5432".into())
        .parse::<usize>()
        .expect("Invalid psql port");
    format!("postgresql://{}:{}@{}:{}/{}", user, pwd, host, port, db)
}
