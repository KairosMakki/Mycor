//! Documentation generator for Mycor.

use myc_ast::File;

pub fn summarize(file: &File) -> String {
    let mut out = String::new();
    for item in &file.items {
        match item {
            myc_ast::Item::Fn(f) => {
                out.push_str(&format!("fn {}\n", f.name));
            }
            myc_ast::Item::Struct(s) => {
                out.push_str(&format!("struct {}\n", s.name));
            }
            _ => {}
        }
    }
    out
}
