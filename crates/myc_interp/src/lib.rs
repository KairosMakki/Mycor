//! Bytecode interpreter and compile-time evaluation for Mycor.

use myc_ast::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Char(char),
    Unit,
}

pub struct Interpreter {
    functions: HashMap<String, FnDef>,
}

impl Interpreter {
    pub fn new(file: &File) -> Self {
        let mut functions = HashMap::new();
        for item in &file.items {
            if let Item::Fn(f) = item {
                functions.insert(f.name.clone(), f.clone());
            }
        }
        Self { functions }
    }

    pub fn run_main(&mut self) -> Result<(), String> {
        if let Some(f) = self.functions.get("main").cloned() {
            self.call_fn(&f, vec![])?;
            Ok(())
        } else {
            Err("no main".to_string())
        }
    }

    fn call_fn(&mut self, f: &FnDef, args: Vec<Value>) -> Result<Value, String> {
        // bind params
        let mut env = HashMap::new();
        for (p, v) in f.params.iter().zip(args) {
            env.insert(p.name.clone(), v);
        }
        if let Some(body) = &f.body {
            self.eval_block(body, &mut env)
        } else {
            Ok(Value::Unit)
        }
    }

    fn eval_block(&mut self, b: &Block, env: &mut HashMap<String, Value>) -> Result<Value, String> {
        for stmt in &b.stmts {
            match stmt {
                Stmt::Let(l) => {
                    let v = if let Some(init) = &l.init {
                        self.eval_expr(init, env)?
                    } else {
                        Value::Unit
                    };
                    env.insert(l.name.clone(), v);
                }
                Stmt::Expr(e) => {
                    self.eval_expr(e, env)?;
                }
                Stmt::Return(e) => {
                    let v = if let Some(e) = e {
                        self.eval_expr(e, env)?
                    } else {
                        Value::Unit
                    };
                    return Ok(v);
                }
                Stmt::Defer(e) => {
                    self.eval_expr(e, env)?;
                }
                _ => {}
            }
        }
        if let Some(e) = &b.expr {
            self.eval_expr(e, env)
        } else {
            Ok(Value::Unit)
        }
    }

    fn eval_expr(&mut self, e: &Expr, env: &mut HashMap<String, Value>) -> Result<Value, String> {
        match e {
            Expr::Lit(Lit::Int(v), _) => Ok(Value::Int(*v)),
            Expr::Lit(Lit::Float(v), _) => Ok(Value::Float(*v)),
            Expr::Lit(Lit::Str(s), _) => Ok(Value::Str(s.clone())),
            Expr::Lit(Lit::Char(c), _) => Ok(Value::Char(*c)),
            Expr::Lit(Lit::Bool(b), _) => Ok(Value::Bool(*b)),
            Expr::Path(p, _) => {
                if p.len() == 1 {
                    env.get(&p[0])
                        .cloned()
                        .ok_or_else(|| format!("unknown var {}", p[0]))
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let l = self.eval_expr(left, env)?;
                let r = self.eval_expr(right, env)?;
                use myc_ast::BinOp::*;
                match (op, l, r) {
                    (Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    (Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    (Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                    (Div, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                    (Rem, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                    (Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
                    (Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
                    (Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                    (Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                    (Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                    (Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Add, Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
                    (And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    (Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Ok(Value::Unit),
                }
            }
            Expr::Unary { op, operand, .. } => {
                let v = self.eval_expr(operand, env)?;
                match (op, v) {
                    (UnOp::Neg, Value::Int(x)) => Ok(Value::Int(-x)),
                    (UnOp::Not, Value::Bool(x)) => Ok(Value::Bool(!x)),
                    _ => Ok(Value::Unit),
                }
            }
            Expr::Call { callee, args, .. } => {
                let cname = match &**callee {
                    Expr::Path(p, _) if p.len() == 1 => p[0].clone(),
                    _ => String::new(),
                };
                if cname == "println" || cname == "print" {
                    let mut vals = Vec::new();
                    for a in args {
                        vals.push(self.eval_expr(a, env)?);
                    }
                    let s = vals
                        .iter()
                        .map(|v| match v {
                            Value::Int(x) => x.to_string(),
                            Value::Float(x) => x.to_string(),
                            Value::Str(x) => x.clone(),
                            Value::Bool(x) => x.to_string(),
                            Value::Char(x) => x.to_string(),
                            Value::Unit => String::new(),
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("{}", s);
                    return Ok(Value::Unit);
                }
                // user function
                if let Some(f) = self.functions.get(&cname).cloned() {
                    let mut av = Vec::new();
                    for a in args {
                        av.push(self.eval_expr(a, env)?);
                    }
                    // call in new env; need mutable borrow; clone functions ok
                    // We'll evaluate by recursive call_fn with a clone of self? call_fn uses &mut self but only reads functions.
                    // To avoid borrow issues, extract function and call.
                    return self.call_fn(&f, av);
                }
                Ok(Value::Unit)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let c = self.eval_expr(cond, env)?;
                let truth = match c {
                    Value::Bool(b) => b,
                    Value::Int(x) => x != 0,
                    _ => false,
                };
                if truth {
                    self.eval_block(then_branch, env)
                } else if let Some(eb) = else_branch {
                    self.eval_expr(eb, env)
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::While { cond, body, .. } => {
                loop {
                    let c = self.eval_expr(cond, env)?;
                    let truth = match c {
                        Value::Bool(b) => b,
                        Value::Int(x) => x != 0,
                        _ => false,
                    };
                    if !truth {
                        break;
                    }
                    self.eval_block(body, env)?;
                }
                Ok(Value::Unit)
            }
            Expr::For {
                name, iter, body, ..
            } => {
                // iter as range? handle 0..n
                if let Expr::Range { start, end, .. } = iter.as_ref() {
                    let s = start
                        .as_ref()
                        .map(|e| self.eval_expr(e, env))
                        .transpose()?
                        .map(|v| match v {
                            Value::Int(x) => x,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let eend = end
                        .as_ref()
                        .map(|e| self.eval_expr(e, env))
                        .transpose()?
                        .map(|v| match v {
                            Value::Int(x) => x,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let mut i = s;
                    while i < eend {
                        env.insert(name.clone(), Value::Int(i));
                        self.eval_block(body, env)?;
                        i += 1;
                    }
                }
                Ok(Value::Unit)
            }
            Expr::Block(b) => self.eval_block(b, env),
            Expr::Spawn { body, .. } => self.eval_block(body, env),
            Expr::Assign { target, value, .. } => {
                let v = self.eval_expr(value, env)?;
                if let Expr::Path(p, _) = target.as_ref()
                    && p.len() == 1
                {
                    env.insert(p[0].clone(), v);
                }
                Ok(Value::Unit)
            }
            _ => Ok(Value::Unit),
        }
    }
}
