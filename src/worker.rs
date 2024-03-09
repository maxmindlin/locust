use http::StatusCode;
use sqlx::PgPool;
use std::sync::{mpsc, Arc};
use tracing::warn;

use crate::metrics::{MetricClient, ProxyMetric};

pub struct DBWorker<T> {
    pool: Arc<PgPool>,
    channel: mpsc::Receiver<DBJob>,
    metrics_clients: Option<T>,
}

impl<T> DBWorker<T>
where
    T: MetricClient,
{
    pub fn new(
        pool: Arc<PgPool>,
        channel: mpsc::Receiver<DBJob>,
        metrics_clients: Option<T>,
    ) -> Self {
        Self {
            pool,
            channel,
            metrics_clients,
        }
    }

    pub async fn start(&mut self) {
        while let Ok(job) = self.channel.recv() {
            self.process_job(job).await;
        }

        warn!("Error receiving worker job. Exiting");
    }

    async fn process_job(&mut self, job: DBJob) {
        match job {
            DBJob::ProxyResponse {
                proxy_id,
                status,
                response_time,
                domain,
            } => {
                let metric = ProxyMetric {
                    proxy_id,
                    domain: domain.unwrap_or("".to_string()),
                    status: status.as_u16(),
                    response_time,
                };

                if let Some(client) = &mut self.metrics_clients {
                    if let Err(e) = client.send_proxy_metric(&metric) {
                        warn!("error sending proxy metric: {e:?}");
                    };
                }

                // Modify success coefficient in DB
                // to keep track of proxy success across domains.
                // Coeff will be used to calculate likelihood of
                // What proxies to use.

                // Check for domain entry

                // Add domain if not exists

                // Increment/decrement entry for proxy_id & domain
                // Add entry with default if not exists
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
