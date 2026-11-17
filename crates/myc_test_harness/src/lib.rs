//! Test runners for Mycor.

use std::fs;
use std::path::Path;
use std::process::Command;

fn extract_error_code(src: &str) -> Option<String> {
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("//") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix("error:") {
                let code = rest.trim();
                if !code.is_empty() {
                    return Some(code.to_string());
                }
            }
        }
    }
    // also support `// error: E0300` anywhere in line
    for line in src.lines() {
        if let Some(idx) = line.find("error:") {
            let after = &line[idx + "error:".len()..];
            let code = after.trim().split_whitespace().next().unwrap_or("");
            if code.starts_with('E') {
                return Some(code.to_string());
            }
        }
    }
    None
}

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
}

pub fn run_pass_test(path: &Path) -> bool {
    let src = fs::read_to_string(path).unwrap_or_default();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("test");
    let out = std::env::temp_dir().join(format!("myc_test_{}.exe", stem));
    match myc_driver::build_executable(&src, path.to_str().unwrap_or("t.myc"), &out) {
        Ok(exe) => match Command::new(&exe).output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let expected_path = path.with_extension("stdout");
                if expected_path.exists() {
                    let expected = fs::read_to_string(&expected_path).unwrap_or_default();
                    if normalize(&stdout) == normalize(&expected) {
                        true
                    } else {
                        eprintln!(
                            "FAIL output {}: expected {:?}, got {:?}",
                            path.display(),
                            expected,
                            stdout
                        );
                        false
                    }
                } else {
                    true
                }
            }
            Err(e) => {
                eprintln!("FAIL run {}: {}", path.display(), e);
                false
            }
        },
        Err(e) => {
            eprintln!("FAIL compile {}: {}", path.display(), e);
            false
        }
    }
}

pub fn run_compile_fail_test(path: &Path) -> bool {
    let src = fs::read_to_string(path).unwrap_or_default();
    let expected = extract_error_code(&src);
    let diags = myc_driver::check_source(&src, path.to_str().unwrap_or("t.myc"));
    // ensure it actually fails
    let first_error = diags
        .iter()
        .find(|d| d.severity == myc_diagnostics::Severity::Error);
    if first_error.is_none() {
        // maybe build still fails? try build
        let out = std::env::temp_dir().join("myc_cf.exe");
        if myc_driver::build_executable(&src, path.to_str().unwrap_or("t.myc"), &out).is_ok() {
            eprintln!("FAIL expected compile failure: {}", path.display());
            return false;
        }
        // no diagnostic code available; accept if no expected code
        return expected.is_none();
    }
    let actual = first_error.unwrap().code.to_string();
    match expected {
        Some(exp) => {
            if actual == exp {
                true
            } else {
                eprintln!(
                    "FAIL code {}: expected {}, got {}",
                    path.display(),
                    exp,
                    actual
                );
                false
            }
        }
        None => true,
    }
}

// aliases for compatibility
pub fn run_fail_test(path: &Path) -> bool {
    run_compile_fail_test(path)
}
pub fn run_compile_fail(path: &Path) -> bool {
    run_compile_fail_test(path)
}
pub fn check_compile_fail(path: &Path) -> bool {
    run_compile_fail_test(path)
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
    if let Ok(entries) = fs::read_dir(root.join("tests/compile_fail")) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "myc").unwrap_or(false) {
                if run_compile_fail_test(&p) {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
        }
    }
    (pass, fail)
}
