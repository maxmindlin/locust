use std::process;

use locust_core::models::proxies::NewProxy;
use uuid::Uuid;

use crate::farm::CreatedVM;

const DEFAULT_SQUID_PORT: u16 = 3128;

pub async fn create_vms(
    project: &str,
    zone: &str,
    username: &str,
    pwd: &str,
    num: u16,
) -> Vec<NewProxy> {
    let id = Uuid::new_v4();
    let filename = concat!(env!("OUT_DIR"), "/new_squid.sh");

    let mut proxies = Vec::new();
    for i in 0..num {
        let output = process::Command::new("sh")
            .arg(filename)
            .arg(project)
            .arg(zone)
            .arg(username)
            .arg(pwd)
            .arg(i.to_string())
            .arg(id.to_string())
            .output()
            .expect("failed to execute farm script");
        let mut rdr = csv::Reader::from_reader(output.stdout.as_slice());
        for record in rdr.deserialize() {
            let record: CreatedVM = record.unwrap();
            let p = NewProxy {
                protocol: "http".into(),
                host: record.external_ip.clone(),
                port: DEFAULT_SQUID_PORT as i16,
                username: Some(username.to_string()),
                password: Some(pwd.to_string()),
                provider: "squid".into(),
            };
            proxies.push(p);
        }
    }

    proxies
}
