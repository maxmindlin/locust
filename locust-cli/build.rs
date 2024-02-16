use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("new_squid.sh");
    fs::copy("src/farm/gcp/new_squid.sh", dest_path).unwrap();
    println!("cargo:rerun-if-changed=src/farm/gcp/new_squid.sh");
}
