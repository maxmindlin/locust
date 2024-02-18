use sqlx::{postgres::PgPool, Error, Row};

use crate::models::proxies::{NewProxy, Proxy};

/// Gets the appropriate proxy for a given domain.
/// If no proxy is attached to the given domain via tags,
/// then it gets a general proxy ordered by the date
/// of its last use.
///
/// Updates the date_last_used value of the returned proxy.
pub async fn get_proxy_by_domain(pool: &PgPool, domain: &str) -> Result<Proxy, Error> {
    let proxy = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                p.id, p.protocol, p.host, p.port, p.username, p.password, p.provider
            FROM proxies as p
            JOIN proxy_tag_map as ptm ON p.id = ptm.proxy_id
            JOIN tags as t ON ptm.tag_id = t.id
            JOIN domain_tag_map as dtm ON t.id = dtm.tag_id
            JOIN domains as d ON d.id = dtm.domain_id
            WHERE d.host = $1 AND p.date_deleted IS NULL
            ORDER BY p.date_last_used
        "#,
    )
    .bind(domain)
    .fetch_optional(pool)
    .await?;

    let proxy = match proxy {
        Some(p) => p,
        None => return get_general_proxy(pool).await,
    };

    update_proxy_last_used(pool, proxy.id).await?;

    Ok(proxy)
}

pub async fn get_general_proxy(pool: &PgPool) -> Result<Proxy, Error> {
    let proxy = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                p.id, p.protocol, p.host, p.port, p.username, p.password, p.provider
            FROM proxies as p
            JOIN proxy_tag_map as ptm ON p.id = ptm.proxy_id
            JOIN tags as t ON ptm.tag_id = t.id
            JOIN domain_tag_map as dtm ON t.id = dtm.tag_id
            JOIN domains as d ON d.id = dtm.domain_id
            WHERE p.date_deleted IS NULL
            ORDER BY p.date_last_used
        "#,
    )
    .fetch_one(pool)
    .await?;

    update_proxy_last_used(pool, proxy.id).await?;

    Ok(proxy)
}

async fn update_proxy_last_used(pool: &PgPool, id: i32) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE proxies
            SET date_last_used = now()
            WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_proxies_by_tags(pool: &PgPool, tags: &[String]) -> Result<Vec<Proxy>, Error> {
    let proxies = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                p.id, p.protocol, p.host, p.port, p.username, p.password, p.provider
            FROM proxies as p
            JOIN proxy_tag_map as ptm ON p.id = ptm.proxy_id
            JOIN tags as t ON ptm.tag_id = t.id
            WHERE t.name = any($1)
            AND p.date_deleted IS NULL
        "#,
    )
    .bind(tags)
    .fetch_all(pool)
    .await?;

    Ok(proxies)
}

pub async fn delete_proxies_by_tags(pool: &PgPool, tags: &[String]) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE proxies
            SET date_deleted = now()
            WHERE id IN (
                SELECT p.id
                FROM proxies as p
                JOIN proxy_tag_map as ptm ON p.id = ptm.proxy_id
                JOIN tags as t ON ptm.tag_id = t.id
                WHERE t.name = any($1)
                AND p.date_deleted IS NULL
            )
        "#,
    )
    .bind(tags)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_proxies(pool: &PgPool, proxies: &[NewProxy], tags: &[&str]) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let mut tag_ids: Vec<i32> = Vec::new();
    for tag in tags {
        let id = sqlx::query(
            r#"
                INSERT INTO tags (name) 
                values ($1) ON CONFLICT (name) DO UPDATE 
                SET name=EXCLUDED.name
                RETURNING id
            "#,
        )
        .bind(tag)
        .fetch_one(&mut *tx)
        .await?
        .try_get("id")?;
        tag_ids.push(id);
    }

    for proxy in proxies {
        let proxy_id: i32 = sqlx::query(
            r#"
                INSERT INTO
                proxies (protocol, host, port, username, password, provider)
                values ($1, $2, $3, $4, $5, $6)
                RETURNING id
            "#,
        )
        .bind(&proxy.protocol)
        .bind(&proxy.host)
        .bind(proxy.port)
        .bind(&proxy.username)
        .bind(&proxy.password)
        .bind(&proxy.provider)
        .fetch_one(&mut *tx)
        .await?
        .try_get("id")?;

        for tag_id in &tag_ids {
            sqlx::query(
                r#"
                    INSERT INTO
                    proxy_tag_map (proxy_id, tag_id)
                    values ($1, $2)
                "#,
            )
            .bind(proxy_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
