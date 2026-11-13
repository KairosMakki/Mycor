//! C11 backend for Mycor.

use myc_ast::*;
use myc_types::{Ty, expr_span};
use std::collections::HashMap;

pub fn generate(file: &File, type_map: &HashMap<u32, Ty>) -> String {
    let mut g = Gen {
        type_map,
        out: String::new(),
        indent: 0,
        structs: Vec::new(),
    };
    // collect structs
    for item in &file.items {
        if let Item::Struct(s) = item {
            g.structs.push(s.clone());
        }
    }
    // includes
    g.out.push_str("#include <stdint.h>\n");
    g.out.push_str("#include <stdbool.h>\n");
    g.out.push_str("#include <stdio.h>\n");
    g.out.push_str("#include <stdlib.h>\n\n");

    // struct defs
    for s in &g.structs.clone() {
        g.out.push_str(&format!("typedef struct {} {{\n", s.name));
        for f in &s.fields {
            g.out.push_str(&format!(
                "    {} {};\n",
                c_type(&Ty::from_ast(&f.ty)),
                f.name
            ));
        }
        g.out.push_str(&format!("}} {};\n\n", s.name));
    }

    // functions
    for item in &file.items {
        match item {
            Item::Fn(f) => g.gen_fn(f),
            Item::Impl(_)
            | Item::Trait(_)
            | Item::Const(_)
            | Item::Static(_)
            | Item::TypeAlias(_)
            | Item::Mod(_)
            | Item::Use(_)
            | Item::Enum(_)
            | Item::Struct(_) => {}
        }
    }

    g.out
}

struct Gen<'a> {
    type_map: &'a HashMap<u32, Ty>,
    out: String,
    indent: usize,
    structs: Vec<StructDef>,
}

