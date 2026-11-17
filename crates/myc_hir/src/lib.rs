//! High-level IR for Mycor (typed).

use myc_ast::File;
use myc_types::Ty;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
}

#[derive(Clone, Debug)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
}

pub fn lower_to_hir(file: &File, type_map: &HashMap<u32, Ty>) -> HirModule {
    let mut functions = Vec::new();
    for item in &file.items {
        if let myc_ast::Item::Fn(f) = item {
            let mut params = Vec::new();
            for p in &f.params {
                let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
                params.push((p.name.clone(), ty));
            }
            let ret = f.ret.as_ref().map(Ty::from_ast).unwrap_or(Ty::Void);
            functions.push(HirFunction {
                name: f.name.clone(),
                params,
                ret,
            });
        }
    }
    let _ = type_map;
    HirModule { functions }
}
