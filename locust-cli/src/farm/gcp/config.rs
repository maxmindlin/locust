use std::{
    io::{self, Write},
    process,
};

pub fn config_firewall() {
    let output = process::Command::new("gcloud")
        .arg("compute")
        .arg("firewall-rules")
        .arg("create")
        .arg("allow-squid-proxy-locust")
        .arg("--direction=INGRESS")
        .arg("--priority=1000")
        .arg("--network=default")
        .arg("--action=ALLOW")
        .arg("--rules=tcp:3128")
        .arg("--source-ranges=0.0.0.0/0")
        .arg("--target-tags=squid-proxy-locust")
        .output()
        .expect("error running firewall command");
    println!("Status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}
