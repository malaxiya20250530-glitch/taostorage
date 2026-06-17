use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tao_core::*;

mod daemon;
mod invite;
mod mcp;
mod history;
mod query;

// ============================================================
// Tao CLI — 统一命令行接口
// ============================================================

#[derive(Parser, Debug)]
#[command(
    name = "tao",
    version = "0.3.0",
    about = "道存储 — 个人数据仓库 CLI + 分布式网络节点",
    long_about = concat!(
        "道可道，非常道 — AI Agent 知识数据库 + P2P 存储网络\n",
        "The Tao that can be told is not the eternal Tao.\n\n",
        "示例 / Examples:\n",
        "  tao put note \"hello world\" --tag demo                    写入数据\n",
        "  tao get note                                                读取数据\n",
        "  tao query \"tag:critical AND age>18\"                      高级查询\n",
        "  tao search hello                                            全文搜索\n",
        "  tao history note                                            版本历史\n",
        "  tao rollback note 2                                         回滚版本\n",
        "  tao mcp                                                     MCP Server\n",
        "  tao daemon start --background                               后台启动\n",
        "  tao daemon status                                           查看状态\n"
    )
)]
struct Cli {
    #[arg(short = 'd', long, default_value = "~/.taostorage", help = "数据目录")]
    data_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 写入/更新一条数据
    Put {
        key: String,
        value: String,
        #[arg(short = 't', long, help = "标签（可多次使用）")]
        tag: Vec<String>,
    },
    /// 读取指定 key 的最新数据
    Get {
        key: String,
    },
    /// 列出所有 key（可选按 key 过滤）
    List {
        key: Option<String>,
    },
    /// 模糊搜索（搜索 key/value/tag）
    Search {
        query: String,
        #[arg(long, default_value = "20", help = "最大结果数")]
        limit: usize,
    },
    /// 按标签查询
    ByTag {
        tag: Vec<String>,
        #[arg(long, help = "AND 模式（同时包含所有标签）")]
        all: bool,
        #[arg(long, help = "OR 模式（包含任一标签，默认）")]
        any: bool,
    },
    /// 标签云（按使用次数排序显示）
    TagCloud,
    /// 删除数据
    Delete {
        key: String,
        #[arg(long, help = "按 ID 删除（hex content_hash）")]
        id: Option<String>,
    },
    /// 导出备份为 JSON（v0.2 兼容格式）
    Export {
        #[arg(default_value = "tao-backup.json")]
        path: String,
    },
    /// 从 JSON 备份导入
    Import {
        path: String,
    },
    /// 显示存储统计信息
    Stats,
    /// 生成邀请码
    Invite {
        #[arg(help = "操作: generate / use / leaderboard / stats")]
        action: String,
        #[arg(help = "参数: node_id 或 invite_code")]
        args: Vec<String>,
    },
    /// 查看信誉/排行榜
    Reputation {
        #[arg(help = "节点 ID (留空显示排行榜)")]
        node_id: Option<String>,
    },
    /// 高级查询 (DSL: tag:critical, age>18, time>2026-01)
    Query {
        #[arg(help = "查询表达式")]
        expression: String,
    },
    /// 查看数据版本历史
    History {
        #[arg(help = "数据键名")]
        key: String,
    },
    /// 回滚到指定版本
    Rollback {
        #[arg(help = "数据键名")]
        key: String,
        #[arg(help = "版本号")]
        version: u64,
    },
    /// 比较两个版本差异
    Diff {
        #[arg(help = "数据键名")]
        key: String,
        #[arg(help = "版本号 1")]
        v1: u64,
        #[arg(help = "版本号 2")]
        v2: u64,
    },
    /// 启动 MCP Server (AI 协议接口)
    Mcp,
    /// 启动浏览器节点 (HTTP + WebSocket)
    Browser {
        #[arg(short = 'p', long, default_value = "3000", help = "HTTP 端口")]
        port: u16,
        #[arg(short = 'w', long, default_value = "3001", help = "WebSocket 端口")]
        ws_port: u16,
    },
        /// 管理 Tao 守护进程（HTTP API + 可选 P2P 网络）
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand, Debug)]
enum DaemonAction {
    /// 启动 Daemon（前台运行）
    Start {
        #[arg(short = 'l', long, default_value = "/ip4/0.0.0.0/tcp/0", help = "P2P 监听地址")]
        listen: String,
        #[arg(short = 'p', long, default_value = "8788", help = "HTTP API 端口")]
        port: u16,
        #[arg(short = 'b', long, help = "P2P 引导节点地址")]
        bootstrap: Vec<String>,
        #[arg(long, help = "启用 P2P 网络（默认仅本地模式）")]
        network: bool,
        #[arg(long, help = "后台运行（fork 子进程）")]
        background: bool,
    },
    /// 查看 Daemon 运行状态
    Status,
    /// 停止 Daemon
    Stop,
    /// 重启 Daemon
    Restart {
        #[arg(short = 'l', long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen: String,
        #[arg(short = 'p', long, default_value = "8788", help = "HTTP API 端口")]
        port: u16,
        #[arg(short = 'b', long)]
        bootstrap: Vec<String>,
        #[arg(long, help = "启用 P2P 网络")]
        network: bool,
        #[arg(long, help = "后台运行")]
        background: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data_dir = expand_tilde(&cli.data_dir);
    let path = PathBuf::from(&data_dir);

    match cli.command {
        Commands::Put { key, value, tag } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_put(&store, &tag_index, &key, &value, &tag)?;
            // 自动记录版本历史
            history::cmd_record_version(&path, &key, &value, &tag, "put");
        }
        Commands::Get { key } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_get(&store, &tag_index, &key)?;
        }
        Commands::List { key } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_list(&store, &tag_index, key.as_deref())?;
        }
        Commands::Search { query, limit } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            cmd_search(&store, &query, limit)?;
        }
        Commands::ByTag { tag, all, any: _ } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            if all && !tag.is_empty() {
                cmd_by_tags_all(&store, &tag_index, &tag)?;
            } else {
                cmd_by_tags_any(&store, &tag_index, &tag)?;
            }
        }
        Commands::TagCloud => {
            std::fs::create_dir_all(&path)?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_tag_cloud(&tag_index)?;
        }
        Commands::Delete { key, id } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_delete(&store, &tag_index, &key, id.as_deref())?;
        }
        Commands::Export { path: export_path } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_export(&store, &tag_index, &export_path)?;
        }
        Commands::Import { path: import_path } => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_import(&store, &tag_index, &import_path)?;
        }
        Commands::Stats => {
            std::fs::create_dir_all(&path)?;
            let store = sled::open(path.join("store"))?;
            let tag_index = TagIndex::open(path.join("tags"))?;
            cmd_stats(&store, &tag_index)?;
        }
        Commands::Daemon { action } => {
            match action {
                DaemonAction::Start { listen, port, bootstrap, network, background } => {
                    if background {
                        daemon::start_daemon_background(&data_dir, &listen, port, &bootstrap)?;
                    } else {
                        let rt = tokio::runtime::Runtime::new()?;
                        rt.block_on(daemon::run_daemon(&data_dir, &listen, port, &bootstrap, network))?;
                    }
                }
                DaemonAction::Status => {
                    let status = daemon::check_status(&data_dir);
                    println!("\n📡 Tao Daemon Status");
                    println!("  {:-<40}", "");
                    println!("  Running:   {}", if status.running { "✅ Yes" } else { "❌ No" });
                    if let Some(pid) = status.pid {
                        println!("  PID:       {}", pid);
                    }
                    if let Some(dir) = status.data_dir {
                        println!("  Data Dir:  {}", dir);
                    }
                    if let Some(port) = status.api_port {
                        println!("  API Port:  {}", port);
                    }
                    if let Some(p) = status.peers {
                        println!("  Peers:     {}", p);
                    }
                    if let Some(o) = status.objects {
                        println!("  Objects:   {}", o);
                    }
                    if let Some(u) = status.uptime_secs {
                        println!("  Uptime:    {}s", u);
                    }
                    println!("  {:-<40}", "");
                    if status.running {
                        println!("  💡 Use: tao daemon stop  to stop");
                    } else {
                        println!("  💡 Use: tao daemon start --background  to start");
                    }
                    println!();
                }
                DaemonAction::Stop => {
                    match daemon::stop_daemon(&data_dir) {
                        Ok(()) => println!("✅ Daemon stopped"),
                        Err(e) => println!("❌ {}", e),
                    }
                }
                DaemonAction::Restart { listen, port, bootstrap, network, background } => {
                    println!("🔄 Restarting daemon...");
                    let _ = daemon::stop_daemon(&data_dir);
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if background {
                        daemon::start_daemon_background(&data_dir, &listen, port, &bootstrap)?;
                    } else {
                        let rt = tokio::runtime::Runtime::new()?;
                        rt.block_on(daemon::run_daemon(&data_dir, &listen, port, &bootstrap, network))?;
                    }
                }
            }
        }
    }

    Ok(())
}

