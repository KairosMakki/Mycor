//! Diagnostics: error codes, labels, suggestions, rendering, JSON output.

use myc_span::{SourceMap, Span};
use std::fmt;

/// Severity of a diagnostic.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
            Severity::Help => write!(f, "help"),
        }
    }
}

/// A label pointing at a span.
#[derive(Clone, Debug)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub is_primary: bool,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            is_primary: true,
        }
    }
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            is_primary: false,
        }
    }
}

/// A machine-applicable suggestion.
#[derive(Clone, Debug)]
pub struct Suggestion {
    pub span: Span,
    pub replacement: String,
    pub message: String,
    pub machine_applicable: bool,
}

/// A diagnostic.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        let mut d = Self::error(code, message);
        d.severity = Severity::Warning;
        d
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_suggestion(mut self, s: Suggestion) -> Self {
        self.suggestions.push(s);
        self
    }
}

/// Render a diagnostic to a string.
pub fn render(diag: &Diagnostic, src: &SourceMap, color: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let sev = diag.severity;
    let prefix = if color {
        match sev {
            Severity::Error => "\x1b[31merror\x1b[0m",
            Severity::Warning => "\x1b[33mwarning\x1b[0m",
            Severity::Note => "\x1b[34mnote\x1b[0m",
            Severity::Help => "\x1b[32mhelp\x1b[0m",
        }
    } else {
        match sev {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    };
    let _ = write!(out, "{}[{}]: {}", prefix, diag.code, diag.message);
    if let Some(l) = diag.labels.first() {
        let pos = src.position(l.span);
        let _ = write!(
            out,
            "\n  --> {}:{}:{}",
            src.file_name(l.span.file),
            pos.line,
            pos.column
        );
        let line = src.line_text(l.span.file, pos.line);
        let _ = write!(out, "\n   | {}", line);
        let mut caret = String::new();
        let col = (pos.column as usize).saturating_sub(1);
        for _ in 0..col {
            caret.push(' ');
        }
        let len = (l.span.end - l.span.start).max(1) as usize;
        for _ in 0..len {
            caret.push('^');
        }
        let _ = write!(out, "\n   | {}", caret);
        if !l.message.is_empty() {
            let _ = write!(out, " {}", l.message);
        }
    }
    for n in &diag.notes {
        let _ = write!(out, "\nnote: {}", n);
    }
    for s in &diag.suggestions {
        let _ = write!(out, "\nhelp: {} -> {}", s.message, s.replacement);
    }
    out
}

/// Render as JSON (one object).
pub fn render_json(diag: &Diagnostic, src: &SourceMap) -> String {
    let primary = diag.labels.first();
    let (file, line, col) = match primary {
        Some(l) => {
            let p = src.position(l.span);
            (src.file_name(l.span.file).to_string(), p.line, p.column)
        }
        None => (String::new(), 0, 0),
    };
    format!(
        "{{\"code\":\"{}\",\"severity\":\"{}\",\"message\":\"{}\",\"file\":\"{}\",\"line\":{},\"column\":{}}}",
        diag.code,
        diag.severity,
        escape_json(&diag.message),
        escape_json(&file),
        line,
        col
    )
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Error code registry. Keep this list in sync with docs/errors/.
// ---------------------------------------------------------------------------

pub mod codes {
    // Syntax
    pub const E0001: &str = "E0001"; // unexpected token
    pub const E0002: &str = "E0002"; // expected expression
    pub const E0003: &str = "E0003"; // expected type
    pub const E0004: &str = "E0004"; // expected identifier
    pub const E0005: &str = "E0005"; // expected ';'
    pub const E0006: &str = "E0006"; // expected '{'
    pub const E0007: &str = "E0007"; // expected '}'
    pub const E0008: &str = "E0008"; // expected ')'
    pub const E0009: &str = "E0009"; // expected ']'
    pub const E0010: &str = "E0010"; // expected '>'
    pub const E0011: &str = "E0011"; // invalid literal
    pub const E0012: &str = "E0012"; // unterminated string
    pub const E0013: &str = "E0013"; // unterminated comment
    pub const E0014: &str = "E0014"; // invalid character
    pub const E0015: &str = "E0015"; // expected pattern
    // Names
    pub const E0100: &str = "E0100"; // unknown name
    pub const E0101: &str = "E0101"; // duplicate definition
    pub const E0102: &str = "E0102"; // private item
    pub const E0103: &str = "E0103"; // import not found
    pub const E0104: &str = "E0104"; // ambiguous import
    pub const E0105: &str = "E0105"; // unused import
    pub const E0106: &str = "E0106"; // did you mean
    // Types
    pub const E0200: &str = "E0200"; // type mismatch
    pub const E0201: &str = "E0201"; // cannot infer type
    pub const E0202: &str = "E0202"; // wrong number of type arguments
    pub const E0203: &str = "E0203"; // wrong number of arguments
    pub const E0204: &str = "E0204"; // missing trait implementation
    pub const E0205: &str = "E0205"; // recursive type without indirection
    pub const E0206: &str = "E0206"; // invalid cast
    pub const E0207: &str = "E0207"; // missing match arm
    pub const E0208: &str = "E0208"; // non-exhaustive match
    pub const E0209: &str = "E0209"; // unreachable pattern
    pub const E0210: &str = "E0210"; // uninitialized variable
    // Ownership and borrows
    pub const E0300: &str = "E0300"; // use after move
    pub const E0301: &str = "E0301"; // double mutable borrow
    pub const E0302: &str = "E0302"; // shared and mutable borrow conflict
    pub const E0303: &str = "E0303"; // dangling reference
    pub const E0304: &str = "E0304"; // returning reference to local
    pub const E0305: &str = "E0305"; // cannot move out of borrowed
    pub const E0306: &str = "E0306"; // borrow lives too long
    pub const E0307: &str = "E0307"; // mutable borrow of immutable
    // Unsafe
    pub const E0400: &str = "E0400"; // unsafe operation outside unsafe block
    pub const E0401: &str = "E0401"; // dereference of raw pointer outside unsafe
    // Concurrency
    pub const E0500: &str = "E0500"; // data race
    pub const E0501: &str = "E0501"; // non-sendable value sent to thread
    pub const E0502: &str = "E0502"; // non-sync shared access
    // Constants and overflow
    pub const E0600: &str = "E0600"; // integer overflow in constant
    pub const E0601: &str = "E0601"; // division by zero in constant
    // Forward declarations
    pub const E0700: &str = "E0700"; // forward declaration mismatch
    pub const E0701: &str = "E0701"; // conflicting definitions
    // Warnings
    pub const W0001: &str = "W0001"; // unused variable
    pub const W0002: &str = "W0002"; // unreachable code
    pub const W0003: &str = "W0003"; // unused import
}

pub fn all_codes() -> Vec<&'static str> {
    vec![
        codes::E0001,
        codes::E0002,
        codes::E0003,
        codes::E0004,
        codes::E0005,
        codes::E0006,
        codes::E0007,
        codes::E0008,
        codes::E0009,
        codes::E0010,
        codes::E0011,
        codes::E0012,
        codes::E0013,
        codes::E0014,
        codes::E0015,
        codes::E0100,
        codes::E0101,
        codes::E0102,
        codes::E0103,
        codes::E0104,
        codes::E0105,
        codes::E0106,
        codes::E0200,
        codes::E0201,
        codes::E0202,
        codes::E0203,
        codes::E0204,
        codes::E0205,
        codes::E0206,
        codes::E0207,
        codes::E0208,
        codes::E0209,
        codes::E0210,
        codes::E0300,
        codes::E0301,
        codes::E0302,
        codes::E0303,
        codes::E0304,
        codes::E0305,
        codes::E0306,
        codes::E0307,
        codes::E0400,
        codes::E0401,
        codes::E0500,
        codes::E0501,
        codes::E0502,
        codes::E0600,
        codes::E0601,
        codes::E0700,
        codes::E0701,
        codes::W0001,
        codes::W0002,
        codes::W0003,
    ]
}
