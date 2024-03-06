use sqlx::{Error, PgPool};

use crate::models::domains::Domain;

pub async fn get_domain_by_host(pool: &PgPool, host: &str) -> Result<Option<Domain>, Error> {
    sqlx::query_as::<_, Domain>(
        r#"
            SELECT id, host
            FROM locust_domains
            WHERE host = $1
        "#,
    )
    .bind(host)
    .fetch_optional(pool)
    .await
}

pub async fn create_domain(pool: &PgPool, host: &str) -> Result<Domain, Error> {
    sqlx::query_as::<_, Domain>(
        r#"
            INSERT INTO
            locust_domains (host)
            values ($1) ON CONFLICT (host) DO UPDATE
            SET host=EXCLUDED.host
            RETURNING id, host
        "#,
    )
    .bind(host)
    .fetch_one(pool)
    .await
}