// ============================================================
// CLI 命令实现（复用之前逻辑）
// ============================================================

fn cmd_put(store: &sled::Db, tag_index: &TagIndex, key: &str, value: &str, tags: &[String]) -> anyhow::Result<()> {
    let payload = value.as_bytes().to_vec();
    let owner = [0u8; 32];
    let mut unit = DataUnit::new(payload, key.to_string(), owner);
    unit.yang.tags = tags.to_vec();
    let content_hash = unit.yin.content_hash;
    let id = unit.id();
    let bytes = bincode::serialize(&unit)?;
    store.insert(&content_hash, bytes)?;
    store.flush()?;
    if !tags.is_empty() {
        tag_index.add_tags_bidirectional(&content_hash, tags)?;
    }
    println!("✅ 已存储 / Stored [{}] → {} [{}]", key, id, tags.join(", "));
    Ok(())
}

fn cmd_get(store: &sled::Db, _tag_index: &TagIndex, key: &str) -> anyhow::Result<()> {
    let mut found = false;
    for item in store.iter() {
        let (_k, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            if unit.yang.name == key {
                found = true;
                let value_str = String::from_utf8_lossy(&unit.yin.payload);
                let id = unit.id();
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  ID:     {}", id);
                println!("  Key:    {}", unit.yang.name);
                println!("  Value:  {}", value_str);
                println!("  Tags:   [{}]", unit.yang.tags.join(", "));
                println!("  Heat:   {}", unit.yang.heat);
                println!("  Hexagram: {:?}", unit.qi.hexagram);
                println!("  Created: {}", timestamp_str(unit.yang.created_at));
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
        }
    }
    if !found {
        println!("❌ 未找到 / Not found: [{}]", key);
    }
    Ok(())
}

fn cmd_list(store: &sled::Db, _tag_index: &TagIndex, filter_key: Option<&str>) -> anyhow::Result<()> {
    let mut results: Vec<(String, String, u64, Vec<String>)> = Vec::new();
    for item in store.iter() {
        let (_k, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            if let Some(filter) = filter_key {
                if unit.yang.name != filter { continue; }
            }
            let id = hex::encode(&unit.yin.content_hash[..8]);
            results.push((unit.yang.name.clone(), id, unit.yang.created_at, unit.yang.tags.clone()));
        }
    }
    results.sort_by(|a, b| b.2.cmp(&a.2));
    if results.is_empty() {
        println!("📭 空 / Empty");
        return Ok(());
    }
    println!("\n  {:<20} {:<20} {:<12} Tags", "Key", "ID (prefix)", "Created");
    println!("  {:-<20} {:-<20} {:-<12} {:-}", "", "", "", "");
    for (key, id_prefix, ts, tags) in &results {
        let tag_str = if tags.is_empty() { String::new() } else { format!("[{}]", tags.join(", ")) };
        println!("  {:<20} {:<20} {:<12} {}", key, format!("…{}", id_prefix), timestamp_str(*ts), tag_str);
    }
    println!("\n  📊 共 {} 条 / Total items", results.len());
    Ok(())
}

fn cmd_search(store: &sled::Db, query: &str, max_results: usize) -> anyhow::Result<()> {
    let hits = fuzzy_search(store, query, max_results)?;
    if hits.is_empty() {
        println!("🔍 未找到匹配 / No matches for: '{}'", query);
        return Ok(());
    }
    println!("\n🔍 搜索 / Search: '{}' — 找到 {} 条结果", query, hits.len());
    println!("  {:-<60}", "");
    for hit in &hits {
        let snippet = if hit.value_snippet.len() > 40 { &hit.value_snippet[..40] } else { &hit.value_snippet };
        println!("  [{}] {} ({})  ← {}", hit.key, snippet, hit.tags.join(","), hit.match_field);
    }
    Ok(())
}

fn cmd_by_tags_any(store: &sled::Db, tag_index: &TagIndex, tags: &[String]) -> anyhow::Result<()> {
    let hash_hexes = tag_index.get_by_tags_any(tags)?;
    display_by_hash_hexes(store, &hash_hexes, "OR")
}

fn cmd_by_tags_all(store: &sled::Db, tag_index: &TagIndex, tags: &[String]) -> anyhow::Result<()> {
    let hash_hexes = tag_index.get_by_tags_all(tags)?;
    display_by_hash_hexes(store, &hash_hexes, "AND")
}

fn display_by_hash_hexes(store: &sled::Db, hash_hexes: &[String], mode: &str) -> anyhow::Result<()> {
    if hash_hexes.is_empty() {
        println!("🔍 无匹配结果（{} 模式）", mode);
        return Ok(());
    }
    println!("\n🏷️  标签查询 / By Tag ({}): {} 条结果", mode, hash_hexes.len());
    println!("  {:-<60}", "");
    for item in store.iter() {
        let (_k, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            let id = hex::encode(unit.yin.content_hash);
            if hash_hexes.contains(&id) {
                let value_str = String::from_utf8_lossy(&unit.yin.payload);
                let snippet = if value_str.len() > 50 { &value_str[..50] } else { &value_str };
                println!("  [{}] {}  tags=[{}]", unit.yang.name, snippet, unit.yang.tags.join(","));
            }
        }
    }
    Ok(())
}

fn cmd_tag_cloud(tag_index: &TagIndex) -> anyhow::Result<()> {
    let cloud = tag_index.tag_cloud()?;
    if cloud.is_empty() {
        println!("📭 暂无标签 / No tags");
        return Ok(());
    }
    let max_count = cloud.first().map(|(_, c)| *c).unwrap_or(1).max(1);
    let bar_width = 30;
    println!("\n📊 标签云 / Tag Cloud");
    println!("  {:-<50}", "");
    for (tag, count) in &cloud {
        let bar_len = (*count * bar_width / max_count).max(1);
        let bar = "█".repeat(bar_len);
        println!("  {:>12} │ {:<30} {} 次", tag, bar, count);
    }
    println!("  {:-<50}", "");
    println!("  🏷️  共 {} 个标签", cloud.len());
    Ok(())
}

fn cmd_delete(store: &sled::Db, tag_index: &TagIndex, key: &str, id: Option<&str>) -> anyhow::Result<()> {
    let mut deleted = 0usize;
    if let Some(id_hex) = id {
        let content_hash = hex_to_hash(id_hex)?;
        tag_index.remove_hash(&content_hash)?;
        store.remove(&content_hash)?;
        deleted = 1;
        println!("🗑️  已删除 ID: {}", id_hex);
    } else {
        let to_remove: Vec<[u8; 32]> = store.iter()
            .filter_map(|r| r.ok())
            .filter_map(|(_k, value)| {
                bincode::deserialize::<DataUnit>(&value).ok().and_then(|unit| {
                    if unit.yang.name == key { Some(unit.yin.content_hash) } else { None }
                })
            })
            .collect();
        for hash in &to_remove {
            tag_index.remove_hash(hash)?;
            store.remove(hash)?;
            deleted += 1;
        }
    }
    store.flush()?;
    println!("✅ 已删除 {} 条 / Deleted {} items with key [{}]", deleted, deleted, key);
    Ok(())
}

fn cmd_export(store: &sled::Db, tag_index: &TagIndex, path: &str) -> anyhow::Result<()> {
    let json = export_backup(store, tag_index)?;
    std::fs::write(path, &json)?;
    let count = store.iter().count();
    println!("✅ 已导出到 / Exported to: {}", path);
    println!("   {} items", count);
    Ok(())
}

fn cmd_import(store: &sled::Db, tag_index: &TagIndex, path: &str) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(path)?;
    let result = import_backup(store, tag_index, &json)?;
    println!("✅ 导入完成 / Import complete:");
    println!("   导入: {} 条", result.imported);
    println!("   跳过(已存在): {} 条", result.skipped);
    Ok(())
}

