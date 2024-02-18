use std::{process, thread};

use crate::farm::ExistingVM;

pub async fn delete_vms(zone: String) {
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
    let mut rdr = csv::Reader::from_reader(output.stdout.as_slice());
    let mut threads = vec![];
    for record in rdr.deserialize() {
        let record: ExistingVM = record.unwrap();
        let zone = zone.clone();
        threads.push(thread::spawn(move || {
            println!("DELETING {}", record.name);
            let delete_output = process::Command::new("gcloud")
                .arg("compute")
                .arg("instances")
                .arg("delete")
                .arg(&record.name)
                .arg(format!("--zone={zone}"))
                .output()
                .expect("failed to run delete command");
            println!("status: {}", delete_output.status);
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
}