impl<'a> Gen<'a> {
    fn ind(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn gen_fn(&mut self, f: &FnDef) {
        let ret = if f.name == "main" {
            "int".to_string()
        } else {
            f.ret
                .as_ref()
                .map(|t| c_type(&Ty::from_ast(t)).to_string())
                .unwrap_or_else(|| "void".to_string())
        };
        // main handling
        let name = if f.name == "main" {
            "main".to_string()
        } else {
            f.name.clone()
        };
        let params = if f.params.is_empty() {
            if f.name == "main" {
                "void".to_string()
            } else {
                "".to_string()
            }
        } else {
            f.params
                .iter()
                .map(|p| {
                    let ty =
                        p.ty.as_ref()
                            .map(|t| c_type(&Ty::from_ast(t)).to_string())
                            .unwrap_or_else(|| "int32_t".to_string());
                    format!("{} {}", ty, p.name)
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.out
            .push_str(&format!("{} {}({}) {{\n", ret, name, params));
        self.indent += 1;
        if let Some(body) = &f.body {
            for stmt in &body.stmts {
                self.gen_stmt(stmt);
            }
            if let Some(e) = &body.expr {
                let e = &**e;
                match e {
                    Expr::If { .. }
                    | Expr::While { .. }
                    | Expr::For { .. }
                    | Expr::Loop { .. }
                    | Expr::Unsafe { .. }
                    | Expr::Block(_) => {
                        let s = Stmt::Expr(e.clone());
                        self.gen_stmt(&s);
                        if f.name == "main" {
                            self.out.push_str(&format!("{}return 0;\n", self.ind()));
                        }
                    }
                    _ => {
                        let code = self.gen_expr(e);
                        self.out
                            .push_str(&format!("{}return {};\n", self.ind(), code));
                    }
                }
            } else if f.name == "main" {
                self.out.push_str(&format!("{}return 0;\n", self.ind()));
            }
        } else if f.name == "main" {
            self.out.push_str(&format!("{}return 0;\n", self.ind()));
        }
        self.indent -= 1;
        self.out.push_str("}\n\n");
    }

    fn gen_block_stmts(&mut self, b: &Block) {
        for stmt in &b.stmts {
            self.gen_stmt(stmt);
        }
        if let Some(e) = &b.expr {
            let code = self.gen_expr(e);
            self.out.push_str(&format!("{}{};\n", self.ind(), code));
        }
    }

    fn gen_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let(l) => {
                let ty = if let Some(t) = &l.ty {
                    c_type(&Ty::from_ast(t))
                } else if let Some(init) = &l.init {
                    c_type(&self.expr_ty(init))
                } else {
                    "int32_t".to_string()
                };
                let mut line = format!("{}{}", self.ind(), ty);
                line.push(' ');
                line.push_str(&l.name);
                if let Some(init) = &l.init {
                    line.push_str(" = ");
                    line.push_str(&self.gen_expr(init));
                }
                line.push(';');
                self.out.push_str(&line);
                self.out.push('\n');
            }
            Stmt::Expr(e) => match e {
                Expr::If {
                    cond,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    let c = self.gen_expr(cond);
                    self.out.push_str(&format!("{}if ({}) {{\n", self.ind(), c));
                    self.indent += 1;
                    self.gen_block_stmts(then_branch);
                    self.indent -= 1;
                    if let Some(eb) = else_branch {
                        self.out.push_str(&format!("{}}} else {{\n", self.ind()));
                        self.indent += 1;
                        // else branch may be block or expr
                        match eb.as_ref() {
                            Expr::Block(b) => self.gen_block_stmts(b),
                            _ => {
                                let code = self.gen_expr(eb);
                                self.out.push_str(&format!("{}{};\n", self.ind(), code));
                            }
                        }
                        self.indent -= 1;
                    }
                    self.out.push_str(&format!("{}}}\n", self.ind()));
                }
                Expr::While { cond, body, .. } => {
                    let c = self.gen_expr(cond);
                    self.out
                        .push_str(&format!("{}while ({}) {{\n", self.ind(), c));
                    self.indent += 1;
                    self.gen_block_stmts(body);
                    self.indent -= 1;
                    self.out.push_str(&format!("{}}}\n", self.ind()));
                }
                Expr::For {
                    name, iter, body, ..
                } => {
                    // for (int32_t name = 0; name < iter; name++) { ... } (simplified)
                    self.out.push_str(&format!(
                        "{}for (int32_t {} = 0; {} < ",
                        self.ind(),
                        name,
                        name
                    ));
                    // iter should be upper bound expression
                    let bound = self.gen_expr(iter);
                    self.out.push_str(&format!("{}; {}++) {{\n", bound, name));
                    self.indent += 1;
                    self.gen_block_stmts(body);
                    self.indent -= 1;
                    self.out.push_str(&format!("{}}}\n", self.ind()));
                }
                Expr::Loop { body, .. } => {
                    self.out.push_str(&format!("{}while (1) {{\n", self.ind()));
                    self.indent += 1;
                    self.gen_block_stmts(body);
                    self.indent -= 1;
                    self.out.push_str(&format!("{}}}\n", self.ind()));
                }
                Expr::Spawn { body, .. } => {
                    self.out.push_str(&format!("{}{{\n", self.ind()));
                    self.indent += 1;
                    self.gen_block_stmts(body);
                    self.indent -= 1;
                    self.out.push_str(&format!("{}}}\n", self.ind()));
                }
                _ => {
                    let code = self.gen_expr(e);
                    self.out.push_str(&format!("{}{};\n", self.ind(), code));
                }
            },
            Stmt::Return(e) => {
                if let Some(e) = e {
                    let code = self.gen_expr(e);
                    self.out
                        .push_str(&format!("{}return {};\n", self.ind(), code));
                } else {
                    self.out.push_str(&format!("{}return;\n", self.ind()));
                }
            }
            Stmt::Defer(e) => {
                let code = self.gen_expr(e);
                self.out.push_str(&format!("{}{};\n", self.ind(), code));
            }
            Stmt::Break => self.out.push_str(&format!("{}break;\n", self.ind())),
            Stmt::Continue => self.out.push_str(&format!("{}continue;\n", self.ind())),
            Stmt::Item(_) => {}
        }
    }

    fn gen_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Lit(Lit::Int(v), _) => v.to_string(),
            Expr::Lit(Lit::Float(v), _) => format!("{}", v),
            Expr::Lit(Lit::Str(s), _) => format!("\"{}\"", escape_str(s)),
            Expr::Lit(Lit::Char(c), _) => format!("'{}'", c),
            Expr::Lit(Lit::Bool(b), _) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Expr::Path(p, _) => {
                if p.len() == 1 {
                    p[0].clone()
                } else {
                    p.join("_")
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let o = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Rem => "%",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    BinOp::BitXor => "^",
                    BinOp::Shl => "<<",
                    BinOp::Shr => ">>",
                    _ => "+",
                };
                format!("({} {} {})", l, o, r)
            }
            Expr::Unary { op, operand, .. } => {
                let v = self.gen_expr(operand);
                match op {
                    UnOp::Neg => format!("(-{})", v),
                    UnOp::Not => format!("(!{})", v),
                    UnOp::Deref => format!("(*{})", v),
                    UnOp::Ref => format!("(&{})", v),
                    UnOp::RefMut => format!("(&{})", v),
                }
            }
            Expr::Call { callee, args, .. } => {
                let cname = match &**callee {
                    Expr::Path(p, _) if p.len() == 1 => p[0].clone(),
                    _ => "unknown".to_string(),
                };
                let a: Vec<String> = args.iter().map(|x| self.gen_expr(x)).collect();
                if cname == "println" || cname == "print" {
                    // format based on first arg type
                    if args.len() == 1 {
                        let ty = self.expr_ty(&args[0]);
                        match ty {
                            Ty::Str => return format!("printf(\"%s\\n\", {})", a[0]),
                            Ty::I32 | Ty::I64 | Ty::Isize | Ty::Usize => {
                                return format!("printf(\"%d\\n\", {})", a[0]);
                            }
                            Ty::F64 | Ty::F32 => return format!("printf(\"%f\\n\", {})", a[0]),
                            Ty::Bool => {
                                return format!(
                                    "printf(\"%s\\n\", {} ? \"true\" : \"false\")",
                                    a[0]
                                );
                            }
                            Ty::Char => return format!("printf(\"%c\\n\", {})", a[0]),
                            _ => return format!("printf(\"%d\\n\", {})", a[0]),
                        }
                    }
                    // multi-arg: print each? just first
                    if args.is_empty() {
                        return "printf(\"\\n\")".to_string();
                    }
                }
                format!("{}({})", cname, a.join(", "))
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                // statement-like expression; generate as ternary? We'll generate if statement not expression.
                // For expression context, wrap in IIFE not possible in C. Generate conditional expression if both branches are expressions?
                // Fallback: generate GNU statement expr? Use ternary for simple. We'll just generate (cond ? then : else) with blocks?
                let c = self.gen_expr(cond);
                // then branch as last expr if tail
                let t = if let Some(e) = &then_branch.expr {
                    self.gen_expr(e)
                } else {
                    "0".to_string()
                };
                if let Some(eb) = else_branch {
                    let e = self.gen_expr(eb);
                    format!("({} ? {} : {})", c, t, e)
                } else {
                    format!("({} ? {} : 0)", c, t)
                }
            }
            Expr::Block(b) => {
                // not supported as expression well; generate 0
                let _ = b;
                "0".to_string()
            }
            Expr::Assign { target, value, .. } => {
                let t = self.gen_expr(target);
                let v = self.gen_expr(value);
                format!("({} = {})", t, v)
            }
            Expr::Index { base, index, .. } => {
                let b = self.gen_expr(base);
                let i = self.gen_expr(index);
                format!("{}[{}]", b, i)
            }
            Expr::Field { base, name, .. } => {
                let b = self.gen_expr(base);
                format!("{}.{}", b, name)
            }
            Expr::Cast { expr, ty, .. } => {
                let e = self.gen_expr(expr);
                format!("(({}){})", c_type(&Ty::from_ast(ty)), e)
            }
            Expr::Try { expr, .. } => self.gen_expr(expr),
            Expr::Unsafe { block, .. } => {
                let _ = block;
                "0".to_string()
            }
            Expr::Spawn { body, .. } => {
                let _ = body;
                "0".to_string()
            }
            Expr::Tuple { .. } => "0".to_string(),
            Expr::Array { .. } => "0".to_string(),
            Expr::Range { .. } => "0".to_string(),
            Expr::Closure { .. } => "0".to_string(),
            Expr::Match { .. } => "0".to_string(),
            Expr::MethodCall {
                receiver,
                method,
                args,
                ..
            } => {
                let r = self.gen_expr(receiver);
                let mut a = vec![r];
                a.extend(args.iter().map(|x| self.gen_expr(x)));
                format!("{}({})", method, a.join(", "))
            }
            Expr::While { .. } | Expr::For { .. } | Expr::Loop { .. } => "0".to_string(),
        }
    }

    fn expr_ty(&self, e: &Expr) -> Ty {
        self.type_map
            .get(&expr_span(e).start)
            .cloned()
            .unwrap_or(Ty::I32)
    }
}

fn c_type(ty: &Ty) -> String {
    match ty {
        Ty::I8 => "int8_t".to_string(),
        Ty::I16 => "int16_t".to_string(),
        Ty::I32 => "int32_t".to_string(),
        Ty::I64 => "int64_t".to_string(),
        Ty::Isize => "intptr_t".to_string(),
        Ty::U8 => "uint8_t".to_string(),
        Ty::U16 => "uint16_t".to_string(),
        Ty::U32 => "uint32_t".to_string(),
        Ty::U64 => "uint64_t".to_string(),
        Ty::Usize => "size_t".to_string(),
        Ty::F32 => "float".to_string(),
        Ty::F64 => "double".to_string(),
        Ty::Bool => "bool".to_string(),
        Ty::Char => "char".to_string(),
        Ty::Str => "const char*".to_string(),
        Ty::Named(n) => n.clone(),
        Ty::Void => "void".to_string(),
        _ => "int32_t".to_string(),
    }
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
