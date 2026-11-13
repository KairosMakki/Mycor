//! Source spans, positions, file ids, and string interning.

use std::collections::HashMap;

/// A handle to a source file in a [`SourceMap`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// A half-open byte range inside a single source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Self { file, start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        debug_assert_eq!(self.file, other.file);
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// A line and column position (1-based).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Owns source text for one or more files and answers position queries.
#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

struct SourceFile {
    name: String,
    text: String,
    line_starts: Vec<u32>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        let text = text.into();
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile {
            name: name.into(),
            text,
            line_starts,
        });
        id
    }

    pub fn file_name(&self, id: FileId) -> &str {
        &self.files[id.0 as usize].name
    }

    pub fn source(&self, id: FileId) -> &str {
        &self.files[id.0 as usize].text
    }

    pub fn position(&self, span: Span) -> Position {
        let f = &self.files[span.file.0 as usize];
        let idx = f.line_starts.partition_point(|&s| s <= span.start);
        let line = idx.saturating_sub(1) as u32 + 1;
        let line_start = f.line_starts[line as usize - 1];
        Position {
            line,
            column: span.start - line_start + 1,
        }
    }

    pub fn line_text(&self, file: FileId, line: u32) -> &str {
        let f = &self.files[file.0 as usize];
        if line == 0 || line as usize > f.line_starts.len() {
            return "";
        }
        let start = f.line_starts[line as usize - 1] as usize;
        let end = f
            .line_starts
            .get(line as usize)
            .copied()
            .unwrap_or(f.text.len() as u32) as usize;
        let mut s = &f.text[start..end];
        if s.ends_with('\n') {
            s = &s[..s.len() - 1];
        }
        s
    }
}

/// A simple string interner.
#[derive(Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    strings: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }

    pub fn resolve(&self, id: u32) -> &str {
        &self.strings[id as usize]
    }
}
