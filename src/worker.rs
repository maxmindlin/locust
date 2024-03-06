use std::sync::{mpsc, Arc};

use http::StatusCode;
use locust_core::crud::{
    domains::{create_domain, get_domain_by_host},
    proxies::increment_proxy_coefficient,
};
use sqlx::PgPool;
use tracing::warn;

pub struct DBWorker {
    pool: Arc<PgPool>,
    channel: mpsc::Receiver<DBJob>,
}

impl DBWorker {
    pub fn new(pool: Arc<PgPool>, channel: mpsc::Receiver<DBJob>) -> Self {
        Self { pool, channel }
    }

    pub async fn start(&self) {
        while let Ok(job) = self.channel.recv() {
            self.process_job(job).await;
        }

        warn!("Error receiving worker job. Exiting");
    }

    async fn process_job(&self, job: DBJob) {
        match job {
            DBJob::ProxyResponse {
                proxy_id,
                status,
                response_time: _,
                domain,
            } => {
                // Modify success coefficient in DB
                // to keep track of proxy success across domains.
                // Coeff will be used to calculate likelihood of
                // What proxies to use.

                if domain.is_none() {
                    warn!("Worker received domain of None");
                    return;
                }

                let host = domain.unwrap();

                // Check for domain entry
                let maybe_domain = get_domain_by_host(&self.pool, &host)
                    .await
                    .expect("Error getting domain");

                // Add domain if not exists
                let domain_id = match maybe_domain {
                    Some(domain) => domain.id,
                    None => {
                        create_domain(&self.pool, &host)
                            .await
                            .expect("Error creating domain")
                            .id
                    }
                };

                // Increment/decrement entry for proxy_id & domain
                // Add entry with default if not exists
                increment_proxy_coefficient(&self.pool, proxy_id, domain_id, status.as_u16() < 400)
                    .await
                    .expect("Error incrementing proxy coeff");
            }
            DBJob::CalcNextProxies {} => {}
        }
    }
}

pub enum DBJob {
    /// Results from a proxy response.
    ProxyResponse {
        proxy_id: i32,
        status: StatusCode,
        response_time: u32,
        domain: Option<String>,
    },

    /// Time to calculate next proxy
    /// to use across domains based upon
    /// success coefficients.
    CalcNextProxies {},
}
