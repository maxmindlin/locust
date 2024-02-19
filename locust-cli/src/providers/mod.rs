use locust_core::models::proxies::NewProxy;

pub mod infatica;
pub mod webshare;

pub trait ProxyFileParser {
    fn parse_file(&self, content: &str) -> Vec<NewProxy>;
}
