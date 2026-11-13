//! Type representation, inference, and checking for Mycor.

use myc_ast::*;
use myc_diagnostics::{Diagnostic, Label, codes};
use myc_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Void,
    Named(String),
    Ref(Box<Ty>),
    RefMut(Box<Ty>),
    Ptr(Box<Ty>),
    PtrMut(Box<Ty>),
    Array(Box<Ty>),
    Slice(Box<Ty>),
    Tuple(Vec<Ty>),
    Unknown,
    Error,
}

impl Ty {
    pub fn from_ast(t: &Type) -> Self {
        match t {
            Type::Named(name, _args, _) => match name.as_str() {
                "i8" => Ty::I8,
                "i16" => Ty::I16,
                "i32" => Ty::I32,
                "i64" => Ty::I64,
                "isize" => Ty::Isize,
                "u8" => Ty::U8,
                "u16" => Ty::U16,
                "u32" => Ty::U32,
                "u64" => Ty::U64,
                "usize" => Ty::Usize,
                "f32" => Ty::F32,
                "f64" => Ty::F64,
                "bool" => Ty::Bool,
                "char" => Ty::Char,
                "str" => Ty::Str,
                _ => Ty::Named(name.clone()),
            },
            Type::Ref(inner, _) => Ty::Ref(Box::new(Ty::from_ast(inner))),
            Type::RefMut(inner, _) => Ty::RefMut(Box::new(Ty::from_ast(inner))),
            Type::Ptr(inner, _) => Ty::Ptr(Box::new(Ty::from_ast(inner))),
            Type::PtrMut(inner, _) => Ty::PtrMut(Box::new(Ty::from_ast(inner))),
            Type::Array(inner, _, _) => Ty::Array(Box::new(Ty::from_ast(inner))),
            Type::Slice(inner, _) => Ty::Slice(Box::new(Ty::from_ast(inner))),
            Type::Tuple(elems, _) => Ty::Tuple(elems.iter().map(Ty::from_ast).collect()),
            Type::Infer(_) => Ty::Unknown,
            _ => Ty::Unknown,
        }
    }

    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            Ty::I8
                | Ty::I16
                | Ty::I32
                | Ty::I64
                | Ty::Isize
                | Ty::U8
                | Ty::U16
                | Ty::U32
                | Ty::U64
                | Ty::Usize
                | Ty::F32
                | Ty::F64
                | Ty::Bool
                | Ty::Char
                | Ty::Ptr(_)
                | Ty::PtrMut(_)
        )
    }
}

#[derive(Clone, Debug)]
pub struct FnSig {
    pub name: String,
    pub params: Vec<Ty>,
    pub ret: Ty,
}

#[derive(Default)]
pub struct Checker {
    pub diagnostics: Vec<Diagnostic>,
    functions: HashMap<String, FnSig>,
    structs: HashMap<String, Vec<(String, Ty)>>,
    scopes: Vec<HashMap<String, Ty>>,
    pub type_map: HashMap<u32, Ty>, // key: span start
}

