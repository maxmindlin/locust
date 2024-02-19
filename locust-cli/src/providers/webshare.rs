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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webshare_parser() {
        let input = "1.1.1.1:1:user:pass
1.1.1.1:2:user1:pass1
";
        let parser = WebshareParser {};
        let r = parser.parse_file(input);
        let exp = vec![
            NewProxy {
                protocol: "http".into(),
                host: "1.1.1.1".into(),
                port: 1,
                username: Some("user".into()),
                password: Some("pass".into()),
                provider: "webshare".into(),
            },
            NewProxy {
                protocol: "http".into(),
                host: "1.1.1.1".into(),
                port: 2,
                username: Some("user1".into()),
                password: Some("pass1".into()),
                provider: "webshare".into(),
            },
        ];
        assert_eq!(r, exp);
    }
}
