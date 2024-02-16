use std::fmt::Display;

use locust_core::models::proxies::Proxy;
use tabled::builder;

pub struct ProxyTable(pub Vec<Proxy>);

impl Display for ProxyTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = builder::Builder::new();
        builder.push_record(["id", "protocol", "host", "port", "provider"]);
        for proxy in &self.0 {
            builder.push_record([
                &proxy.id.to_string(),
                &proxy.protocol,
                &proxy.host,
                &proxy.port.to_string(),
                &proxy.provider,
            ]);
        }

        let table = builder.build().to_string();
        write!(f, "{}", table)
    }
}