impl Checker {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            scopes: Vec::new(),
            type_map: HashMap::new(),
        }
    }

    pub fn check_file(&mut self, file: &File) {
        // first pass: collect functions and structs
        for item in &file.items {
            match item {
                Item::Fn(f) => {
                    let mut params = Vec::new();
                    for p in &f.params {
                        let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
                        params.push(ty);
                    }
                    let ret = f.ret.as_ref().map(Ty::from_ast).unwrap_or(Ty::Void);
                    self.functions.insert(
                        f.name.clone(),
                        FnSig {
                            name: f.name.clone(),
                            params,
                            ret,
                        },
                    );
                }
                Item::Struct(s) => {
                    let mut fields = Vec::new();
                    for fld in &s.fields {
                        fields.push((fld.name.clone(), Ty::from_ast(&fld.ty)));
                    }
                    self.structs.insert(s.name.clone(), fields);
                }
                _ => {}
            }
        }
        // check items
        for item in &file.items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.check_fn(f),
            Item::Const(c) => {
                let t = self.check_expr(&c.value);
                if let Some(ty) = &c.ty {
                    let et = Ty::from_ast(ty);
                    if t != Ty::Unknown && t != Ty::Error && t != et {
                        self.diag_type_mismatch(expr_span(&c.value), &et, &t);
                    }
                }
            }
            Item::Struct(_) | Item::Enum(_) | Item::Trait(_) | Item::Impl(_) => {}
            _ => {}
        }
    }

    fn check_fn(&mut self, f: &FnDef) {
        self.push_scope();
        for p in &f.params {
            let ty = p.ty.as_ref().map(Ty::from_ast).unwrap_or(Ty::Unknown);
            self.bind(&p.name, ty);
        }
        if let Some(body) = &f.body {
            self.check_block(body);
            // if return type declared, check tail expr? skip
        }
        self.pop_scope();
    }

    fn check_block(&mut self, b: &Block) {
        self.push_scope();
        for stmt in &b.stmts {
            self.check_stmt(stmt);
        }
        if let Some(e) = &b.expr {
            self.check_expr(e);
        }
        self.pop_scope();
    }

    fn check_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let(l) => {
                let init_ty = if let Some(init) = &l.init {
                    self.check_expr(init)
                } else {
                    Ty::Unknown
                };
                let ty = if let Some(t) = &l.ty {
                    let et = Ty::from_ast(t);
                    if init_ty != Ty::Unknown && init_ty != Ty::Error && init_ty != et {
                        self.diag_type_mismatch(l.span, &et, &init_ty);
                    }
                    et
                } else if init_ty == Ty::Unknown {
                    // default i32?
                    Ty::I32
                } else {
                    init_ty
                };
                self.bind(&l.name, ty);
            }
            Stmt::Expr(e) => {
                self.check_expr(e);
            }
            Stmt::Return(Some(e)) => {
                self.check_expr(e);
            }
            Stmt::Return(None) => {}
            Stmt::Defer(e) => {
                self.check_expr(e);
            }
            _ => {}
        }
        // handle If/While/For/Match as expressions in Stmt::Expr? Actually Stmt enum has no If variant. If is expr statement. okay.
    }

    fn check_expr(&mut self, e: &Expr) -> Ty {
        let ty = match e {
            Expr::Lit(Lit::Int(_), _) => Ty::I32,
            Expr::Lit(Lit::Float(_), _) => Ty::F64,
            Expr::Lit(Lit::Str(_), _) => Ty::Str,
            Expr::Lit(Lit::Char(_), _) => Ty::Char,
            Expr::Lit(Lit::Bool(_), _) => Ty::Bool,
            Expr::Path(path, _) => {
                if path.len() == 1 {
                    self.lookup(&path[0]).unwrap_or_else(|| {
                        self.diag_unknown(path[0].clone(), expr_span(e));
                        Ty::Error
                    })
                } else {
                    // function or module path; assume function value not supported -> unknown
                    Ty::Unknown
                }
            }
            Expr::Call { callee, args, .. } => {
                // callee path
                let callee_name = match &**callee {
                    Expr::Path(p, _) if p.len() == 1 => p[0].clone(),
                    _ => String::new(),
                };
                if let Some(sig) = self.functions.get(&callee_name).cloned() {
                    if args.len() != sig.params.len() {
                        self.diag_wrong_args(expr_span(e), args.len(), sig.params.len());
                    }
                    for (a, pt) in args.iter().zip(sig.params.iter()) {
                        let at = self.check_expr(a);
                        if at != Ty::Unknown && at != Ty::Error && at != *pt {
                            self.diag_type_mismatch(expr_span(a), pt, &at);
                        }
                    }
                    sig.ret
                } else {
                    // unknown function; check args anyway
                    for a in args {
                        self.check_expr(a);
                    }
                    // print? special
                    if callee_name == "println" || callee_name == "print" {
                        for a in args {
                            self.check_expr(a);
                        }
                        Ty::Void
                    } else {
                        Ty::Unknown
                    }
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let lt = self.check_expr(left);
                let rt = self.check_expr(right);
                use BinOp::*;
                match op {
                    Add | Sub | Mul | Div | Rem => {
                        if lt == Ty::Str && rt == Ty::Str && matches!(op, Add) {
                            Ty::Str
                        } else if lt == Ty::F64 || rt == Ty::F64 {
                            Ty::F64
                        } else {
                            Ty::I32
                        }
                    }
                    Eq | Ne | Lt | Le | Gt | Ge | And | Or => Ty::Bool,
                    _ => Ty::Unknown,
                }
            }
            Expr::Unary { op, operand, .. } => {
                let t = self.check_expr(operand);
                match op {
                    UnOp::Neg => t,
                    UnOp::Not => Ty::Bool,
                    UnOp::Ref => Ty::Ref(Box::new(t)),
                    UnOp::RefMut => Ty::RefMut(Box::new(t)),
                    UnOp::Deref => match t {
                        Ty::Ref(x) | Ty::RefMut(x) | Ty::Ptr(x) | Ty::PtrMut(x) => *x,
                        _ => Ty::Unknown,
                    },
                }
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let ct = self.check_expr(cond);
                if ct != Ty::Bool && ct != Ty::Unknown && ct != Ty::Error {
                    self.diag_type_mismatch(expr_span(cond), &Ty::Bool, &ct);
                }
                self.check_block(then_branch);
                if let Some(eb) = else_branch {
                    self.check_expr(eb);
                }
                Ty::Void
            }
            Expr::While { cond, body, .. } => {
                let ct = self.check_expr(cond);
                if ct != Ty::Bool && ct != Ty::Unknown && ct != Ty::Error {
                    self.diag_type_mismatch(expr_span(cond), &Ty::Bool, &ct);
                }
                self.check_block(body);
                Ty::Void
            }
            Expr::For {
                name, iter, body, ..
            } => {
                self.check_expr(iter);
                self.push_scope();
                self.bind(name, Ty::I32);
                self.check_block(body);
                self.pop_scope();
                Ty::Void
            }
            Expr::Loop { body, .. } => {
                self.check_block(body);
                Ty::Void
            }
            Expr::Block(b) => {
                self.check_block(b);
                Ty::Void
            }
            Expr::Assign { target, value, .. } => {
                let vt = self.check_expr(value);
                let tt = self.check_expr(target);
                if tt != Ty::Unknown
                    && tt != Ty::Error
                    && vt != Ty::Unknown
                    && vt != Ty::Error
                    && tt != vt
                {
                    self.diag_type_mismatch(expr_span(value), &tt, &vt);
                }
                Ty::Void
            }
            Expr::Cast { expr, ty, .. } => {
                self.check_expr(expr);
                Ty::from_ast(ty)
            }
            Expr::Index { base, index, .. } => {
                self.check_expr(index);
                let bt = self.check_expr(base);
                match bt {
                    Ty::Array(x) | Ty::Slice(x) => *x,
                    Ty::Str => Ty::Char,
                    _ => Ty::Unknown,
                }
            }
            Expr::Field { base, name, .. } => {
                let bt = self.check_expr(base);
                if let Ty::Named(sname) = bt
                    && let Some(fields) = self.structs.get(&sname)
                    && let Some((_, ft)) = fields.iter().find(|(n, _)| n == name)
                {
                    return ft.clone();
                }
                Ty::Unknown
            }
            Expr::MethodCall {
                receiver,
                method: _,
                args,
                ..
            } => {
                self.check_expr(receiver);
                for a in args {
                    self.check_expr(a);
                }
                // vec push etc not handled
                Ty::Void
            }
            Expr::Try { expr, .. } => self.check_expr(expr),
            Expr::Tuple { elems, .. } => {
                let mut ts = Vec::new();
                for e in elems {
                    ts.push(self.check_expr(e));
                }
                Ty::Tuple(ts)
            }
            Expr::Array { elems, .. } => {
                let mut t = Ty::Unknown;
                if let Some(first) = elems.first() {
                    t = self.check_expr(first);
                }
                for e in elems.iter().skip(1) {
                    self.check_expr(e);
                }
                Ty::Array(Box::new(t))
            }
            Expr::Closure { body, .. } => {
                self.check_expr(body);
                Ty::Unknown
            }
            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.check_expr(scrutinee);
                for arm in arms {
                    self.check_expr(&arm.body);
                }
                Ty::Void
            }
            Expr::Range { .. } => Ty::Unknown,
            Expr::Unsafe { block, .. } => {
                self.check_block(block);
                Ty::Void
            }
            Expr::Spawn { body, .. } => {
                self.check_block(body);
                Ty::Void
            }
        };
        // record type for all expr spans
        self.record(expr_span(e).start, ty.clone());
        ty
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn bind(&mut self, name: &str, ty: Ty) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }
    fn lookup(&mut self, name: &str) -> Option<Ty> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn record(&mut self, key: u32, ty: Ty) {
        self.type_map.insert(key, ty);
    }

    fn diag_unknown(&mut self, name: String, span: Span) {
        self.diagnostics.push(
            Diagnostic::error(codes::E0100, format!("unknown name `{}`", name))
                .with_label(Label::primary(span, "not found".to_string())),
        );
    }
    fn diag_type_mismatch(&mut self, span: Span, expected: &Ty, found: &Ty) {
        self.diagnostics.push(
            Diagnostic::error(
                codes::E0200,
                format!("type mismatch: expected {:?}, found {:?}", expected, found),
            )
            .with_label(Label::primary(span, "here".to_string())),
        );
    }
    fn diag_wrong_args(&mut self, span: Span, got: usize, expected: usize) {
        self.diagnostics.push(
            Diagnostic::error(
                codes::E0203,
                format!(
                    "wrong number of arguments: got {}, expected {}",
                    got, expected
                ),
            )
            .with_label(Label::primary(span, "here".to_string())),
        );
    }
}

pub fn check_file(file: &File) -> (Checker, ()) {
    let mut c = Checker::new();
    c.check_file(file);
    (c, ())
}

pub fn expr_span(e: &Expr) -> Span {
    match e {
        Expr::Lit(_, s) => *s,
        Expr::Path(_, s) => *s,
        Expr::Call { span, .. } => *span,
        Expr::MethodCall { span, .. } => *span,
        Expr::Field { span, .. } => *span,
        Expr::Index { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Assign { span, .. } => *span,
        Expr::If { span, .. } => *span,
        Expr::While { span, .. } => *span,
        Expr::For { span, .. } => *span,
        Expr::Loop { span, .. } => *span,
        Expr::Match { span, .. } => *span,
        Expr::Block(b) => b.span,
        Expr::Closure { span, .. } => *span,
        Expr::Cast { span, .. } => *span,
        Expr::Range { span, .. } => *span,
        Expr::Try { span, .. } => *span,
        Expr::Unsafe { span, .. } => *span,
        Expr::Spawn { span, .. } => *span,
        Expr::Tuple { span, .. } => *span,
        Expr::Array { span, .. } => *span,
    }
}
