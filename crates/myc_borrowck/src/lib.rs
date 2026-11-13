//! Ownership and borrow checking for Mycor (simplified).

use myc_ast::*;
use myc_diagnostics::{Diagnostic, Label, codes};
use myc_types::{Ty, expr_span};
use std::collections::{HashMap, HashSet};

pub fn check(file: &File, type_map: &HashMap<u32, Ty>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for item in &file.items {
        if let Item::Fn(f) = item
            && let Some(body) = &f.body
        {
            let mut ctx = Ctx {
                type_map,
                moved: HashSet::new(),
                diags: &mut diags,
            };
            check_block(body, &mut ctx);
        }
    }
    diags
}

struct Ctx<'a> {
    type_map: &'a HashMap<u32, Ty>,
    moved: HashSet<String>,
    diags: &'a mut Vec<Diagnostic>,
}

fn is_copy_expr(e: &Expr, type_map: &HashMap<u32, Ty>) -> bool {
    let ty = type_map
        .get(&expr_span(e).start)
        .cloned()
        .unwrap_or(Ty::I32);
    ty.is_copy()
}

fn check_block(b: &Block, ctx: &mut Ctx) {
    for stmt in &b.stmts {
        check_stmt(stmt, ctx);
    }
    if let Some(e) = &b.expr {
        check_expr(e, ctx);
    }
}

fn check_stmt(s: &Stmt, ctx: &mut Ctx) {
    match s {
        Stmt::Let(l) => {
            if let Some(init) = &l.init {
                check_expr(init, ctx);
            }
        }
        Stmt::Return(Some(e)) => check_expr(e, ctx),
        Stmt::Return(None) => {}
        Stmt::Expr(e) => check_expr(e, ctx),
        Stmt::Defer(e) => check_expr(e, ctx),
        _ => {}
    }
}

fn check_expr(e: &Expr, ctx: &mut Ctx) {
    match e {
        Expr::Path(p, _) if p.len() == 1 => {
            let name = &p[0];
            if ctx.moved.contains(name) {
                ctx.diags.push(
                    Diagnostic::error(codes::E0300, format!("use after move: `{}`", name))
                        .with_label(Label::primary(
                            expr_span(e),
                            "used here after move".to_string(),
                        )),
                );
            }
        }
        Expr::Call { callee, args, .. } => {
            // if callee is a function taking non-copy by value, mark moved
            for a in args {
                check_expr(a, ctx);
            }
            if let Expr::Path(p, _) = &**callee
                && p.len() == 1
            {
                let fname = &p[0];
                // heuristic: if arg is a simple path and function is not builtin, mark moved
                // We'll mark moved for any non-copy arg path when calling user fn.
                // To avoid false positives for println etc, skip builtins.
                if fname != "println" && fname != "print" {
                    for a in args {
                        if let Expr::Path(pp, _) = a
                            && pp.len() == 1
                            && !is_copy_expr(a, ctx.type_map)
                        {
                            let argname = &pp[0];
                            ctx.moved.insert(argname.clone());
                        }
                    }
                }
            }
            check_expr(callee, ctx);
        }
        Expr::Binary { left, right, .. } => {
            check_expr(left, ctx);
            check_expr(right, ctx);
        }
        Expr::Unary { operand, .. } => check_expr(operand, ctx),
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            check_expr(cond, ctx);
            check_block(then_branch, ctx);
            if let Some(eb) = else_branch {
                check_expr(eb, ctx);
            }
        }
        Expr::While { cond, body, .. } => {
            check_expr(cond, ctx);
            check_block(body, ctx);
        }
        Expr::For { iter, body, .. } => {
            check_expr(iter, ctx);
            check_block(body, ctx);
        }
        Expr::Block(b) => check_block(b, ctx),
        Expr::Assign { target, value, .. } => {
            check_expr(target, ctx);
            check_expr(value, ctx);
        }
        Expr::Index { base, index, .. } => {
            check_expr(base, ctx);
            check_expr(index, ctx);
        }
        Expr::Field { base, .. } => check_expr(base, ctx),
        Expr::Cast { expr, .. } => check_expr(expr, ctx),
        Expr::Try { expr, .. } => check_expr(expr, ctx),
        Expr::Unsafe { block, .. } => check_block(block, ctx),
        Expr::Spawn { body, .. } => check_block(body, ctx),
        Expr::Tuple { elems, .. } => {
            for e in elems {
                check_expr(e, ctx);
            }
        }
        Expr::Array { elems, .. } => {
            for e in elems {
                check_expr(e, ctx);
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            check_expr(scrutinee, ctx);
            for arm in arms {
                check_expr(&arm.body, ctx);
            }
        }
        _ => {}
    }
}
