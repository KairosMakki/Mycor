//! Language server for Mycor.

use lsp_server::{Connection, Message};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};
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
                match req.method.as_str() {
                    "textDocument/didOpen" | "textDocument/didChange" => {
                        // ignore for now
                    }
                    _ => {}
                }
            }
            Message::Notification(notif) => {
                if notif.method == "textDocument/didOpen"
                    && let Ok(params) = serde_json::from_value::<lsp_types::DidOpenTextDocumentParams>(
                        notif.params.clone(),
                    )
                {
                    let text = params.text_document.text.clone();
                    let uri = params.text_document.uri.clone();
                    documents.insert(uri.to_string(), text.clone());
                    publish_diagnostics(&connection, &uri, text);
                }
                if notif.method == "textDocument/didChange" {
                    // skip for now
                }
            }
            Message::Response(_) => {}
        }
    }
    io_threads.join().unwrap();
}

fn publish_diagnostics(connection: &Connection, uri: &Uri, text: String) {
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
            ..Default::default()
        })
        .collect();
    let params = lsp_types::PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: lsp_diags,
        version: None,
    };
    connection
        .sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params).unwrap(),
        }))
        .unwrap();
}
