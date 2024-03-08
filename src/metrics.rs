use dotenv::dotenv;
use std::env;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

#[derive(Serialize)]
pub struct Metric<'a> {
    pub measurement: &'a str,
    pub tags: Vec<(&'a str, &'a str)>,
    pub fields: Vec<(&'a str, f64)>,
}

pub struct MetricsService {
    telegraf_url: String,
}

impl MetricsService {
    pub fn new() -> Self {
        dotenv().ok(); // Load .env file, if exists

        let host = env::var("TELEGRAFCLIENT_HOST").expect("TELEGRAFCLIENT_HOST not set");
        let port = env::var("TELEGRAFCLIENT_PORT").expect("TELEGRAFCLIENT_PORT not set");
        // Directly use host and port for the TCP connection
        let telegraf_url = format!("{}:{}", host, port);

        MetricsService {
            telegraf_url,
        }
    }

    pub async fn send_metric<'a>(&self, metric: Metric<'a>) -> Result<(), Box<dyn std::error::Error>> {
        let influx_data = metric.fields.iter().fold(
            format!(
                "{},{} {}",
                metric.measurement,
                metric.tags.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(","),
                metric.fields.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(",")
            ),
            |acc, _| acc,
        );

        let mut stream = TcpStream::connect(&self.telegraf_url).await?;
        stream.write_all(influx_data.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }
}
