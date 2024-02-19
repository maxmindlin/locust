use locust_core::models::proxies::NewProxy;

use super::ProxyFileParser;

pub struct InfaticaParser;

impl ProxyFileParser for InfaticaParser {
    fn parse_file(&self, content: &str) -> Vec<NewProxy> {
        content
            .trim()
            .lines()
            .map(|line| {
                let split: Vec<&str> = line.split([':', '@']).collect();
                let user = split[0];
                let pass = split[1];
                let ip = split[2];
                let port = split[3];

                NewProxy {
                    protocol: "http".into(),
                    host: ip.into(),
                    port: port.parse().unwrap(),
                    username: Some(user.into()),
                    password: Some(pass.into()),
                    provider: "infatica".into(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infatica_parser() {
        let input = "user:pass@1.1.1.1:10001
user:pass@1.1.1.1:10002
";
        let parser = InfaticaParser {};
        let r = parser.parse_file(input);
        let exp = vec![
            NewProxy {
                protocol: "http".into(),
                host: "1.1.1.1".into(),
                port: 10001,
                username: Some("user".into()),
                password: Some("pass".into()),
                provider: "infatica".into(),
            },
            NewProxy {
                protocol: "http".into(),
                host: "1.1.1.1".into(),
                port: 10002,
                username: Some("user".into()),
                password: Some("pass".into()),
                provider: "infatica".into(),
            },
        ];
        assert_eq!(r, exp);
    }
}
