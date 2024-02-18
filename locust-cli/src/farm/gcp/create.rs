use std::{
    process,
    sync::{Arc, Mutex},
    thread,
};

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

    let proxies: Arc<Mutex<Vec<NewProxy>>> = Arc::new(Mutex::new(Vec::new()));
    thread::scope(|s| {
        for i in 0..num {
            let proxies_a = Arc::clone(&proxies);
            s.spawn(move || {
                let vm_id = format!("squid-{}-{}", id, i);
                println!("CREATING {}", vm_id);
                let output = process::Command::new("gcloud")
                    .arg("compute")
                    .arg("instances")
                    .arg("create")
                    .arg(vm_id)
                    .arg("--format=csv(name,zone,status,networkInterfaces[0].accessConfigs[0].natIP:label=external_ip)")
                    .arg(format!("--project={project}"))
                    .arg(format!("--zone={zone}"))
                    .arg("--machine-type=e2-micro")
                    .arg("--image-family=debian-12")
                    .arg("--image-project=debian-cloud")
                    .arg("--tags=squid-proxy-locust")
                    .arg(format!(r#"--metadata=startup-script=#! /bin/bash
sudo apt-get update
sudo apt-get install -y squid apache2-utils
cat <<'EOF' > /etc/squid/squid.conf
acl Safe_ports port 80 443
acl CONNECT method CONNECT 
http_access allow localhost
http_access deny !Safe_ports
http_access deny CONNECT !Safe_ports
forwarded_for off
request_header_access Allow allow all
request_header_access Authorization allow all
request_header_access WWW-Authenticate allow all
request_header_access Proxy-Authorization allow all
request_header_access Proxy-Authenticate allow all
request_header_access Cache-Control allow all
request_header_access Content-Encoding allow all
request_header_access Content-Length allow all
request_header_access Content-Type allow all
request_header_access Date allow all
request_header_access Expires allow all
request_header_access Host allow all
request_header_access If-Modified-Since allow all
request_header_access Last-Modified allow all
request_header_access Location allow all
request_header_access Pragma allow all
request_header_access Accept allow all
request_header_access Accept-Charset allow all
request_header_access Accept-Encoding allow all
request_header_access Accept-Language allow all
request_header_access Content-Language allow all
request_header_access Mime-Version allow all
request_header_access Retry-After allow all
request_header_access Title allow all
request_header_access Connection allow all
request_header_access Proxy-Connection allow all
request_header_access User-Agent allow all
request_header_access Cookie allow all
request_header_access All deny all
auth_param basic program /usr/lib/squid/basic_ncsa_auth /etc/squid/passwd
auth_param basic realm proxy
acl authenticated proxy_auth REQUIRED
http_access allow authenticated
http_port 3128
EOF
echo "{username}:$(openssl passwd -apr1 {pwd})" | sudo tee /etc/squid/passwd > /dev/null
sudo systemctl restart squid
            "#))
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
                    proxies_a.lock().unwrap().push(p);
                }

            });

            // let mut handles = Vec::new();
            // let mut rdr = csv::Reader::from_reader(output.stdout.as_slice());
            // let proxies: Vec<NewProxy> = rdr
            //     .deserialize()
            //     .par_bridge()
            //     .into_par_iter()
            //     .map(|r| {
            //         let record: CreatedVM = r.unwrap();

            //         NewProxy {
            //             protocol: "http".into(),
            //             host: record.external_ip.clone(),
            //             port: DEFAULT_SQUID_PORT as i16,
            //             username: Some(username.to_string()),
            //             password: Some(pwd.to_string()),
            //             provider: "squid".into(),
            //         }
            //     })
            //     .collect();

            // proxies
            // for record in rdr.deserialize() {
            //     let record: CreatedVM = record.unwrap();
            //     let p = NewProxy {
            //         protocol: "http".into(),
            //         host: record.external_ip.clone(),
            //         port: DEFAULT_SQUID_PORT as i16,
            //         username: Some(username.to_string()),
            //         password: Some(pwd.to_string()),
            //         provider: "squid".into(),
            //     };
            //     // proxies.push(p);
            // }
        }
    });

    Arc::try_unwrap(proxies).unwrap().into_inner().unwrap()
}
