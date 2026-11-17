//! Name resolution for Mycor.

use myc_ast::File;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ResolveResult {
    pub names: HashMap<String, String>,
}

pub fn resolve_file(_file: &File) -> ResolveResult {
    ResolveResult::default()
}
