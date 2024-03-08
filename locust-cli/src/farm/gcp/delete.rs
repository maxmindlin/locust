use std::{process, thread};

use crate::farm::ExistingVM;

use super::{query::query_vms, THREAD_MAX_SIZE};

pub fn query_and_delete_vms(zone: String) {
    // let mut threads = Vec::new();
    // for vm in query_vms(&zone) {
    //     let zone = zone.clone();
    //     threads.push(thread::spawn(move || {
    //         delete_vm(zone, vm);
    //     }));
    // }

    // for t in threads {
    //     t.join().unwrap();
    // }

    let vms = query_vms(&zone);
    let n_vms = vms.len();
    let mut batch = 0;
    while batch < n_vms - 1 {
        let mut threads = Vec::new();
        let start = batch;
        batch += THREAD_MAX_SIZE as usize;
        let ceil = std::cmp::min(batch, n_vms - 1);
        for i in start..ceil {
            let vm = vms[i].clone();
            let zone = zone.clone();
            threads.push(thread::spawn(move || {
                delete_vm(zone, vm);
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}

pub fn delete_vms(zone: String, vms: Vec<ExistingVM>) {
    let n_vms = vms.len();
    let mut batch = 0;
    while batch < n_vms - 1 {
        let mut threads = Vec::new();
        let start = batch;
        batch += THREAD_MAX_SIZE as usize;
        let ceil = std::cmp::min(batch, n_vms - 1);
        for i in start..ceil {
            let vm = vms[i].clone();
            let zone = zone.clone();
            threads.push(thread::spawn(move || {
                delete_vm(zone, vm);
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}

pub fn delete_vm(zone: String, vm: ExistingVM) {
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
}
