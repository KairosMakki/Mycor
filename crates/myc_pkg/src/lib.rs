//! Package management for Mycor.

use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
}

pub fn load_manifest(dir: &Path) -> Option<Package> {
    let path = dir.join("mycor.toml");
    let text = std::fs::read_to_string(path).ok()?;
    let mut name = String::new();
    let mut version = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("name") {
            name = v.trim().trim_matches('=').trim().trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("version") {
            version = v.trim().trim_matches('=').trim().trim_matches('"').to_string();
        }
    }
    Some(Package { name, version })
}