//! Lexer for Mycor.

use myc_diagnostics::{Diagnostic, codes};
use myc_span::{FileId, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // literals
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),
    // identifiers / keywords
    Ident,
    // keywords
    Fn,
    Let,
    Mut,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Const,
    Static,
    If,
    Else,
    While,
    For,
    Loop,
    Match,
    Return,
    Break,
    Continue,
    Defer,
    Unsafe,
    Mod,
    Use,
    Pub,
    As,
    In,
    Where,
    // punctuation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    AmpAmp,
    Pipe,
    PipePipe,
    Caret,
    Bang,
    Tilde,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Arrow,
    FatArrow,
    Colon,
    ColonColon,
    Semicolon,
    Comma,
    Dot,
    DotDot,
    DotDotEq,
    Question,
    At,
    Hash,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    // special
    Eof,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

pub struct Lexer<'a> {
    src: &'a str,
    file: FileId,
    pos: usize,
    bytes: &'a [u8],
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, file: FileId) -> Self {
        Self {
            src,
            file,
            pos: 0,
            bytes: src.as_bytes(),
            diagnostics: Vec::new(),
        }
    }

    pub fn lex(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut toks = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.pos >= self.bytes.len() {
                toks.push(Token {
                    kind: TokenKind::Eof,
                    span: Span::new(self.file, self.pos as u32, self.pos as u32),
                    text: String::new(),
                });
                break;
            }
            let start = self.pos;
            let _ = self.bytes[self.pos];
            let tok = self.next_token(start);
            toks.push(tok);
        }
        (toks, self.diagnostics)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            // whitespace
            while self.pos < self.bytes.len() && (self.bytes[self.pos] as char).is_whitespace() {
                self.pos += 1;
            }
            // line comment
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'/'
            {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            // block comment
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'*'
            {
                let start = self.pos;
                self.pos += 2;
                let mut closed = false;
                while self.pos + 1 < self.bytes.len() {
                    if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                        self.pos += 2;
                        closed = true;
                        break;
                    }
                    self.pos += 1;
                }
                if !closed {
                    self.diag(codes::E0013, "unterminated block comment", start, self.pos);
                }
                continue;
            }
            break;
        }
    }

    fn next_token(&mut self, start: usize) -> Token {
        let b = self.bytes[self.pos];
        // single char tokens etc.
        // two-char
        let two = if self.pos + 1 < self.bytes.len() {
            Some((self.bytes[self.pos], self.bytes[self.pos + 1]))
        } else {
            None
        };
        match b {
            b'+' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Plus,
                    span: sp(self.file, start, self.pos),
                    text: "+".into(),
                }
            }
            b'-' => {
                if matches!(two, Some((b'-', b'>'))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::Arrow,
                        span: sp(self.file, start, self.pos),
                        text: "->".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Minus,
                        span: sp(self.file, start, self.pos),
                        text: "-".into(),
                    }
                }
            }
            b'*' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Star,
                    span: sp(self.file, start, self.pos),
                    text: "*".into(),
                }
            }
            b'/' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Slash,
                    span: sp(self.file, start, self.pos),
                    text: "/".into(),
                }
            }
            b'%' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Percent,
                    span: sp(self.file, start, self.pos),
                    text: "%".into(),
                }
            }
            b'&' => {
                if matches!(two, Some((b'&', b'&'))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::AmpAmp,
                        span: sp(self.file, start, self.pos),
                        text: "&&".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Amp,
                        span: sp(self.file, start, self.pos),
                        text: "&".into(),
                    }
                }
            }
            b'|' => {
                if matches!(two, Some((b'|', b'|'))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::PipePipe,
                        span: sp(self.file, start, self.pos),
                        text: "||".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Pipe,
                        span: sp(self.file, start, self.pos),
                        text: "|".into(),
                    }
                }
            }
            b'^' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Caret,
                    span: sp(self.file, start, self.pos),
                    text: "^".into(),
                }
            }
            b'!' => {
                if matches!(two, Some((b'!', b'='))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::NotEq,
                        span: sp(self.file, start, self.pos),
                        text: "!=".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Bang,
                        span: sp(self.file, start, self.pos),
                        text: "!".into(),
                    }
                }
            }
            b'~' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Tilde,
                    span: sp(self.file, start, self.pos),
                    text: "~".into(),
                }
            }
            b'=' => {
                if matches!(two, Some((b'=', b'='))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::EqEq,
                        span: sp(self.file, start, self.pos),
                        text: "==".into(),
                    }
                } else if matches!(two, Some((b'=', b'>'))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::FatArrow,
                        span: sp(self.file, start, self.pos),
                        text: "=>".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Eq,
                        span: sp(self.file, start, self.pos),
                        text: "=".into(),
                    }
                }
            }
            b'<' => {
                if matches!(two, Some((b'<', b'='))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::Le,
                        span: sp(self.file, start, self.pos),
                        text: "<=".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Lt,
                        span: sp(self.file, start, self.pos),
                        text: "<".into(),
                    }
                }
            }
            b'>' => {
                if matches!(two, Some((b'>', b'='))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::Ge,
                        span: sp(self.file, start, self.pos),
                        text: ">=".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Gt,
                        span: sp(self.file, start, self.pos),
                        text: ">".into(),
                    }
                }
            }
            b':' => {
                if matches!(two, Some((b':', b':'))) {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::ColonColon,
                        span: sp(self.file, start, self.pos),
                        text: "::".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Colon,
                        span: sp(self.file, start, self.pos),
                        text: ":".into(),
                    }
                }
            }
            b';' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Semicolon,
                    span: sp(self.file, start, self.pos),
                    text: ";".into(),
                }
            }
            b',' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Comma,
                    span: sp(self.file, start, self.pos),
                    text: ",".into(),
                }
            }
            b'.' => {
                if self.pos + 2 < self.bytes.len()
                    && self.bytes[self.pos + 1] == b'.'
                    && self.bytes[self.pos + 2] == b'='
                {
                    self.pos += 3;
                    Token {
                        kind: TokenKind::DotDotEq,
                        span: sp(self.file, start, self.pos),
                        text: "..=".into(),
                    }
                } else if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'.' {
                    self.pos += 2;
                    Token {
                        kind: TokenKind::DotDot,
                        span: sp(self.file, start, self.pos),
                        text: "..".into(),
                    }
                } else {
                    self.pos += 1;
                    Token {
                        kind: TokenKind::Dot,
                        span: sp(self.file, start, self.pos),
                        text: ".".into(),
                    }
                }
            }
            b'?' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Question,
                    span: sp(self.file, start, self.pos),
                    text: "?".into(),
                }
            }
            b'@' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::At,
                    span: sp(self.file, start, self.pos),
                    text: "@".into(),
                }
            }
            b'#' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::Hash,
                    span: sp(self.file, start, self.pos),
                    text: "#".into(),
                }
            }
            b'(' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::LParen,
                    span: sp(self.file, start, self.pos),
                    text: "(".into(),
                }
            }
            b')' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::RParen,
                    span: sp(self.file, start, self.pos),
                    text: ")".into(),
                }
            }
            b'{' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::LBrace,
                    span: sp(self.file, start, self.pos),
                    text: "{".into(),
                }
            }
            b'}' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::RBrace,
                    span: sp(self.file, start, self.pos),
                    text: "}".into(),
                }
            }
            b'[' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::LBracket,
                    span: sp(self.file, start, self.pos),
                    text: "[".into(),
                }
            }
            b']' => {
                self.pos += 1;
                Token {
                    kind: TokenKind::RBracket,
                    span: sp(self.file, start, self.pos),
                    text: "]".into(),
                }
            }
            b'"' => self.lex_string(start),
            b'\'' => self.lex_char(start),
            _ if (b as char).is_ascii_digit() => self.lex_number(start),
            _ if is_ident_start(b) => self.lex_ident(start),
            _ => {
                self.pos += 1;
                self.diag(codes::E0014, "invalid character", start, self.pos);
                Token {
                    kind: TokenKind::Unknown,
                    span: sp(self.file, start, self.pos),
                    text: String::from_utf8_lossy(&self.bytes[start..self.pos]).into(),
                }
            }
        }
    }

    fn lex_string(&mut self, start: usize) -> Token {
        self.pos += 1; // opening quote
        let mut value = String::new();
        let mut closed = false;
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos];
            if c == b'"' {
                closed = true;
                self.pos += 1;
                break;
            }
            if c == b'\\' {
                self.pos += 1;
                if self.pos < self.bytes.len() {
                    let e = self.bytes[self.pos];
                    match e {
                        b'n' => value.push('\n'),
                        b't' => value.push('\t'),
                        b'r' => value.push('\r'),
                        b'\\' => value.push('\\'),
                        b'"' => value.push('"'),
                        b'0' => value.push('\0'),
                        _ => {
                            value.push(e as char);
                        }
                    }
                    self.pos += 1;
                }
            } else {
                // utf-8 char
                let ch = self.src[self.pos..].chars().next().unwrap();
                value.push(ch);
                self.pos += ch.len_utf8();
            }
        }
        if !closed {
            self.diag(codes::E0012, "unterminated string literal", start, self.pos);
        }
        Token {
            kind: TokenKind::Str(value),
            span: sp(self.file, start, self.pos),
            text: self.src[start..self.pos].to_string(),
        }
    }

    fn lex_char(&mut self, start: usize) -> Token {
        self.pos += 1;
        let mut value = '\0';
        if self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'\\' {
                self.pos += 1;
                if self.pos < self.bytes.len() {
                    let e = self.bytes[self.pos];
                    value = match e {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'\\' => '\\',
                        b'\'' => '\'',
                        b'0' => '\0',
                        _ => e as char,
                    };
                    self.pos += 1;
                }
            } else {
                let ch = self.src[self.pos..].chars().next().unwrap();
                value = ch;
                self.pos += ch.len_utf8();
            }
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'\'' {
            self.pos += 1;
        } else {
            self.diag(codes::E0011, "invalid char literal", start, self.pos);
        }
        Token {
            kind: TokenKind::Char(value),
            span: sp(self.file, start, self.pos),
            text: self.src[start..self.pos].to_string(),
        }
    }

    fn lex_number(&mut self, start: usize) -> Token {
        // integer or float
        let mut has_dot = false;
        let mut has_exp = false;
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos];
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == b'.' && !has_dot && !has_exp {
                // peek next is digit
                if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1].is_ascii_digit() {
                    has_dot = true;
                    self.pos += 1;
                } else {
                    break;
                }
            } else if (c == b'e' || c == b'E') && !has_exp {
                has_exp = true;
                self.pos += 1;
                if self.pos < self.bytes.len()
                    && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-')
                {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let text = &self.src[start..self.pos];
        if has_dot || has_exp {
            let v: f64 = text.parse().unwrap_or(0.0);
            Token {
                kind: TokenKind::Float(v),
                span: sp(self.file, start, self.pos),
                text: text.to_string(),
            }
        } else {
            // handle suffix? no
            let v: i64 = text.parse().unwrap_or(0);
            Token {
                kind: TokenKind::Int(v),
                span: sp(self.file, start, self.pos),
                text: text.to_string(),
            }
        }
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while self.pos < self.bytes.len() && is_ident_cont(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let text = &self.src[start..self.pos];
        let kind = keyword(text).unwrap_or(TokenKind::Ident);
        // bool literals
        let kind = if text == "true" {
            TokenKind::Bool(true)
        } else if text == "false" {
            TokenKind::Bool(false)
        } else {
            kind
        };
        Token {
            kind,
            span: sp(self.file, start, self.pos),
            text: text.to_string(),
        }
    }

    fn diag(&mut self, code: &'static str, msg: &str, start: usize, end: usize) {
        self.diagnostics
            .push(Diagnostic::error(code, msg.to_string()).with_label(
                myc_diagnostics::Label::primary(
                    Span::new(self.file, start as u32, end as u32),
                    msg.to_string(),
                ),
            ));
    }
}

fn sp(file: FileId, start: usize, end: usize) -> Span {
    Span::new(file, start as u32, end as u32)
}

fn is_ident_start(b: u8) -> bool {
    (b as char).is_alphabetic() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    let c = b as char;
    c.is_alphanumeric() || b == b'_'
}
fn keyword(s: &str) -> Option<TokenKind> {
    Some(match s {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "trait" => TokenKind::Trait,
        "impl" => TokenKind::Impl,
        "type" => TokenKind::Type,
        "const" => TokenKind::Const,
        "static" => TokenKind::Static,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "loop" => TokenKind::Loop,
        "match" => TokenKind::Match,
        "return" => TokenKind::Return,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "defer" => TokenKind::Defer,
        "unsafe" => TokenKind::Unsafe,
        "mod" => TokenKind::Mod,
        "use" => TokenKind::Use,
        "pub" => TokenKind::Pub,
        "as" => TokenKind::As,
        "in" => TokenKind::In,
        "where" => TokenKind::Where,
        _ => return None,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_simple() {
        let mut sm = myc_span::SourceMap::new();
        let fid = sm.add_file("t.myc", "fn main() { let x = 5; }");
        let (toks, diags) = Lexer::new(sm.source(fid), fid).lex();
        assert!(diags.is_empty(), "{:?}", diags);
        assert!(toks.len() > 5);
    }
}
