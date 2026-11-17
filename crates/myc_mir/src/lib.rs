//! Mid-level IR for Mycor.

use myc_ast::{File, Item};
use myc_types::Ty;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
}

#[derive(Clone, Debug)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
}

pub fn lower_file(file: &File, _type_map: &HashMap<u32, Ty>) -> Vec<MirFunction> {
    let mut out = Vec::new();
    for item in &file.items {
        match item {
            Item::Fn(f) => {
                let mut params = Vec::new();
                for p in &f.params {
                    let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
                    params.push((p.name.clone(), ty));
                }
                let ret = f.ret.as_ref().map(Ty::from_ast).unwrap_or(Ty::Void);
                out.push(MirFunction {
                    name: f.name.clone(),
                    params,
                    ret,
                });
            }
            Item::Impl(i) => {
                for it in &i.items {
                    if let Item::Fn(f) = it {
                        let mut params = Vec::new();
                        for p in &f.params {
                            let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
                            params.push((p.name.clone(), ty));
                        }
                        let ret = f.ret.as_ref().map(Ty::from_ast).unwrap_or(Ty::Void);
                        out.push(MirFunction {
                            name: f.name.clone(),
                            params,
                            ret,
                        });
                    }
                }
            }
            Item::Trait(t) => {
                for it in &t.items {
                    if let Item::Fn(f) = it {
                        let mut params = Vec::new();
                        for p in &f.params {
                            let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
                            params.push((p.name.clone(), ty));
                        }
                        let ret = f.ret.as_ref().map(Ty::from_ast).unwrap_or(Ty::Void);
                        out.push(MirFunction {
                            name: f.name.clone(),
                            params,
                            ret,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    out
}

pub fn lower_module(file: &File, type_map: &HashMap<u32, Ty>) -> MirModule {
    MirModule {
        functions: lower_file(file, type_map),
    }
}