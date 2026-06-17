// ============================================================
// 🤖 Tao MCP Server — AI Agent 接入协议
// ============================================================
// 启动: tao mcp
// 然后在 AI 工具中配置:
// {
//   "mcpServers": {
//     "tao": { "command": "tao", "args": ["mcp"] }
//   }
// }
// ============================================================

use std::path::PathBuf;
use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};

// ============================================================
// MCP JSON-RPC 协议
// ============================================================

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ============================================================
// MCP Tool 定义
// ============================================================

const MCP_TOOLS: &str = r#"[
  {
    "name": "tao_put",
    "description": "写入/更新数据到 TaoStorage",
    "inputSchema": {
      "type": "object",
      "properties": {
        "key": {"type": "string", "description": "数据键名"},
        "value": {"type": "string", "description": "数据值 (支持 JSON)"},
        "tags": {"type": "array", "items": {"type": "string"}, "description": "标签"}
      },
      "required": ["key", "value"]
    }
  },
  {
    "name": "tao_get",
    "description": "读取数据",
    "inputSchema": {
      "type": "object",
      "properties": {
        "key": {"type": "string", "description": "数据键名"}
      },
      "required": ["key"]
    }
  },
  {
    "name": "tao_search",
    "description": "搜索数据",
    "inputSchema": {
      "type": "object",
      "properties": {
        "query": {"type": "string", "description": "搜索关键词"}
      },
      "required": ["query"]
    }
  },
  {
    "name": "tao_query",
    "description": "高级查询 (如 tag:critical, age>18)",
    "inputSchema": {
      "type": "object",
      "properties": {
        "expression": {"type": "string", "description": "查询表达式"}
      },
      "required": ["expression"]
    }
  },
  {
    "name": "tao_delete",
    "description": "删除数据",
    "inputSchema": {
      "type": "object",
      "properties": {
        "key": {"type": "string", "description": "数据键名"}
      },
      "required": ["key"]
    }
  },
  {
    "name": "tao_history",
    "description": "查看数据版本历史",
    "inputSchema": {
      "type": "object",
      "properties": {
        "key": {"type": "string", "description": "数据键名"}
      },
      "required": ["key"]
    }
  },
  {
    "name": "tao_stats",
    "description": "查看存储统计",
    "inputSchema": {"type": "object", "properties": {}}
  },
  {
    "name": "tao_invite",
    "description": "生成邀请码",
    "inputSchema": {
      "type": "object",
      "properties": {
        "node_id": {"type": "string", "description": "节点 ID"}
      },
      "required": ["node_id"]
    }
  },
  {
    "name": "tao_reputation",
    "description": "查看排行榜",
    "inputSchema": {"type": "object", "properties": {}}
  }
]"#;

// ============================================================
// MCP Server 实现
// ============================================================

pub fn run_mcp_server(data_dir: PathBuf) {
    eprintln!("🦀 Tao MCP Server 启动");
    eprintln!("   协议: JSON-RPC over stdio");
    eprintln!("   数据: {}", data_dir.display());
    eprintln!("   就绪: AI 工具可直接连接");

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: 0,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let _ = writeln!(stdout.lock(), "{}", serde_json::to_string(&err).unwrap());
                continue;
            }
        };

        let response = handle_request(&request, &data_dir);
        let _ = writeln!(stdout.lock(), "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.lock().flush();
    }

    eprintln!("🦀 Tao MCP Server 关闭");
}

fn handle_request(req: &JsonRpcRequest, data_dir: &PathBuf) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2025-03-26",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "tao-storage",
                    "version": "0.3.0"
                }
            })),
            error: None,
        },

        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(serde_json::json!({
                "tools": serde_json::from_str::<serde_json::Value>(MCP_TOOLS).unwrap_or_default()
            })),
            error: None,
        },

        "tools/call" => {
            let args = req.params.as_ref()
                .and_then(|p| p.get("arguments"))
                .and_then(|a| a.as_object())
                .cloned()
                .unwrap_or_default();

            let tool_name = req.params.as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");

            let result = execute_tool(tool_name, &args, data_dir);

            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": result
                    }]
                })),
                error: None,
            }
        }

        "notifications/initialized" => {
            // 忽略通知
            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({})),
                error: None,
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        },
    }
}

