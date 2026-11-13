//! Parser with error recovery.

use myc_ast::*;
use myc_diagnostics::{Diagnostic, Label, codes};
use myc_lexer::Token;
use myc_span::{FileId, Span};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: FileId,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: FileId) -> Self {
        Self {
            tokens,
            pos: 0,
            file,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse_file(mut self) -> (File, Vec<Diagnostic>) {
        let start_span = self.peek_span();
        let mut items = Vec::new();
        while !self.at_eof() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                // skip
                self.advance();
            }
        }
        let span = if items.is_empty() {
            start_span
        } else {
            item_span(&items[0]).merge(item_span(items.last().unwrap()))
        };
        let file = File { items, span };
        (file, self.diagnostics)
    }

    // helpers
    fn at_eof(&self) -> bool {
        self.peek()
            .map(|t| matches!(t.kind, myc_lexer::TokenKind::Eof))
            .unwrap_or(true)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<myc_lexer::TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    fn peek_span(&self) -> Span {
        self.peek()
            .map(|t| t.span)
            .unwrap_or(Span::new(self.file, 0, 0))
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect(&mut self, pred: fn(&myc_lexer::TokenKind) -> bool, msg: &str) -> Option<Token> {
        if let Some(t) = self.peek() && pred(&t.kind) {
            return self.advance();
        }
        let span = self.peek_span();
        self.diag(codes::E0001, msg, span);
        None
    }

    fn diag(&mut self, code: &'static str, msg: &str, span: Span) {
        self.diagnostics.push(
            Diagnostic::error(code, msg.to_string())
                .with_label(Label::primary(span, msg.to_string())),
        );
    }

    fn is(&self, k: &myc_lexer::TokenKind) -> bool {
        self.peek()
            .map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(k))
            .unwrap_or(false)
    }

    // Top-level items
    fn parse_item(&mut self) -> Option<Item> {
        // attributes? skip #
        while self.peek_kind() == Some(myc_lexer::TokenKind::Hash) {
            self.advance(); // #
            if self.peek_kind() == Some(myc_lexer::TokenKind::LBracket) {
                self.advance();
                // skip until ]
                while !self.at_eof()
                    && !matches!(self.peek_kind(), Some(myc_lexer::TokenKind::RBracket))
                {
                    self.advance();
                }
                self.advance();
            }
        }
        // pub
        let is_pub = if self.peek_kind() == Some(myc_lexer::TokenKind::Pub) {
            self.advance();
            true
        } else {
            false
        };
        let kw = self.peek_kind()?;
        match kw {
            myc_lexer::TokenKind::Fn => {
                let span_start = self.peek_span();
                self.advance();
                let mut def = self.parse_fn_rest(is_pub)?;
                def.span = def.span.merge(span_start);
                Some(Item::Fn(def))
            }
            myc_lexer::TokenKind::Struct => {
                self.advance();
                let def = self.parse_struct(is_pub)?;
                Some(Item::Struct(def))
            }
            myc_lexer::TokenKind::Enum => {
                self.advance();
                let def = self.parse_enum(is_pub)?;
                Some(Item::Enum(def))
            }
            myc_lexer::TokenKind::Trait => {
                self.advance();
                let def = self.parse_trait(is_pub)?;
                Some(Item::Trait(def))
            }
            myc_lexer::TokenKind::Impl => {
                self.advance();
                let def = self.parse_impl()?;
                Some(Item::Impl(def))
            }
            myc_lexer::TokenKind::Const => {
                self.advance();
                let def = self.parse_const()?;
                Some(Item::Const(def))
            }
            myc_lexer::TokenKind::Static => {
                self.advance();
                let def = self.parse_static()?;
                Some(Item::Static(def))
            }
            myc_lexer::TokenKind::Type => {
                self.advance();
                let def = self.parse_type_alias()?;
                Some(Item::TypeAlias(def))
            }
            myc_lexer::TokenKind::Mod => {
                self.advance();
                let def = self.parse_mod()?;
                Some(Item::Mod(def))
            }
            myc_lexer::TokenKind::Use => {
                self.advance();
                let def = self.parse_use()?;
                Some(Item::Use(def))
            }
            myc_lexer::TokenKind::Unsafe => {
                // unsafe fn
                self.advance();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Fn) {
                    self.advance();
                    let mut def = self.parse_fn_rest(is_pub)?;
                    def.is_unsafe = true;
                    Some(Item::Fn(def))
                } else {
                    self.diag(codes::E0001, "expected fn after unsafe", self.peek_span());
                    None
                }
            }
            _ => {
                // forward decl? fn name(...); already handled by Fn branch requires body optional.
                // maybe expression-level? skip
                self.diag(codes::E0001, "expected item", self.peek_span());
                None
            }
        }
    }

    fn parse_fn_rest(&mut self, is_pub: bool) -> Option<FnDef> {
        let span_start = self.peek_span();
        // name
        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        // params
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LParen),
            "expected '('",
        );
        let mut params = Vec::new();
        if !self.is(&myc_lexer::TokenKind::RParen) {
            loop {
                let pname = self.parse_ident().unwrap_or_default();
                let mut ty = None;
                if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
                    self.advance();
                    ty = Some(self.parse_type());
                }
                params.push(Param {
                    name: pname,
                    ty,
                    span: self.peek_span(),
                });
                if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                    self.advance();
                    if self.is(&myc_lexer::TokenKind::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        let _ = self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RParen),
            "expected ')'",
        );
        // return type
        let mut ret = None;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Arrow) {
            self.advance();
            ret = Some(self.parse_type());
        }
        // body or forward decl
        let mut body = None;
        let is_unsafe = false;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        } else if self.peek_kind() == Some(myc_lexer::TokenKind::LBrace) {
            body = Some(self.parse_block());
        } else {
            self.diag(codes::E0006, "expected '{' or ';'", self.peek_span());
        }
        Some(FnDef {
            name,
            generics,
            params,
            ret,
            body,
            is_unsafe,
            is_pub,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_generics(&mut self) -> Vec<GenericParam> {
        let mut out = Vec::new();
        if self.peek_kind() == Some(myc_lexer::TokenKind::LBracket) {
            // use [T] style? spec uses [T] maybe. Also support <T>? We'll support [..]
            self.advance();
            loop {
                if self.is(&myc_lexer::TokenKind::RBracket) {
                    break;
                }
                let name = self.parse_ident().unwrap_or_default();
                let mut bounds = Vec::new();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
                    self.advance();
                    bounds.push(self.parse_type());
                    while self.peek_kind() == Some(myc_lexer::TokenKind::Plus) {
                        self.advance();
                        bounds.push(self.parse_type());
                    }
                }
                out.push(GenericParam {
                    name,
                    bounds,
                    span: self.peek_span(),
                });
                if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek_kind() == Some(myc_lexer::TokenKind::RBracket) {
                self.advance();
            }
        }
        out
    }

    fn parse_struct(&mut self, is_pub: bool) -> Option<StructDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LBrace),
            "expected '{'",
        );
        let mut fields = Vec::new();
        while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
            // field
            let fname = match self.parse_ident() {
                Some(n) => n,
                None => {
                    self.advance();
                    continue;
                }
            };
            if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
                self.advance();
                let ty = self.parse_type();
                fields.push(Field {
                    name: fname,
                    ty,
                    span: self.peek_span(),
                });
            }
            if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                self.advance();
            } else if !self.is(&myc_lexer::TokenKind::RBrace) {
                break;
            }
        }
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RBrace),
            "expected '}'",
        );
        Some(StructDef {
            name,
            generics,
            fields,
            is_pub,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_enum(&mut self, is_pub: bool) -> Option<EnumDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LBrace),
            "expected '{'",
        );
        let mut variants = Vec::new();
        while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
            let vname = match self.parse_ident() {
                Some(n) => n,
                None => {
                    self.advance();
                    continue;
                }
            };
            let mut fields = Vec::new();
            if self.peek_kind() == Some(myc_lexer::TokenKind::LParen) {
                self.advance();
                if !self.is(&myc_lexer::TokenKind::RParen) {
                    loop {
                        fields.push(self.parse_type());
                        if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                let _ = self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RParen),
                    "expected ')'",
                );
            }
            variants.push(Variant {
                name: vname,
                fields,
                span: self.peek_span(),
            });
            if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                self.advance();
            } else if !self.is(&myc_lexer::TokenKind::RBrace) {
                break;
            }
        }
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RBrace),
            "expected '}'",
        );
        Some(EnumDef {
            name,
            generics,
            variants,
            is_pub,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_trait(&mut self, _is_pub: bool) -> Option<TraitDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LBrace),
            "expected '{'",
        );
        let mut items = Vec::new();
        while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                self.advance();
            }
        }
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RBrace),
            "expected '}'",
        );
        Some(TraitDef {
            name,
            items,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_impl(&mut self) -> Option<ImplDef> {
        let span_start = self.peek_span();
        // optional trait for type
        let first = self.parse_type();
        let (trait_name, for_type) = if self.peek_kind() == Some(myc_lexer::TokenKind::For) {
            // first should be named type as trait
            let tname = match first {
                Type::Named(n, _, _) => n,
                _ => String::new(),
            };
            self.advance();
            (Some(tname), self.parse_type())
        } else {
            (None, first)
        };
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LBrace),
            "expected '{'",
        );
        let mut items = Vec::new();
        while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                self.advance();
            }
        }
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RBrace),
            "expected '}'",
        );
        Some(ImplDef {
            trait_name,
            for_type,
            items,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_const(&mut self) -> Option<ConstDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        let mut ty = None;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
            self.advance();
            ty = Some(self.parse_type());
        }
        self.expect(|k| matches!(k, myc_lexer::TokenKind::Eq), "expected '='");
        let value = self.parse_expr();
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        }
        Some(ConstDef {
            name,
            ty,
            value,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_static(&mut self) -> Option<StaticDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        let mut ty = None;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
            self.advance();
            ty = Some(self.parse_type());
        }
        let mut value = None;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Eq) {
            self.advance();
            value = Some(self.parse_expr());
        }
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        }
        Some(StaticDef {
            name,
            ty,
            value,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_type_alias(&mut self) -> Option<TypeAlias> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        self.expect(|k| matches!(k, myc_lexer::TokenKind::Eq), "expected '='");
        let ty = self.parse_type();
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        }
        Some(TypeAlias {
            name,
            ty,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_mod(&mut self) -> Option<ModDef> {
        let span_start = self.peek_span();
        let name = self.parse_ident()?;
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        }
        Some(ModDef {
            name,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_use(&mut self) -> Option<UseDef> {
        let span_start = self.peek_span();
        let mut path = Vec::new();
        // path segments separated by :: or .
        while let Some(id) = self.parse_ident() {
            path.push(id);
            if self.peek_kind() == Some(myc_lexer::TokenKind::ColonColon)
                || self.peek_kind() == Some(myc_lexer::TokenKind::Dot)
            {
                self.advance();
                continue;
            }
            break;
        }
        if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
            self.advance();
        }
        Some(UseDef {
            path,
            span: span_start.merge(self.peek_span()),
        })
    }

    fn parse_ident(&mut self) -> Option<String> {
        match self.peek() {
            Some(t) if matches!(t.kind, myc_lexer::TokenKind::Ident) => {
                let txt = t.text.clone();
                self.advance();
                Some(txt)
            }
            Some(t)
                if matches!(
                    t.kind,
                    myc_lexer::TokenKind::Fn | myc_lexer::TokenKind::Let | _
                ) =>
            {
                // allow keywords as idents? no. try ident only
                None
            }
            _ => None,
        }
    }

    // Types
    fn parse_type(&mut self) -> Type {
        let span_start = self.peek_span();
        // reference / pointer
        if self.peek_kind() == Some(myc_lexer::TokenKind::Amp) {
            self.advance();
            if self.peek_kind() == Some(myc_lexer::TokenKind::Mut) {
                self.advance();
                let inner = self.parse_type();
                return Type::RefMut(Box::new(inner), span_start.merge(self.peek_span()));
            }
            let inner = self.parse_type();
            return Type::Ref(Box::new(inner), span_start.merge(self.peek_span()));
        }
        if self.peek_kind() == Some(myc_lexer::TokenKind::Star) {
            // *T or *mut T
            self.advance();
            if self.peek_kind() == Some(myc_lexer::TokenKind::Mut) {
                self.advance();
                let inner = self.parse_type();
                return Type::PtrMut(Box::new(inner), span_start.merge(self.peek_span()));
            }
            let inner = self.parse_type();
            return Type::Ptr(Box::new(inner), span_start.merge(self.peek_span()));
        }
        // tuple or parenthesized
        if self.peek_kind() == Some(myc_lexer::TokenKind::LParen) {
            self.advance();
            if self.is(&myc_lexer::TokenKind::RParen) {
                self.advance();
                return Type::Tuple(vec![], span_start.merge(self.peek_span()));
            }
            let first = self.parse_type();
            if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                let mut elems = vec![first];
                while self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                    self.advance();
                    if self.is(&myc_lexer::TokenKind::RParen) {
                        break;
                    }
                    elems.push(self.parse_type());
                }
                let _ = self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RParen),
                    "expected ')'",
                );
                return Type::Tuple(elems, span_start.merge(self.peek_span()));
            } else {
                let _ = self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RParen),
                    "expected ')'",
                );
                // parenthesized type
                return first;
            }
        }
        // array [T; N] or slice [T]
        if self.peek_kind() == Some(myc_lexer::TokenKind::LBracket) {
            self.advance();
            let inner = self.parse_type();
            if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                self.advance();
                let size = self.parse_expr();
                self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RBracket),
                    "expected ']'",
                );
                return Type::Array(
                    Box::new(inner),
                    Some(Box::new(size)),
                    span_start.merge(self.peek_span()),
                );
            }
            self.expect(
                |k| matches!(k, myc_lexer::TokenKind::RBracket),
                "expected ']'",
            );
            // could be slice &[T] handled by caller? Return slice type
            return Type::Slice(Box::new(inner), span_start.merge(self.peek_span()));
        }
        // named type, possibly generic Foo[T, U]
        let name = self.parse_ident().unwrap_or_default();
        let mut args = Vec::new();
        if self.peek_kind() == Some(myc_lexer::TokenKind::LBracket) {
            self.advance();
            if !self.is(&myc_lexer::TokenKind::RBracket) {
                loop {
                    args.push(self.parse_type());
                    if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            if self.peek_kind() == Some(myc_lexer::TokenKind::RBracket) {
                self.advance();
            }
        }
        // function type? fn(...) -> ...
        if name == "fn" {
            // parse params
            // not common; skip
        }
        Type::Named(name, args, span_start.merge(self.peek_span()))
    }

    fn parse_block(&mut self) -> Block {
        let span_start = self.peek_span();
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::LBrace),
            "expected '{'",
        );
        let mut stmts = Vec::new();
        let mut expr = None;
        while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
            // item?
            if self.is_item_start() && let Some(item) = self.parse_item() {
                stmts.push(Stmt::Item(item));
                continue;
            }
            // let
            if self.peek_kind() == Some(myc_lexer::TokenKind::Let) {
                self.advance();
                let mutable = if self.peek_kind() == Some(myc_lexer::TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let name = self.parse_ident().unwrap_or_default();
                let mut ty = None;
                if self.peek_kind() == Some(myc_lexer::TokenKind::Colon) {
                    self.advance();
                    ty = Some(self.parse_type());
                }
                let mut init = None;
                if self.peek_kind() == Some(myc_lexer::TokenKind::Eq) {
                    self.advance();
                    init = Some(self.parse_expr());
                }
                if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                    self.advance();
                }
                stmts.push(Stmt::Let(LetStmt {
                    name,
                    mutable,
                    ty,
                    init,
                    span: self.peek_span(),
                }));
                continue;
            }
            // defer
            if self.peek_kind() == Some(myc_lexer::TokenKind::Defer) {
                self.advance();
                let e = self.parse_expr();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                    self.advance();
                }
                stmts.push(Stmt::Defer(e));
                continue;
            }
            // return
            if self.peek_kind() == Some(myc_lexer::TokenKind::Return) {
                self.advance();
                let e = if self.is(&myc_lexer::TokenKind::Semicolon)
                    || self.is(&myc_lexer::TokenKind::RBrace)
                {
                    None
                } else {
                    Some(self.parse_expr())
                };
                if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                    self.advance();
                }
                stmts.push(Stmt::Return(e));
                continue;
            }
            // break/continue
            if self.peek_kind() == Some(myc_lexer::TokenKind::Break) {
                self.advance();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                    self.advance();
                }
                stmts.push(Stmt::Break);
                continue;
            }
            if self.peek_kind() == Some(myc_lexer::TokenKind::Continue) {
                self.advance();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                    self.advance();
                }
                stmts.push(Stmt::Continue);
                continue;
            }
            // parse expression statement
            let e = self.parse_expr();
            if self.peek_kind() == Some(myc_lexer::TokenKind::Semicolon) {
                self.advance();
                stmts.push(Stmt::Expr(e));
            } else if self.is(&myc_lexer::TokenKind::RBrace) {
                expr = Some(Box::new(e));
            } else {
                stmts.push(Stmt::Expr(e));
            }
        }
        self.expect(
            |k| matches!(k, myc_lexer::TokenKind::RBrace),
            "expected '}'",
        );
        Block {
            stmts,
            expr,
            span: span_start.merge(self.peek_span()),
        }
    }

    fn is_item_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(myc_lexer::TokenKind::Fn)
                | Some(myc_lexer::TokenKind::Struct)
                | Some(myc_lexer::TokenKind::Enum)
                | Some(myc_lexer::TokenKind::Const)
                | Some(myc_lexer::TokenKind::Static)
                | Some(myc_lexer::TokenKind::Type)
                | Some(myc_lexer::TokenKind::Mod)
                | Some(myc_lexer::TokenKind::Use)
                | Some(myc_lexer::TokenKind::Impl)
                | Some(myc_lexer::TokenKind::Trait)
                | Some(myc_lexer::TokenKind::Pub)
                | Some(myc_lexer::TokenKind::Unsafe)
        )
    }

    // Expressions
    fn parse_expr(&mut self) -> Expr {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Expr {
        let left = self.parse_range();
        if let Some(k) = self.peek_kind() {
            // only simple =
            if matches!(k, myc_lexer::TokenKind::Eq) {
                self.advance();
                let right = self.parse_assign();
                let span = Parser::span_of(&left).merge(Parser::span_of(&right));
                return Expr::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                    span,
                };
            }
        }
        left
    }

    fn span_of(e: &Expr) -> Span {
        match e {
            Expr::Lit(_, s) => *s,
            Expr::Path(_, s) => *s,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::Field { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::While { span, .. } => *span,
            Expr::For { span, .. } => *span,
            Expr::Loop { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Block(b) => b.span,
            Expr::Closure { span, .. } => *span,
            Expr::Cast { span, .. } => *span,
            Expr::Range { span, .. } => *span,
            Expr::Try { span, .. } => *span,
            Expr::Unsafe { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Array { span, .. } => *span,
        }
    }

    fn parse_range(&mut self) -> Expr {
        let left = self.parse_or();
        if self.peek_kind() == Some(myc_lexer::TokenKind::DotDot) {
            self.advance();
            let right = if self.is_range_end() {
                None
            } else {
                Some(Box::new(self.parse_or()))
            };
            let span = Parser::span_of(&left).merge(self.peek_span());
            return Expr::Range {
                start: Some(Box::new(left)),
                end: right,
                inclusive: false,
                span,
            };
        }
        if self.peek_kind() == Some(myc_lexer::TokenKind::DotDotEq) {
            self.advance();
            let right = if self.is_range_end() {
                None
            } else {
                Some(Box::new(self.parse_or()))
            };
            let span = Parser::span_of(&left).merge(self.peek_span());
            return Expr::Range {
                start: Some(Box::new(left)),
                end: right,
                inclusive: true,
                span,
            };
        }
        left
    }

    fn is_range_end(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(myc_lexer::TokenKind::Semicolon)
                | Some(myc_lexer::TokenKind::Comma)
                | Some(myc_lexer::TokenKind::RParen)
                | Some(myc_lexer::TokenKind::RBrace)
                | Some(myc_lexer::TokenKind::RBracket)
                | None
        )
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.peek_kind() == Some(myc_lexer::TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_and();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_eq();
        while self.peek_kind() == Some(myc_lexer::TokenKind::AmpAmp) {
            self.advance();
            let right = self.parse_eq();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_eq(&mut self) -> Expr {
        let mut left = self.parse_cmp();
        loop {
            let op = match self.peek_kind() {
                Some(myc_lexer::TokenKind::EqEq) => BinOp::Eq,
                Some(myc_lexer::TokenKind::NotEq) => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_cmp();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_cmp(&mut self) -> Expr {
        let mut left = self.parse_bitor();
        loop {
            let op = match self.peek_kind() {
                Some(myc_lexer::TokenKind::Lt) => BinOp::Lt,
                Some(myc_lexer::TokenKind::Le) => BinOp::Le,
                Some(myc_lexer::TokenKind::Gt) => BinOp::Gt,
                Some(myc_lexer::TokenKind::Ge) => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_bitor();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_bitor(&mut self) -> Expr {
        let mut left = self.parse_bitxor();
        while self.peek_kind() == Some(myc_lexer::TokenKind::Pipe) {
            self.advance();
            let right = self.parse_bitxor();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op: BinOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_bitxor(&mut self) -> Expr {
        let mut left = self.parse_bitand();
        while self.peek_kind() == Some(myc_lexer::TokenKind::Caret) {
            self.advance();
            let right = self.parse_bitand();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op: BinOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_bitand(&mut self) -> Expr {
        let mut left = self.parse_shift();
        while self.peek_kind() == Some(myc_lexer::TokenKind::Amp) {
            self.advance();
            let right = self.parse_shift();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op: BinOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_add();
        loop {
            let op = match self.peek_kind() {
                Some(myc_lexer::TokenKind::Lt) if self.peek2_is(&myc_lexer::TokenKind::Lt) => {
                    BinOp::Shl
                }
                Some(myc_lexer::TokenKind::Gt) if self.peek2_is(&myc_lexer::TokenKind::Gt) => {
                    BinOp::Shr
                }
                _ => break,
            };
            self.advance();
            self.advance();
            let right = self.parse_add();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn peek2_is(&self, k: &myc_lexer::TokenKind) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(k))
            .unwrap_or(false)
    }

    fn parse_add(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            let op = match self.peek_kind() {
                Some(myc_lexer::TokenKind::Plus) => BinOp::Add,
                Some(myc_lexer::TokenKind::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_mul(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let op = match self.peek_kind() {
                Some(myc_lexer::TokenKind::Star) => BinOp::Mul,
                Some(myc_lexer::TokenKind::Slash) => BinOp::Div,
                Some(myc_lexer::TokenKind::Percent) => BinOp::Rem,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            let span = Parser::span_of(&left).merge(Parser::span_of(&right));
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek_kind() {
            Some(myc_lexer::TokenKind::Minus) => {
                self.advance();
                let operand = self.parse_unary();
                let span = self.peek_span().merge(Parser::span_of(&operand));
                Expr::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Bang) => {
                self.advance();
                let operand = self.parse_unary();
                let span = self.peek_span().merge(Parser::span_of(&operand));
                Expr::Unary {
                    op: UnOp::Not,
                    operand: Box::new(operand),
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Star) => {
                self.advance();
                let operand = self.parse_unary();
                let span = self.peek_span().merge(Parser::span_of(&operand));
                Expr::Unary {
                    op: UnOp::Deref,
                    operand: Box::new(operand),
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Amp) => {
                self.advance();
                // &mut?
                if self.peek_kind() == Some(myc_lexer::TokenKind::Mut) {
                    self.advance();
                    let operand = self.parse_unary();
                    let span = self.peek_span().merge(Parser::span_of(&operand));
                    Expr::Unary {
                        op: UnOp::RefMut,
                        operand: Box::new(operand),
                        span,
                    }
                } else {
                    let operand = self.parse_unary();
                    let span = self.peek_span().merge(Parser::span_of(&operand));
                    Expr::Unary {
                        op: UnOp::Ref,
                        operand: Box::new(operand),
                        span,
                    }
                }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            match self.peek_kind() {
                Some(myc_lexer::TokenKind::LParen) => {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.is(&myc_lexer::TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr());
                            if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    let _ = self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::RParen),
                        "expected ')'",
                    );
                    let span = Parser::span_of(&expr).merge(self.peek_span());
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    };
                }
                Some(myc_lexer::TokenKind::Dot) => {
                    self.advance();
                    if let Some(myc_lexer::TokenKind::Ident) = self.peek_kind() {
                        let method = self.peek().unwrap().text.clone();
                        // could be field or method call; check next
                        if self
                            .tokens
                            .get(self.pos + 1)
                            .map(|t| matches!(t.kind, myc_lexer::TokenKind::LParen))
                            .unwrap_or(false)
                        {
                            self.advance();
                            self.advance(); // (
                            let mut args = Vec::new();
                            if !self.is(&myc_lexer::TokenKind::RParen) {
                                loop {
                                    args.push(self.parse_expr());
                                    if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            let _ = self.expect(
                                |k| matches!(k, myc_lexer::TokenKind::RParen),
                                "expected ')'",
                            );
                            let span = Parser::span_of(&expr).merge(self.peek_span());
                            expr = Expr::MethodCall {
                                receiver: Box::new(expr),
                                method,
                                args,
                                span,
                            };
                        } else {
                            self.advance();
                            let span = Parser::span_of(&expr).merge(self.peek_span());
                            expr = Expr::Field {
                                base: Box::new(expr),
                                name: method,
                                span,
                            };
                        }
                    } else {
                        self.diag(codes::E0004, "expected identifier", self.peek_span());
                        break;
                    }
                }
                Some(myc_lexer::TokenKind::LBracket) => {
                    self.advance();
                    let idx = self.parse_expr();
                    self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::RBracket),
                        "expected ']'",
                    );
                    let span = Parser::span_of(&expr).merge(self.peek_span());
                    expr = Expr::Index {
                        base: Box::new(expr),
                        index: Box::new(idx),
                        span,
                    };
                }
                Some(myc_lexer::TokenKind::Question) => {
                    self.advance();
                    let span = Parser::span_of(&expr);
                    expr = Expr::Try {
                        expr: Box::new(expr),
                        span,
                    };
                }
                Some(myc_lexer::TokenKind::As) => {
                    self.advance();
                    let ty = self.parse_type();
                    let span = Parser::span_of(&expr).merge(ty.span());
                    expr = Expr::Cast {
                        expr: Box::new(expr),
                        ty,
                        span,
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek_kind() {
            Some(myc_lexer::TokenKind::Int(v)) => {
                let span = self.peek_span();
                self.advance();
                Expr::Lit(Lit::Int(v), span)
            }
            Some(myc_lexer::TokenKind::Float(v)) => {
                let span = self.peek_span();
                self.advance();
                Expr::Lit(Lit::Float(v), span)
            }
            Some(myc_lexer::TokenKind::Str(s)) => {
                let s = s.clone();
                let span = self.peek_span();
                self.advance();
                Expr::Lit(Lit::Str(s), span)
            }
            Some(myc_lexer::TokenKind::Char(c)) => {
                let span = self.peek_span();
                self.advance();
                Expr::Lit(Lit::Char(c), span)
            }
            Some(myc_lexer::TokenKind::Bool(b)) => {
                let span = self.peek_span();
                self.advance();
                Expr::Lit(Lit::Bool(b), span)
            }
            Some(myc_lexer::TokenKind::LParen) => {
                self.advance();
                if self.is(&myc_lexer::TokenKind::RParen) {
                    self.advance();
                    return Expr::Tuple {
                        elems: vec![],
                        span: self.peek_span(),
                    };
                }
                let first = self.parse_expr();
                if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                    let mut elems = vec![first];
                    while self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                        self.advance();
                        if self.is(&myc_lexer::TokenKind::RParen) {
                            break;
                        }
                        elems.push(self.parse_expr());
                    }
                    let _ = self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::RParen),
                        "expected ')'",
                    );
                    let span = self.peek_span();
                    Expr::Tuple { elems, span }
                } else {
                    let _ = self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::RParen),
                        "expected ')'",
                    );
                    first
                }
            }
            Some(myc_lexer::TokenKind::LBracket) => {
                self.advance();
                let mut elems = Vec::new();
                if !self.is(&myc_lexer::TokenKind::RBracket) {
                    loop {
                        elems.push(self.parse_expr());
                        if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RBracket),
                    "expected ']'",
                );
                let span = self.peek_span();
                Expr::Array { elems, span }
            }
            Some(myc_lexer::TokenKind::LBrace) => {
                let b = self.parse_block();
                Expr::Block(b)
            }
            Some(myc_lexer::TokenKind::If) => {
                self.advance();
                let cond = self.parse_expr();
                let then_branch = self.parse_block();
                let mut else_branch = None;
                if self.peek_kind() == Some(myc_lexer::TokenKind::Else) {
                    self.advance();
                    if self.peek_kind() == Some(myc_lexer::TokenKind::If) {
                        else_branch = Some(Box::new(self.parse_expr()));
                    } else {
                        let b = self.parse_block();
                        else_branch = Some(Box::new(Expr::Block(b)));
                    }
                }
                let span = Parser::span_of(&cond).merge(then_branch.span);
                Expr::If {
                    cond: Box::new(cond),
                    then_branch,
                    else_branch,
                    span,
                }
            }
            Some(myc_lexer::TokenKind::While) => {
                self.advance();
                let cond = self.parse_expr();
                let body = self.parse_block();
                let span = Parser::span_of(&cond).merge(body.span);
                Expr::While {
                    cond: Box::new(cond),
                    body,
                    span,
                }
            }
            Some(myc_lexer::TokenKind::For) => {
                self.advance();
                let name = self.parse_ident().unwrap_or_default();
                if self.peek_kind() == Some(myc_lexer::TokenKind::In) {
                    self.advance();
                }
                let iter = self.parse_expr();
                let body = self.parse_block();
                let span = Parser::span_of(&iter).merge(body.span);
                Expr::For {
                    name,
                    iter: Box::new(iter),
                    body,
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Loop) => {
                self.advance();
                let body = self.parse_block();
                let span = body.span;
                Expr::Loop { body, span }
            }
            Some(myc_lexer::TokenKind::Match) => {
                self.advance();
                let scrutinee = self.parse_expr();
                self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::LBrace),
                    "expected '{'",
                );
                let mut arms = Vec::new();
                while !self.is(&myc_lexer::TokenKind::RBrace) && !self.at_eof() {
                    let pat = self.parse_pattern();
                    self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::FatArrow),
                        "expected '=>'",
                    );
                    let body = self.parse_expr();
                    arms.push(MatchArm {
                        pattern: pat,
                        body,
                        span: self.peek_span(),
                    });
                    if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                        self.advance();
                    }
                }
                self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RBrace),
                    "expected '}'",
                );
                let span = Parser::span_of(&scrutinee).merge(self.peek_span());
                Expr::Match {
                    scrutinee: Box::new(scrutinee),
                    arms,
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Unsafe) => {
                self.advance();
                let block = self.parse_block();
                Expr::Unsafe {
                    block,
                    span: self.peek_span(),
                }
            }
            Some(myc_lexer::TokenKind::Pipe) | Some(myc_lexer::TokenKind::PipePipe) => {
                // closure |x| expr
                self.advance();
                let mut params = Vec::new();
                if !matches!(
                    self.peek_kind(),
                    Some(myc_lexer::TokenKind::Pipe) | Some(myc_lexer::TokenKind::PipePipe)
                ) {
                    // parse param names until |
                    while let Some(id) = self.parse_ident() {
                        params.push(id);
                        if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                // expect closing |
                if matches!(
                    self.peek_kind(),
                    Some(myc_lexer::TokenKind::Pipe) | Some(myc_lexer::TokenKind::PipePipe)
                ) {
                    self.advance();
                }
                let body = self.parse_expr();
                let span = self.peek_span();
                Expr::Closure {
                    params,
                    body: Box::new(body),
                    span,
                }
            }
            Some(myc_lexer::TokenKind::Ident) => {
                let name = self.peek().unwrap().text.clone();
                self.advance();
                // path?
                let mut path = vec![name];
                while self.peek_kind() == Some(myc_lexer::TokenKind::ColonColon) {
                    self.advance();
                    if let Some(id) = self.parse_ident() {
                        path.push(id);
                    } else {
                        break;
                    }
                }
                Expr::Path(path, self.peek_span())
            }
            _ => {
                let span = self.peek_span();
                self.diag(codes::E0002, "expected expression", span);
                // advance to avoid loop
                self.advance();
                Expr::Lit(Lit::Int(0), span)
            }
        }
    }

    fn parse_pattern(&mut self) -> Pattern {
        match self.peek_kind() {
            Some(myc_lexer::TokenKind::Ident) => {
                let name = self.peek().unwrap().text.clone();
                let span = self.peek_span();
                self.advance();
                // variant with fields?
                if self.peek_kind() == Some(myc_lexer::TokenKind::LParen) {
                    self.advance();
                    let mut pats = Vec::new();
                    if !self.is(&myc_lexer::TokenKind::RParen) {
                        loop {
                            pats.push(self.parse_pattern());
                            if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    let _ = self.expect(
                        |k| matches!(k, myc_lexer::TokenKind::RParen),
                        "expected ')'",
                    );
                    Pattern::Variant(name, pats, span)
                } else if name == "_" {
                    Pattern::Wildcard(span)
                } else {
                    Pattern::Binding(name, span)
                }
            }
            Some(myc_lexer::TokenKind::LParen) => {
                self.advance();
                let mut pats = Vec::new();
                if !self.is(&myc_lexer::TokenKind::RParen) {
                    loop {
                        pats.push(self.parse_pattern());
                        if self.peek_kind() == Some(myc_lexer::TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                let span = self.peek_span();
                let _ = self.expect(
                    |k| matches!(k, myc_lexer::TokenKind::RParen),
                    "expected ')'",
                );
                Pattern::Tuple(pats, span)
            }
            Some(myc_lexer::TokenKind::Int(v)) => {
                let span = self.peek_span();
                self.advance();
                Pattern::Lit(Lit::Int(v), span)
            }
            Some(myc_lexer::TokenKind::Bool(b)) => {
                let span = self.peek_span();
                self.advance();
                Pattern::Lit(Lit::Bool(b), span)
            }
            Some(myc_lexer::TokenKind::Str(s)) => {
                let s = s.clone();
                let span = self.peek_span();
                self.advance();
                Pattern::Lit(Lit::Str(s), span)
            }
            _ => {
                let span = self.peek_span();
                self.advance();
                Pattern::Wildcard(span)
            }
        }
    }
}

// Item span helper
fn item_span(item: &Item) -> Span {
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


#[cfg(test)]
mod tests {
    use super::*;
    use myc_lexer::Lexer;
    use myc_span::SourceMap;
    #[test]
    fn parse_simple() {
        let mut sm = SourceMap::new();
        let fid = sm.add_file("t.myc", "fn main() { let x = 5; println(x); }");
        let src = sm.source(fid);
        let (toks, _) = Lexer::new(src, fid).lex();
        let (file, diags) = Parser::new(toks, fid).parse_file();
        assert!(diags.is_empty(), "{:?}", diags);
        assert_eq!(file.items.len(), 1);
    }
}

