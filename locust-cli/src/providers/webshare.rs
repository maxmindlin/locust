use locust_core::models::proxies::NewProxy;

use super::ProxyFileParser;

pub struct WebshareParser;

impl ProxyFileParser for WebshareParser {
    fn parse_file(&self, content: &str) -> Vec<NewProxy> {
        content
            .trim()
            .lines()
            .map(|line| {
                let split: Vec<&str> = line.split(':').collect();
                let ip = split[0];
                let port = split[1];
                let user = split[2];
                let pwd = split[3];

                NewProxy {
                    protocol: "http".into(),
                    host: ip.into(),
                    port: port.parse().unwrap(),
                    username: Some(user.into()),
                    password: Some(pwd.into()),
                    provider: "webshare".into(),
                }
            })
            .collect()
    }
}
