//! Pipeline orchestration, toolchain detection, and linking for Mycor.

use myc_ast::{File, Item};
use myc_lexer::Lexer;
use myc_parser::Parser;
use myc_span::SourceMap;
use myc_types::check_file;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct CompileResult {
    pub c_source: String,
    pub diagnostics: Vec<myc_diagnostics::Diagnostic>,
    pub output_exe: Option<PathBuf>,
}

pub fn compile_source(source: &str, file_name: &str) -> (File, SourceMap, myc_types::Checker) {
    let mut sm = SourceMap::new();
    let fid = sm.add_file(file_name, source);
    let src = sm.source(fid).to_string();
    let (toks, lex_diags) = Lexer::new(&src, fid).lex();
    let (file, parse_diags) = Parser::new(toks, fid).parse_file();
    let (checker, _) = check_file(&file);
    let mut checker = checker;
    checker.diagnostics.extend(lex_diags);
    checker.diagnostics.extend(parse_diags);
    (file, sm, checker)
}

pub fn compile_project(entry: &Path) -> (File, SourceMap, myc_types::Checker) {
    let mut sm = SourceMap::new();
    let mut items = Vec::new();
    let mut diags = Vec::new();
    let mut visited = HashSet::new();
    load_module(entry, &mut sm, &mut items, &mut diags, &mut visited);

    let span = if items.is_empty() {
        myc_span::Span::new(myc_span::FileId(0), 0, 0)
    } else {
        item_span(&items[0])
    };
    let file = File { items, span };

    let (checker, _) = check_file(&file);
    let mut checker = checker;
    checker.diagnostics.extend(diags);
    (file, sm, checker)
}

fn item_span(item: &Item) -> myc_span::Span {
    match item {
        Item::Fn(f) => f.span,
        Item::Struct(s) => s.span,
        Item::Enum(e) => e.span,
        Item::Const(c) => c.span,
        Item::Static(s) => s.span,
        Item::TypeAlias(t) => t.span,
        Item::Mod(m) => m.span,
        Item::Use(u) => u.span,
        Item::Impl(i) => i.span,
        Item::Trait(t) => t.span,
    }
}

fn load_module(
    path: &Path,
    sm: &mut SourceMap,
    items: &mut Vec<Item>,
    diags: &mut Vec<myc_diagnostics::Diagnostic>,
    visited: &mut HashSet<PathBuf>,
) {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon) {
        return;
    }
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            diags.push(myc_diagnostics::Diagnostic::error(
                "E0107",
                format!("could not read module file {}: {}", path.display(), e),
            ));
            return;
        }
    };
    let fid = sm.add_file(path.to_string_lossy().to_string(), &source);
    let src = sm.source(fid).to_string();
    let (toks, lex_diags) = Lexer::new(&src, fid).lex();
    let (file, parse_diags) = Parser::new(toks, fid).parse_file();
    diags.extend(lex_diags);
    diags.extend(parse_diags);

    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    for item in file.items {
        match &item {
            Item::Mod(m) => {
                if let Some(child) = resolve_mod(base_dir, &m.name) {
                    load_module(&child, sm, items, diags, visited);
                } else {
                    diags.push(myc_diagnostics::Diagnostic::error(
                        "E0107",
                        format!("unknown module `{}`", m.name),
                    ));
                }
            }
            _ => items.push(item),
        }
    }
}

fn resolve_mod(base: &Path, name: &str) -> Option<PathBuf> {
    let candidates = [
        base.join(format!("{}.myc", name)),
        base.join(name).join("mod.myc"),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }
    None
}

pub fn check_source(source: &str, file_name: &str) -> Vec<myc_diagnostics::Diagnostic> {
    let (file, _sm, checker) = compile_source(source, file_name);
    let mut diags = checker.diagnostics.clone();
    let bc = myc_borrowck::check(&file, &checker.type_map);
    diags.extend(bc);
    diags
}

pub fn check_project(entry: &Path) -> Vec<myc_diagnostics::Diagnostic> {
    let (file, _sm, checker) = compile_project(entry);
    let mut diags = checker.diagnostics.clone();
    let bc = myc_borrowck::check(&file, &checker.type_map);
    diags.extend(bc);
    diags
}

pub fn build_executable(source: &str, file_name: &str, out_path: &Path) -> Result<PathBuf, String> {
    let (file, _sm, checker) = compile_source(source, file_name);
    build_from(file, checker, out_path)
}

pub fn build_project(entry: &Path, out_path: &Path) -> Result<PathBuf, String> {
    let (file, _sm, checker) = compile_project(entry);
    build_from(file, checker, out_path)
}

fn build_from(file: myc_ast::File, checker: myc_types::Checker, out_path: &Path) -> Result<PathBuf, String> {
    let mut diags = checker.diagnostics.clone();
    let bc = myc_borrowck::check(&file, &checker.type_map);
    diags.extend(bc);
    if !diags.is_empty() {
        return Err(format!("{} diagnostic(s)", diags.len()));
    }
    let c_src = myc_codegen_c::generate(&file, &checker.type_map);
    let c_path = out_path.with_extension("c");
    fs::write(&c_path, &c_src).map_err(|e| e.to_string())?;
    let compiler = find_c_compiler().ok_or_else(|| "no C compiler found".to_string())?;
    let status = Command::new(&compiler)
        .arg(&c_path)
        .arg("-o")
        .arg(out_path)
        .arg("-std=c11")
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("C compilation failed".to_string());
    }
    Ok(out_path.to_path_buf())
}

pub fn find_c_compiler() -> Option<String> {
    for c in ["clang", "gcc", "cc", "tcc"] {
        if Command::new(c)
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(c.to_string());
        }
    }
    if Command::new("cl").arg("/?").status().is_ok() {
        return Some("cl".to_string());
    }
    None
}

pub fn run_executable(path: &Path) -> Result<i32, String> {
    let status = Command::new(path).status().map_err(|e| e.to_string())?;
    Ok(status.code().unwrap_or(0))
}
