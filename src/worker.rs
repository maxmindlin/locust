use std::sync::{mpsc, Arc};

use http::StatusCode;
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
                response_time,
                domain,
            } => {}
        }
    }
}

pub enum DBJob {
    ProxyResponse {
        proxy_id: i32,
        status: StatusCode,
        response_time: u32,
        domain: Option<String>,
    },
}