fn cmd_stats(store: &sled::Db, tag_index: &TagIndex) -> anyhow::Result<()> {
    let stats = collect_stats(store, tag_index, 10)?;
    println!("\n📊 存储统计 / Store Statistics");
    println!("  {:-<40}", "");
    println!("  总条数 / Total Items:    {}", stats.total_items);
    println!("  唯一 Key / Unique Keys:  {}", stats.unique_keys);
    println!("  总标签数 / Total Tags:    {}", stats.total_tags);
    println!();
    if !stats.top_keys.is_empty() {
        println!("  🔝 热门 Key 排行:");
        for (i, (key, count)) in stats.top_keys.iter().enumerate() {
            println!("    {}. {} ({} 条)", i + 1, key, count);
        }
        println!();
    }
    if !stats.top_tags.is_empty() {
        println!("  🔝 热门标签排行:");
        println!("  {:-<40}", "");
        for (i, (tag, count)) in stats.top_tags.iter().enumerate() {
            println!("    {}. {} ({} 次)", i + 1, tag, count);
        }
    }
    Ok(())
}

// ============================================================
// 工具函数
// ============================================================

        Commands::Invite { action, args } => {
            match action.as_str() {
                "generate" | "gen" => {
                    let node_id = args.first().map(|s| s.as_str()).unwrap_or("anonymous");
                    invite::cmd_generate(&path, node_id);
                }
                "use" => {
                    if args.len() < 2 {
                        println!("用法: tao invite use <code> <node_id>");
                    } else {
                        invite::cmd_use_code(&path, &args[0], &args[1]);
                    }
                }
                "leaderboard" | "lb" | "rank" => {
                    invite::cmd_leaderboard(&path);
                }
                "stats" | "status" => {
                    let node_id = args.first().map(|s| s.as_str()).unwrap_or("anonymous");
                    invite::cmd_reputation(&path, node_id);
                }
                _ => {
                    println!("邀请操作: generate / use / leaderboard / stats");
                }
            }
        }
        Commands::Reputation { node_id } => {
            match node_id {
                Some(id) => invite::cmd_reputation(&path, &id),
                None => invite::cmd_leaderboard(&path),
            }
        }
        Commands::Query { expression } => {
            query::cmd_query(&path, &expression);
        }
        Commands::History { key } => {
            history::cmd_history(&path, &key);
        }
        Commands::Rollback { key, version } => {
            history::cmd_rollback(&path, &key, version);
        }
        Commands::Diff { key, v1, v2 } => {
            history::cmd_diff(&path, &key, v1, v2);
        }
        Commands::Mcp => {
            mcp::run_mcp_server(path);
        }
        Commands::Browser { port, ws_port } => {
            // 启动浏览器节点服务器
            println!("🌐 TaoStorage 浏览器节点服务器启动中...");
            println!("   HTTP:  http://0.0.0.0:{}", port);
            println!("   WS:    ws://0.0.0.0:{}", ws_port);
            println!("   按 Ctrl+C 停止");

            // 查找 www 目录
            let www_dir = find_www_dir();

            match www_dir {
                Some(dir) => {
                    let server_js = dir.join("signaling-server").join("server.js");
                    if server_js.exists() {
                        let status = std::process::Command::new("node")
                            .arg(&server_js)
                            .env("PORT", port.to_string())
                            .env("WS_PORT", ws_port.to_string())
                            .status();
                        match status {
                            Ok(_) => {},
                            Err(e) => {
                                println!("⚠️ 无法启动 Node.js 服务器: {}", e);
                                println!("   请先安装 Node.js，然后手动运行:");
                                println!("   cd www && npm install && npm start");
                            }
                        }
                    } else {
                        println!("⚠️ 找不到 signaling-server (需要初始化):");
                        println!("   cd www/signaling-server && npm install && npm start");
                    }
                }
                None => {
                    println!("⚠️ 找不到 www 目录 (需从项目根目录运行)");
                    println!("   手动启动: cd www/signaling-server && npm install && npm start");
                }
            }
        }


