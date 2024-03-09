use telegraf::Metric;

#[derive(Debug)]
pub enum MetricsError {
    WriteError(String),
}

pub trait MetricClient {
    fn send_proxy_metric(&mut self, metric: &ProxyMetric) -> Result<(), MetricsError>;
}

#[derive(Metric)]
pub struct ProxyMetric {
    #[telegraf(tag)]
    pub domain: String,
    #[telegraf(tag)]
    pub proxy_id: i32,
    pub response_time: u32,
    pub status: u16,
}

pub struct TelegrafClient {
    client: telegraf::Client,
}

impl TelegrafClient {
    pub fn new(addr: &str) -> Self {
        Self {
            client: telegraf::Client::new(addr).unwrap(),
        }
    }
}

impl MetricClient for TelegrafClient {
    fn send_proxy_metric(&mut self, metric: &ProxyMetric) -> Result<(), MetricsError> {
        if let Err(e) = self.client.write(metric) {
            return Err(MetricsError::WriteError(e.to_string()));
        }

        Ok(())
    }
}
