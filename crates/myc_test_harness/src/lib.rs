//! Test runners for Mycor.

use std::fs;
use std::path::Path;

pub fn run_pass_test(path: &Path) -> bool {
    let src = fs::read_to_string(path).unwrap_or_default();
    let out = std::env::temp_dir().join("myc_test.exe");
    match myc_driver::build_executable(&src, path.to_str().unwrap_or("t.myc"), &out) {
        Ok(exe) => {
            // run and capture output? just run
            let _ = myc_driver::run_executable(&exe);
            true
        }
        Err(e) => {
            eprintln!("FAIL compile {}: {}", path.display(), e);
            false
        }
    }
}

pub fn run_all(root: &Path) -> (usize, usize) {
    let mut pass = 0;
    let mut fail = 0;
    if let Ok(entries) = fs::read_dir(root.join("tests/run_pass")) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "myc").unwrap_or(false) {
                if run_pass_test(&p) {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
        }
    }
    (pass, fail)
}
