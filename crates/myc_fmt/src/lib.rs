//! Code formatter for Mycor.

use myc_ast::*;
use myc_lexer::Lexer;
use myc_parser::Parser;
use myc_span::SourceMap;

pub fn format_source(source: &str) -> Result<String, String> {
    let mut sm = SourceMap::new();
    let fid = sm.add_file("fmt.myc", source);
    let src = sm.source(fid).to_string();
    let (toks, diags) = Lexer::new(&src, fid).lex();
    if !diags.is_empty() {
        return Err("lex errors".to_string());
    }
    let (file, pdiags) = Parser::new(toks, fid).parse_file();
    if !pdiags.is_empty() {
        return Err("parse errors".to_string());
    }
    let mut out = String::new();
    for (i, item) in file.items.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        fmt_item(item, &mut out, 0);
        out.push('\n');
    }
    Ok(out)
}

fn ind(n: usize) -> String {
    "    ".repeat(n)
}

fn fmt_item(item: &Item, out: &mut String, d: usize) {
    match item {
        Item::Fn(f) => {
            out.push_str(&ind(d));
            if f.is_pub {
                out.push_str("pub ");
            }
            if f.is_unsafe {
                out.push_str("unsafe ");
            }
            out.push_str("fn ");
            out.push_str(&f.name);
            if !f.generics.is_empty() {
                out.push('[');
                for (i, g) in f.generics.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&g.name);
                }
                out.push(']');
            }
            out.push('(');
            for (i, p) in f.params.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&p.name);
                if let Some(t) = &p.ty {
                    out.push_str(": ");
                    fmt_type(t, out);
                }
            }
            out.push(')');
            if let Some(t) = &f.ret {
                out.push_str(" -> ");
                fmt_type(t, out);
            }
            if let Some(b) = &f.body {
                out.push_str(" {\n");
                fmt_block(b, out, d + 1);
                out.push_str(&ind(d));
                out.push('}');
            } else {
                out.push(';');
            }
            out.push('\n');
        }
        Item::Struct(s) => {
            out.push_str(&ind(d));
            out.push_str("struct ");
            out.push_str(&s.name);
            out.push_str(" {\n");
            for f in &s.fields {
                out.push_str(&ind(d + 1));
                out.push_str(&f.name);
                out.push_str(": ");
                fmt_type(&f.ty, out);
                out.push_str(",\n");
            }
            out.push_str(&ind(d));
            out.push_str("}\n");
        }
        Item::Const(c) => {
            out.push_str(&ind(d));
            out.push_str("const ");
            out.push_str(&c.name);
            if let Some(t) = &c.ty {
                out.push_str(": ");
                fmt_type(t, out);
            }
            out.push_str(" = ");
            fmt_expr(&c.value, out, d);
            out.push_str(";\n");
        }
        _ => {
            out.push_str(&ind(d));
            out.push_str("// item\n");
        }
    }
}

fn fmt_block(b: &Block, out: &mut String, d: usize) {
    for stmt in &b.stmts {
        fmt_stmt(stmt, out, d);
    }
    if let Some(e) = &b.expr {
        out.push_str(&ind(d));
        fmt_expr(e, out, d);
        out.push('\n');
    }
}

fn fmt_stmt(s: &Stmt, out: &mut String, d: usize) {
    match s {
        Stmt::Let(l) => {
            out.push_str(&ind(d));
            out.push_str("let ");
            if l.mutable {
                out.push_str("mut ");
            }
            out.push_str(&l.name);
            if let Some(t) = &l.ty {
                out.push_str(": ");
                fmt_type(t, out);
            }
            if let Some(init) = &l.init {
                out.push_str(" = ");
                fmt_expr(init, out, d);
            }
            out.push_str(";\n");
        }
        Stmt::Expr(e) => {
            out.push_str(&ind(d));
            fmt_expr(e, out, d);
            out.push_str(";\n");
        }
        Stmt::Return(e) => {
            out.push_str(&ind(d));
            out.push_str("return");
            if let Some(e) = e {
                out.push(' ');
                fmt_expr(e, out, d);
            }
            out.push_str(";\n");
        }
        Stmt::Defer(e) => {
            out.push_str(&ind(d));
            out.push_str("defer ");
            fmt_expr(e, out, d);
            out.push_str(";\n");
        }
        _ => {}
    }
}

fn fmt_type(t: &Type, out: &mut String) {
    match t {
        Type::Named(n, args, _) => {
            out.push_str(n);
            if !args.is_empty() {
                out.push('[');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    fmt_type(a, out);
                }
                out.push(']');
            }
        }
        Type::Ref(x, _) => {
            out.push('&');
            fmt_type(x, out);
        }
        Type::RefMut(x, _) => {
            out.push_str("&mut ");
            fmt_type(x, out);
        }
        _ => out.push_str("i32"),
    }
}

fn fmt_expr(e: &Expr, out: &mut String, d: usize) {
    match e {
        Expr::Lit(Lit::Int(v), _) => out.push_str(&v.to_string()),
        Expr::Lit(Lit::Float(v), _) => out.push_str(&v.to_string()),
        Expr::Lit(Lit::Str(s), _) => {
            out.push('"');
            out.push_str(s);
            out.push('"');
        }
        Expr::Lit(Lit::Bool(b), _) => out.push_str(if *b { "true" } else { "false" }),
        Expr::Path(p, _) => out.push_str(&p.join("::")),
        Expr::Assign { target, value, .. } => {
            fmt_expr(target, out, d);
            out.push_str(" = ");
            fmt_expr(value, out, d);
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            fmt_expr(left, out, d);
            out.push(' ');
            out.push_str(binop_str(*op));
            out.push(' ');
            fmt_expr(right, out, d);
        }
        Expr::Call { callee, args, .. } => {
            fmt_expr(callee, out, d);
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                fmt_expr(a, out, d);
            }
            out.push(')');
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            out.push_str("if ");
            fmt_expr(cond, out, d);
            out.push_str(" {\n");
            fmt_block(then_branch, out, d + 1);
            out.push_str(&ind(d));
            out.push('}');
            if let Some(eb) = else_branch {
                out.push_str(" else ");
                // if eb is block, print braces
                if let Expr::Block(b) = eb.as_ref() {
                    out.push_str("{\n");
                    fmt_block(b, out, d + 1);
                    out.push_str(&ind(d));
                    out.push('}');
                } else {
                    fmt_expr(eb, out, d);
                }
            }
        }
        Expr::While { cond, body, .. } => {
            out.push_str("while ");
            fmt_expr(cond, out, d);
            out.push_str(" {\n");
            fmt_block(body, out, d + 1);
            out.push_str(&ind(d));
            out.push('}');
        }
        Expr::Block(b) => {
            out.push_str("{\n");
            fmt_block(b, out, d + 1);
            out.push_str(&ind(d));
            out.push('}');
        }
        _ => out.push_str("/*expr*/"),
    }
}

fn binop_str(op: BinOp) -> &'static str {
    match op {
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
        _ => "+",
    }
}