/// 查找 www 目录（项目根目录下的 www/）
fn find_www_dir() -> Option<std::path::PathBuf> {
    // 1. 当前目录
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("www");
        if candidate.is_dir() { return Some(candidate); }
    }
    // 2. 可执行文件同级
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("www");
            if candidate.is_dir() { return Some(candidate); }
            // 3. 上级目录（target/release/ 情况）
            if let Some(grand) = parent.parent() {
                let candidate = grand.join("www");
                if candidate.is_dir() { return Some(candidate); }
            }
        }
    }
    // 4. HOME 目录下的 .taostorage
    if let Ok(home) = std::env::var("HOME") {
        let candidate = std::path::PathBuf::from(home)
            .join(".taostorage").join("www");
        if candidate.is_dir() { return Some(candidate); }
    }
    None
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        if path == "~" { home } else { format!("{}/{}", home, &path[2..]) }
    } else {
        path.to_string()
    }
}

fn timestamp_str(secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let diff = now.saturating_sub(secs);
    if diff < 60 { format!("{}s ago", diff) }
    else if diff < 3600 { format!("{}m ago", diff / 60) }
    else if diff < 86400 { format!("{}h ago", diff / 3600) }
    else { format!("{}d ago", diff / 86400) }
}

fn hex_to_hash(hex_str: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        anyhow::bail!("ID 必须是 32 字节的 hex 字符串");
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}
