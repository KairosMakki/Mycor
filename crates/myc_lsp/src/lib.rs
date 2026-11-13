//! Language server for Mycor.

use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeResult, MarkupContent,
    MarkupKind, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use serde_json::Value;
use std::collections::HashMap;

pub fn run() {
    let (connection, io_threads) = Connection::stdio();
    let mut documents: HashMap<String, String> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req).unwrap_or(false) {
                    break;
                }
                handle_request(&connection, &mut documents, req);
            }
            Message::Notification(notif) => {
                if notif.method == "textDocument/didOpen"
                    && let Ok(params) =
                        serde_json::from_value::<DidOpenTextDocumentParams>(notif.params.clone())
                {
                    let text = params.text_document.text.clone();
                    let uri = params.text_document.uri.clone();
                    documents.insert(uri.to_string(), text.clone());
                    publish_diagnostics(&connection, &uri, text);
                }
                if notif.method == "textDocument/didChange"
                    && let Ok(params) =
                        serde_json::from_value::<DidChangeTextDocumentParams>(notif.params.clone())
                {
                    let uri = params.text_document.uri.clone();
                    if let Some(change) = params.content_changes.into_iter().last() {
                        documents.insert(uri.to_string(), change.text.clone());
                        publish_diagnostics(&connection, &uri, change.text);
                    }
                }
            }
            Message::Response(_) => {}
        }
    }
    io_threads.join().unwrap();
}

fn handle_request(connection: &Connection, documents: &mut HashMap<String, String>, req: Request) {
    match req.method.as_str() {
        "initialize" => {
            let caps = ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            };
            let result = InitializeResult {
                capabilities: caps,
                server_info: Some(lsp_types::ServerInfo {
                    name: "myc-lsp".to_string(),
                    version: Some("0.2.0".to_string()),
                }),
            };
            let resp = Response {
                id: req.id,
                response_result: Ok(serde_json::to_value(result).unwrap_or(Value::Null)),
            };
            let _ = connection.sender.send(Message::Response(resp));
        }
        "textDocument/hover" => {
            let params: HoverParams = serde_json::from_value(req.params).unwrap_or_else(|_| {
                // fallback empty
                serde_json::from_value(Value::Null).unwrap()
            });
            let uri = params
                .text_document_position_params
                .text_document
                .uri
                .to_string();
            let text = documents.get(&uri).cloned().unwrap_or_default();
            let contents = if text.is_empty() {
                "Mycor".to_string()
            } else {
                "Mycor source".to_string()
            };
            let hover = Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: contents,
                }),
                range: None,
            };
            let resp = Response {
                id: req.id,
                response_result: Ok(serde_json::to_value(hover).unwrap_or(Value::Null)),
            };
            let _ = connection.sender.send(Message::Response(resp));
        }
        _ => {}
    }
}

fn publish_diagnostics(connection: &Connection, uri: &lsp_types::Uri, text: String) {
    let diags = myc_driver::check_source(&text, "main.myc");
    let lsp_diags: Vec<Diagnostic> = diags
        .into_iter()
        .map(|d| Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: d.message,
            code: Some(lsp_types::NumberOrString::String(d.code.to_string())),
            ..Default::default()
        })
        .collect();
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: lsp_diags,
        version: None,
    };
    let _ = connection
        .sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params).unwrap(),
        }));
}
