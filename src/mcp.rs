use crate::commands;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn serve() {
    eprintln!("v8md MCP server starting on stdio");
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&trimmed) {
            Ok(v) => v,
            Err(e) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": format!("Parse error: {}", e)}
                });
                writeln!(stdout_lock, "{}", response).ok();
                stdout_lock.flush().ok();
                continue;
            }
        };

        let id = request.get("id").cloned();
        let method = request.get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        let is_notification = id.is_none() || id.as_ref().map_or(true, |v| v.is_null());

        if method == "notifications/initialized" {
            continue;
        }

        if method == "notifications/cancelled" {
            continue;
        }

        let params = request.get("params").cloned().unwrap_or(json!({}));

        let result = match method.as_str() {
            "initialize" => {
                handle_initialize(params)
            }
            "ping" => json!({}),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(params),
            _ => json!({"error": {"code": -32601, "message": format!("Method not found: {}", method)}}),
        };

        if is_notification {
            continue;
        }

        let mut response = json!({
            "jsonrpc": "2.0",
            "id": id,
        });

        if result.get("error").is_some() {
            response["error"] = result["error"].clone();
        } else {
            response["result"] = result;
        }

        writeln!(stdout_lock, "{}", response).ok();
        stdout_lock.flush().ok();
    }
}

fn handle_initialize(_params: Value) -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "v8md",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "v8md_load_dump",
                "description": "Загрузить XML-дамп конфигурации 1С в кэш именованного проекта. Кэш проекта пересоздаётся заново.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Путь к папке с Configuration.xml"
                        },
                        "project": {
                            "type": "string",
                            "description": "Имя проекта (обязательно). Разные проекты позволяют работать с несколькими базами одновременно."
                        }
                    },
                    "required": ["path", "project"]
                }
            },
            {
                "name": "v8md_projects",
                "description": "Список загруженных проектов (баз 1С) с именем и версией конфигурации. Вызывайте, чтобы выбрать, в какой проект грузить дамп или из какого читать метаданные.",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "v8md_get",
                "description": "Получить метаданные объекта 1С. Формат: Тип.Имя (например: Document.Заказ, Catalog.Контрагенты).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "object": {
                            "type": "string",
                            "description": "Имя объекта, например: Catalog.Контрагенты, Document.Заказ"
                        },
                        "project": {
                            "type": "string",
                            "description": "Имя проекта (обязательно), например из v8md_projects"
                        },
                        "attributes_only": {
                            "type": "boolean",
                            "description": "Показать только атрибуты (реквизиты)"
                        },
                        "tabular_only": {
                            "type": "boolean",
                            "description": "Показать только табличные части"
                        }
                    },
                    "required": ["object", "project"]
                }
            },
            {
                "name": "v8md_types",
                "description": "Получить типы атрибута объекта 1С. Формат: Тип.Объект.Атрибут (например: Document.Заказ.Покупатель).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Путь к атрибуту, например: Document.Заказ.Покупатель"
                        },
                        "project": {
                            "type": "string",
                            "description": "Имя проекта (обязательно), например из v8md_projects"
                        }
                    },
                    "required": ["path", "project"]
                }
            },
            {
                "name": "v8md_list",
                "description": "Список всех объектов 1С в кэше проекта. Можно указать фильтр по типу.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "description": "Фильтр по типу: Catalog, Document, InformationRegister, AccumulationRegister, ChartOfCharacteristicTypes, Enum, Constant, DocumentJournal, ExchangePlan"
                        },
                        "project": {
                            "type": "string",
                            "description": "Имя проекта (обязательно), например из v8md_projects"
                        }
                    },
                    "required": ["project"]
                }
            }
        ]
    })
}

fn handle_tools_call(params: Value) -> Value {
    let tool_name = params.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let arguments = params.get("arguments")
        .cloned()
        .unwrap_or(json!({}));

    let (text, is_error) = match tool_name {
        "v8md_load_dump" => {
            let path = arguments.get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            let project = arguments.get("project")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            match commands::load_dump(&std::path::PathBuf::from(path), project) {
                Ok(msg) => (msg, false),
                Err(e) => (e, true),
            }
        }
        "v8md_projects" => match commands::projects_list() {
            Ok(output) => (output, false),
            Err(e) => (e, true),
        },
        "v8md_get" => {
            let object = arguments.get("object")
                .and_then(|o| o.as_str())
                .unwrap_or("");
            let project = arguments.get("project")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            let attributes_only = arguments.get("attributes_only")
                .and_then(|a| a.as_bool())
                .unwrap_or(false);
            let tabular_only = arguments.get("tabular_only")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            match commands::get_object(object, attributes_only, tabular_only, project) {
                Ok(output) => (output, false),
                Err(e) => (e, true),
            }
        }
        "v8md_types" => {
            let path = arguments.get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            let project = arguments.get("project")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            match commands::get_types(path, project) {
                Ok(output) => (output, false),
                Err(e) => (e, true),
            }
        }
        "v8md_list" => {
            let kind = arguments.get("kind")
                .and_then(|k| k.as_str())
                .map(|s| s.to_string());
            let project = arguments.get("project")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            match commands::list_objects(kind.as_deref(), project) {
                Ok(output) => (output, false),
                Err(e) => (e, true),
            }
        }
        _ => (format!("Unknown tool: {}", tool_name), true),
    };

    let mut result = json!({
        "content": [{"type": "text", "text": text}]
    });
    if is_error {
        result["isError"] = json!(true);
    }
    result
}