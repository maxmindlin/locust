use std::{process, thread};

use super::query::query_vms;

pub fn delete_vms(zone: String) {
    let mut threads = Vec::new();
    for vm in query_vms(&zone) {
        let zone = zone.clone();
        threads.push(thread::spawn(move || {
            println!("DELETING {}", vm.name);
            let delete_output = process::Command::new("gcloud")
                .arg("compute")
                .arg("instances")
                .arg("delete")
                .arg(&vm.name)
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
