use std::sync::{mpsc, Arc};
use http::StatusCode;
use sqlx::PgPool;
use tracing::warn;
use crate::metrics::{MetricsService, Metric};

pub struct DBWorker {
    pool: Arc<PgPool>,
    channel: mpsc::Receiver<DBJob>,
    metrics_service: MetricsService,
}

impl DBWorker {

    pub fn new(pool: Arc<PgPool>, channel: mpsc::Receiver<DBJob>, metrics_service: MetricsService) -> Self {
        Self { pool, channel, metrics_service }
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
                response_time,
                domain,
            } => {

                 if let Some(domain) = domain {
                 let proxy_id = proxy_id.to_string();
                    let metric = Metric {
                        measurement: "proxy_response",
                        tags: vec![
                            ("domain", &domain),
                            ("proxy_id", &proxy_id),
                        ],
                        fields: vec![
                            ("response_time", response_time as f64),
                            ("status", status.as_u16() as f64),
                        ],
                    };

                    if let Err(e) = self.metrics_service.send_metric(metric).await {
                        eprintln!("Failed to send metric: {}", e);}}
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
