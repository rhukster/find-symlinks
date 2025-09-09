use std::{env, fs, io::{Read, Write}, path::PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());

    // Allow CI/user to force the build number, else auto-increment a local counter
    let build_number = env::var("BUILD_NUMBER").ok().and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| increment_local_build_number(&manifest_dir));

    // Export env vars for the main crate
    println!("cargo:rustc-env=BUILD_NUMBER={}", build_number);
    println!(
        "cargo:rustc-env=PKG_VERSION_WITH_BUILD={}",
        format!("{} (build {})", pkg_version, build_number)
    );

    // Re-run logic: rebuild if build number file or env changes
    println!("cargo:rerun-if-env-changed=BUILD_NUMBER");
    println!("cargo:rerun-if-changed=build/build-number");
}

fn increment_local_build_number(manifest_dir: &str) -> u64 {
    let mut path = PathBuf::from(manifest_dir);
    path.push("build");
    let _ = fs::create_dir_all(&path);
    path.push("build-number");

    let mut n: u64 = 0;
    if let Ok(mut f) = fs::File::open(&path) {
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        n = s.trim().parse::<u64>().unwrap_or(0);
    }
    n = n.saturating_add(1);
    if let Ok(mut f) = fs::File::create(&path) {
        let _ = write!(f, "{}\n", n);
    }
    n
}
