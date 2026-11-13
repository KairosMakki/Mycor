//! Pipeline orchestration, toolchain detection, and linking for Mycor.

use myc_ast::File;
use myc_lexer::Lexer;
use myc_parser::Parser;
use myc_span::SourceMap;
use myc_types::check_file;
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
    // merge diagnostics
    let _ = (lex_diags, parse_diags);
    (file, sm, checker)
}

pub fn check_source(source: &str, file_name: &str) -> Vec<myc_diagnostics::Diagnostic> {
    let (file, _sm, checker) = compile_source(source, file_name);
    let mut diags = checker.diagnostics.clone();
    // borrow check
    let bc = myc_borrowck::check(&file, &checker.type_map);
    diags.extend(bc);
    diags
}

pub fn build_executable(source: &str, file_name: &str, out_path: &Path) -> Result<PathBuf, String> {
    let (file, _sm, checker) = compile_source(source, file_name);
    let mut diags = checker.diagnostics.clone();
    let bc = myc_borrowck::check(&file, &checker.type_map);
    diags.extend(bc);
    if !diags.is_empty() {
        return Err(format!("{} diagnostic(s)", diags.len()));
    }
    let c_src = myc_codegen_c::generate(&file, &checker.type_map);
    // write c file next to output
    let c_path = out_path.with_extension("c");
    fs::write(&c_path, &c_src).map_err(|e| e.to_string())?;
    // find compiler
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
    // MSVC cl
    if Command::new("cl").arg("/?").status().is_ok() {
        return Some("cl".to_string());
    }
    None
}

pub fn run_executable(path: &Path) -> Result<i32, String> {
    let status = Command::new(path).status().map_err(|e| e.to_string())?;
    Ok(status.code().unwrap_or(0))
}
