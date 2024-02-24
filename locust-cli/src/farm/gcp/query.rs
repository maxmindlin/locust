use std::process;

use crate::farm::ExistingVM;

pub fn query_vms(zone: &str) -> Vec<ExistingVM> {
    println!("GETTING EXISTING VM LIST...");
    let output = process::Command::new("gcloud")
        .arg("compute")
        .arg("instances")
        .arg("list")
        .arg(format!(
            "--filter=tags.items=squid-proxy-locust AND zone={zone}"
        ))
        .arg("--format=csv(name)")
        .output()
        .expect("failed to query existing instances");
    csv::Reader::from_reader(output.stdout.as_slice())
        .deserialize()
        .map(|r| r.unwrap())
        .collect()
}
