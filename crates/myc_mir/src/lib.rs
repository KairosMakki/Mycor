//! Mid-level IR for Mycor (simplified).

use myc_ast::File;
use myc_types::Ty;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
}

pub fn lower_file(_file: &File, _type_map: &HashMap<u32, Ty>) -> Vec<MirFunction> {
    // Lowering is a no-op placeholder; the backend currently consumes the AST directly.
    Vec::new()
}