fn execute_tool(name: &str, args: &serde_json::Map<String, serde_json::Value>, data_dir: &PathBuf) -> String {
    let store_path = data_dir.join("store");
    let db = match sled::open(&store_path) {
        Ok(d) => d,
        Err(e) => return format!("❌ 存储错误: {}", e),
    };

    match name {
        "tao_put" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let tags: Vec<String> = args.get("tags")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            if key.is_empty() || value.is_empty() {
                return "❌ key 和 value 不能为空".into();
            }

            let owner = [0u8; 32];
            let unit = tao_core::DataUnit::new(value.as_bytes().to_vec(), key.to_string(), owner);
            let id = unit.id();

            if let Ok(bytes) = bincode::serialize(&unit) {
                if db.insert(&unit.yin.content_hash, bytes).is_ok() {
                    let _ = db.flush();
                    return format!("✅ 已存储 [{}] → {}", key, id);
                }
            }
            "❌ 存储失败".into()
        }

        "tao_get" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            if key.is_empty() {
                return "❌ key 不能为空".into();
            }

            for item in db.iter() {
                if let Ok((_, value)) = item {
                    if let Ok(unit) = bincode::deserialize::<tao_core::DataUnit>(&value) {
                        if unit.yang.name == key {
                            let payload = String::from_utf8_lossy(&unit.yin.payload);
                            return format!("{}", payload);
                        }
                    }
                }
            }
            format!("❌ '{}' 未找到", key)
        }

        "tao_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            if query.is_empty() {
                return "❌ 搜索词不能为空".into();
            }

            let mut results = Vec::new();
            for item in db.iter() {
                if let Ok((_, value)) = item {
                    if let Ok(unit) = bincode::deserialize::<tao_core::DataUnit>(&value) {
                        let payload = String::from_utf8_lossy(&unit.yin.payload).to_lowercase();
                        let name = unit.yang.name.to_lowercase();
                        let tags = unit.yang.tags.iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" ");
                        if name.contains(&query) || payload.contains(&query) || tags.contains(&query) {
                            results.push(unit.yang.name.clone());
                        }
                    }
                }
            }

            if results.is_empty() {
                return format!("📭 未找到匹配 '{}'", query);
            }
            format!("🔍 找到 {} 条:\n{}", results.len(), results.join("\n"))
        }

        "tao_query" => {
            let expr = args.get("expression").and_then(|v| v.as_str()).unwrap_or("");
            // 简化版: 直接返回
            format!("🔍 查询 '{}' (完整查询引擎待构建)", expr)
        }

        "tao_delete" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            if key.is_empty() { return "❌ key 不能为空".into(); }

            let mut deleted = 0u64;
            let to_remove: Vec<[u8; 32]> = db.iter()
                .filter_map(|r| r.ok())
                .filter_map(|(k, v)| {
                    bincode::deserialize::<tao_core::DataUnit>(&v).ok().and_then(|unit| {
                        if unit.yang.name == key { Some(*k.as_ref()) } else { None }
                    })
                })
                .collect();

            for hash in &to_remove {
                if db.remove(hash).is_ok() { deleted += 1; }
            }
            let _ = db.flush();
            format!("✅ 已删除 {} 条 '{}'", deleted, key)
        }

        "tao_history" => {
            format!("⏳ 版本历史功能待构建")
        }

        "tao_stats" => {
            let count = db.iter().count();
            format!("📊 总条数: {}", count)
        }

        "tao_invite" => {
            let node_id = args.get("node_id").and_then(|v| v.as_str()).unwrap_or("mcp-node");
            format!("🔗 邀请码已生成 (功能待集成): {}", node_id)
        }

        "tao_reputation" => {
            format!("🏆 排行榜功能待集成")
        }

        _ => format!("❌ 未知工具: {}", name),
    }
}
