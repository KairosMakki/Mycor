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
    pub const E0016: &str = "E0016"; // expected item
    pub const E0017: &str = "E0017"; // expected block
    pub const E0018: &str = "E0018"; // expected type name
    pub const E0019: &str = "E0019"; // expected field
    pub const E0020: &str = "E0020"; // expected variant
    pub const E0021: &str = "E0021"; // expected parameter
    pub const E0022: &str = "E0022"; // expected generic
    pub const E0023: &str = "E0023"; // expected literal
    pub const E0024: &str = "E0024"; // expected path
    pub const E0025: &str = "E0025"; // invalid escape
    pub const E0026: &str = "E0026"; // invalid number
    pub const E0027: &str = "E0027"; // invalid float
    pub const E0028: &str = "E0028"; // unterminated char
    pub const E0029: &str = "E0029"; // invalid utf8
    pub const E0030: &str = "E0030"; // reserved keyword
    // Names
    pub const E0100: &str = "E0100"; // unknown name
    pub const E0101: &str = "E0101"; // duplicate definition
    pub const E0102: &str = "E0102"; // private item
    pub const E0103: &str = "E0103"; // import not found
    pub const E0104: &str = "E0104"; // ambiguous import
    pub const E0105: &str = "E0105"; // unused import
    pub const E0106: &str = "E0106"; // did you mean`n    pub const E0107: &str = "E0107"; // unknown module`n    pub const E0108: &str = "E0108"; // unknown field`n    pub const E0109: &str = "E0109"; // unknown method`n    pub const E0110: &str = "E0110"; // unknown variant
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
    pub const E0210: &str = "E0210"; // uninitialized variable`n    pub const E0211: &str = "E0211"; // cannot call non-function`n    pub const E0212: &str = "E0212"; // cannot index non-array`n    pub const E0213: &str = "E0213"; // cannot field access non-struct`n    pub const E0214: &str = "E0214"; // duplicate field`n    pub const E0215: &str = "E0215"; // missing field
    // Ownership and borrows
    pub const E0300: &str = "E0300"; // use after move
    pub const E0301: &str = "E0301"; // double mutable borrow
    pub const E0302: &str = "E0302"; // shared and mutable borrow conflict
    pub const E0303: &str = "E0303"; // dangling reference
    pub const E0304: &str = "E0304"; // returning reference to local
    pub const E0305: &str = "E0305"; // cannot move out of borrowed
    pub const E0306: &str = "E0306"; // borrow lives too long
    pub const E0307: &str = "E0307"; // mutable borrow of immutable`n    pub const E0308: &str = "E0308"; // borrow outlives value`n    pub const E0309: &str = "E0309"; // cannot move out of index`n    pub const E0310: &str = "E0310"; // cannot borrow packed field
    // Unsafe
    pub const E0400: &str = "E0400"; // unsafe operation outside unsafe block
    pub const E0401: &str = "E0401"; // dereference of raw pointer outside unsafe`n    pub const E0402: &str = "E0402"; // unsafe block not allowed`n    pub const E0403: &str = "E0403"; // invalid unsafe operation
    // Concurrency
    pub const E0500: &str = "E0500"; // data race
    pub const E0501: &str = "E0501"; // non-sendable value sent to thread
    pub const E0502: &str = "E0502"; // non-sync shared access`n    pub const E0503: &str = "E0503"; // channel type mismatch`n    pub const E0504: &str = "E0504"; // thread panicked
    // Constants and overflow
    pub const E0600: &str = "E0600"; // integer overflow in constant
    pub const E0601: &str = "E0601"; // division by zero in constant`n    pub const E0602: &str = "E0602"; // constant eval failed`n    pub const E0603: &str = "E0603"; // non-const in const context
    // Forward declarations
    pub const E0700: &str = "E0700"; // forward declaration mismatch
    pub const E0701: &str = "E0701"; // conflicting definitions`n    pub const E0702: &str = "E0702"; // forward decl not found`n    pub const E0703: &str = "E0703"; // signature mismatch
    // Warnings
    pub const W0001: &str = "W0001"; // unused variable
    pub const W0002: &str = "W0002"; // unreachable code
    pub const W0003: &str = "W0003"; // unused import
}

pub fn all_codes() -> Vec<&'static str> {
    // Auto-generated list of all registered codes.
    let mut v = Vec::new();
    // Syntax
    for c in [
        "E0001", "E0002", "E0003", "E0004", "E0005", "E0006", "E0007", "E0008", "E0009", "E0010",
        "E0011", "E0012", "E0013", "E0014", "E0015", "E0016", "E0017", "E0018", "E0019", "E0020",
        "E0021", "E0022", "E0023", "E0024", "E0025", "E0026", "E0027", "E0028", "E0029", "E0030",
    ] {
        v.push(c);
    }
    // Names
    for c in [
        "E0100", "E0101", "E0102", "E0103", "E0104", "E0105", "E0106", "E0107", "E0108", "E0109",
        "E0110",
    ] {
        v.push(c);
    }
    // Types
    for c in [
        "E0200", "E0201", "E0202", "E0203", "E0204", "E0205", "E0206", "E0207", "E0208", "E0209",
        "E0210", "E0211", "E0212", "E0213", "E0214", "E0215",
    ] {
        v.push(c);
    }
    // Ownership
    for c in [
        "E0300", "E0301", "E0302", "E0303", "E0304", "E0305", "E0306", "E0307", "E0308", "E0309",
        "E0310",
    ] {
        v.push(c);
    }
    // Unsafe
    for c in ["E0400", "E0401", "E0402", "E0403"] {
        v.push(c);
    }
    // Concurrency
    for c in ["E0500", "E0501", "E0502", "E0503", "E0504"] {
        v.push(c);
    }
    // Constants
    for c in ["E0600", "E0601", "E0602", "E0603"] {
        v.push(c);
    }
    // Forward decls
    for c in ["E0700", "E0701", "E0702", "E0703"] {
        v.push(c);
    }
    // Warnings
    for c in ["W0001", "W0002", "W0003"] {
        v.push(c);
    }
    v
}
