use sqlx::{postgres::PgPool, Error, Row};

use crate::models::proxies::{NewProxy, Proxy, ProxyDomainCoeff, ProxySession};

const DEFAULT_COEF: i32 = 50;
const COEF_INCR: i32 = 5;

/// Gets the appropriate proxy for a given domain.
/// If no proxy is attached to the given domain via locust_tags,
/// then it gets a general proxy ordered by the date
/// of its last use.
///
/// Updates the date_last_used value of the returned proxy.
pub async fn get_proxy_by_domain(pool: &PgPool, domain: &str) -> Result<Proxy, Error> {
    let proxy = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                p.id, p.protocol, p.host, p.port, p.username, p.password, p.provider
            FROM locust_proxies as p
            JOIN locust_proxy_tag_map as ptm ON p.id = ptm.proxy_id
            JOIN locust_tags as t ON ptm.tag_id = t.id
            JOIN locust_domain_tag_map as dtm ON t.id = dtm.tag_id
            JOIN locust_domains as d ON d.id = dtm.domain_id
            WHERE d.host = $1 AND p.date_deleted IS NULL
            ORDER BY p.date_last_used DESC
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
            FROM locust_proxies as p
            WHERE p.date_deleted IS NULL
            ORDER BY p.date_last_used DESC
        "#,
    )
    .fetch_one(pool)
    .await?;

    update_proxy_last_used(pool, proxy.id).await?;

    Ok(proxy)
}

pub async fn get_proxy_by_id(pool: &PgPool, id: i32) -> Result<Proxy, Error> {
    let proxy = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                id, protocol, host, port, username, password, provider
            FROM locust_proxies
            WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    update_proxy_last_used(pool, id).await?;

    Ok(proxy)
}

async fn update_proxy_last_used(pool: &PgPool, id: i32) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE locust_proxies
            SET date_last_used = now()
            WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_proxies_by_tags(pool: &PgPool, locust_tags: &[&str]) -> Result<Vec<Proxy>, Error> {
    let proxies = sqlx::query_as::<_, Proxy>(
        r#"
            SELECT
                p.id, p.protocol, p.host, p.port, p.username, p.password, p.provider
            FROM locust_proxies as p
            JOIN locust_proxy_tag_map as ptm ON p.id = ptm.proxy_id
            JOIN locust_tags as t ON ptm.tag_id = t.id
            WHERE t.name = any($1)
            AND p.date_deleted IS NULL
        "#,
    )
    .bind(locust_tags)
    .fetch_all(pool)
    .await?;

    Ok(proxies)
}

pub async fn delete_proxies_by_ids(pool: &PgPool, ids: &[i32]) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE locust_proxies
            SET date_deleted = now()
            WHERE id = any($1)
        "#,
    )
    .bind(ids)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_proxies_by_tags(pool: &PgPool, locust_tags: &[&str]) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE locust_proxies
            SET date_deleted = now()
            WHERE id IN (
                SELECT p.id
                FROM locust_proxies as p
                JOIN locust_proxy_tag_map as ptm ON p.id = ptm.proxy_id
                JOIN locust_tags as t ON ptm.tag_id = t.id
                WHERE t.name = any($1)
                AND p.date_deleted IS NULL
            )
        "#,
    )
    .bind(locust_tags)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_proxies(
    pool: &PgPool,
    proxies: &[NewProxy],
    locust_tags: &[&str],
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let mut tag_ids: Vec<i32> = Vec::new();
    for tag in locust_tags {
        let id = sqlx::query(
            r#"
                INSERT INTO locust_tags (name)
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
                locust_proxies (protocol, host, port, username, password, provider)
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
                    locust_proxy_tag_map (proxy_id, tag_id)
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

pub async fn get_proxy_session(pool: &PgPool, id: i32) -> Result<ProxySession, Error> {
    let session = sqlx::query_as::<_, ProxySession>(
        r#"
            SELECT id, proxy_id FROM locust_sessions WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(session)
}

pub async fn create_proxy_session(pool: &PgPool, proxy_id: i32) -> Result<ProxySession, Error> {
    let session = sqlx::query_as::<_, ProxySession>(
        r#"
            INSERT INTO
            locust_sessions (proxy_id)
            values ($1)
            RETURNING id, proxy_id
        "#,
    )
    .bind(proxy_id)
    .fetch_one(pool)
    .await?;

    Ok(session)
}

pub async fn increment_proxy_coefficient(
    pool: &PgPool,
    proxy_id: i32,
    domain_id: i32,
    success: bool,
) -> Result<(), Error> {
    let maybe_coef = sqlx::query_as::<_, ProxyDomainCoeff>(
        r#"
            SELECT proxy_id, domain_id, coefficient
            FROM locust_domain_coefficients
            WHERE proxy_id = $1 AND domain_id = $2
        "#,
    )
    .bind(proxy_id)
    .bind(domain_id)
    .fetch_optional(pool)
    .await?;

    match maybe_coef {
        Some(coef) => {
            let incr = match success {
                true => std::cmp::min(coef.coefficient + COEF_INCR, 100),
                false => std::cmp::max(coef.coefficient - COEF_INCR, 0),
            };

            sqlx::query(
                r#"
                    UPDATE locust_domain_coefficients
                    SET coefficient = $1
                    WHERE proxy_id = $2 AND domain_id = $3
                "#,
            )
            .bind(incr)
            .bind(proxy_id)
            .bind(domain_id)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query(
                r#"
                    INSERT INTO
                    locust_domain_coefficients (proxy_id, domain_id, coefficient)
                    values ($1, $2, $3)
                "#,
            )
            .bind(proxy_id)
            .bind(domain_id)
            .bind(DEFAULT_COEF)
            .execute(pool)
            .await?;
        }
    };

    Ok(())
}
