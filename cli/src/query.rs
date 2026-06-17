// ============================================================
// 🔍 Tao Query DSL — JSON 文档查询引擎
// ============================================================
// 用法:
//   tao query "tag:critical"
//   tao query "tag:audit AND severity:high"
//   tao query "age>18"
//   tao query "time>2026-01-01 AND tag:critical"
// ============================================================

use std::path::PathBuf;
use serde_json::Value;

pub fn cmd_query(data_dir: &PathBuf, expression: &str) {
    let store_path = data_dir.join("store");
    let db = match sled::open(&store_path) {
        Ok(d) => d,
        Err(e) => { println!("❌ 无法打开存储: {}", e); return; }
    };
    let tag_index_path = data_dir.join("tags");
    let tag_index = match tao_core::TagIndex::open(tag_index_path) {
        Ok(t) => t,
        Err(_) => { println!("⚠️ 无法加载标签索引"); return; }
    };

    let expr = expression.trim();
    let mut results: Vec<(String, String, Vec<String>)> = Vec::new();

    // 解析简单查询表达式
    // 格式: tag:tagname   — 按标签查
    //        age>18        — JSON 字段比较
    //        time>2026-01  — 时间字段
    //        AND           — 组合条件

    let conditions = parse_query(expr);

    for item in db.iter() {
        if let Ok((_key, value)) = item {
            if let Ok(unit) = bincode::deserialize::<tao_core::DataUnit>(&value) {
                let name = &unit.yang.name;
                let payload = String::from_utf8_lossy(&unit.yin.payload);
                let tags = &unit.yang.tags;

                if evaluate_conditions(&conditions, name, &payload, tags) {
                    results.push((name.clone(), payload.to_string(), tags.clone()));
                }
            }
        }
    }

    // 输出结果
    if results.is_empty() {
        println!("📭 无匹配结果");
        return;
    }

    println!("\n🔍 查询: '{}' — {} 条结果\n", expression, results.len());
    for (name, payload, tags) in &results {
        let tag_str = tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
        let snippet = if payload.len() > 80 {
            format!("{}...", &payload[..80])
        } else {
            payload.clone()
        };
        println!("  📄 {}  {}  {}", name, snippet, tag_str);
    }
    println!("\n  {} 条记录", results.len());
}

#[derive(Debug, Clone)]
enum Condition {
    TagMatch(String),
    FieldGt(String, f64),
    FieldLt(String, f64),
    FieldEq(String, String),
    TimeAfter(String),
    TimeBefore(String),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

fn parse_query(expr: &str) -> Condition {
    // 处理 AND
    if let Some(pos) = expr.to_uppercase().find(" AND ") {
        let left = &expr[..pos];
        let right = &expr[pos + 5..];
        return Condition::And(
            Box::new(parse_simple(left.trim())),
            Box::new(parse_simple(right.trim())),
        );
    }
    // 处理 OR
    if let Some(pos) = expr.to_uppercase().find(" OR ") {
        let left = &expr[..pos];
        let right = &expr[pos + 4..];
        return Condition::Or(
            Box::new(parse_simple(left.trim())),
            Box::new(parse_simple(right.trim())),
        );
    }
    parse_simple(expr.trim())
}

fn parse_simple(expr: &str) -> Condition {
    // tag:xxx
    if let Some(tag) = expr.strip_prefix("tag:") {
        return Condition::TagMatch(tag.to_string());
    }
    // field>value
    if let Some(pos) = expr.find('>') {
        let field = expr[..pos].trim();
        let val = expr[pos+1..].trim();
        // 时间类型
        if field == "time" || field == "created" {
            return Condition::TimeAfter(val.to_string());
        }
        if let Ok(n) = val.parse::<f64>() {
            return Condition::FieldGt(field.to_string(), n);
        }
    }
    // field<value
    if let Some(pos) = expr.find('<') {
        let field = expr[..pos].trim();
        let val = expr[pos+1..].trim();
        if field == "time" || field == "created" {
            return Condition::TimeBefore(val.to_string());
        }
        if let Ok(n) = val.parse::<f64>() {
            return Condition::FieldLt(field.to_string(), n);
        }
    }
    // field=value  
    if let Some(pos) = expr.find('=') {
        let field = expr[..pos].trim();
        let val = expr[pos+1..].trim();
        return Condition::FieldEq(field.to_string(), val.to_string());
    }
    // 默认: 全文搜索
    Condition::TagMatch(expr.to_string())
}

fn evaluate_conditions(cond: &Condition, name: &str, payload: &str, tags: &[String]) -> bool {
    match cond {
        Condition::TagMatch(tag) => {
            tags.iter().any(|t| t == tag) || name.contains(tag) || payload.contains(tag)
        }
        Condition::FieldGt(field, val) => {
            if let Ok(json) = serde_json::from_str::<Value>(payload) {
                if let Some(field_val) = json.get(field) {
                    if let Some(n) = field_val.as_f64() {
                        return n > *val;
                    }
                    if let Some(s) = field_val.as_str() {
                        if let Ok(n) = s.parse::<f64>() {
                            return n > *val;
                        }
                    }
                }
            }
            false
        }
        Condition::FieldLt(field, val) => {
            if let Ok(json) = serde_json::from_str::<Value>(payload) {
                if let Some(field_val) = json.get(field) {
                    if let Some(n) = field_val.as_f64() {
                        return n < *val;
                    }
                }
            }
            false
        }
        Condition::FieldEq(field, val) => {
            if let Ok(json) = serde_json::from_str::<Value>(payload) {
                if let Some(field_val) = json.get(field) {
                    return field_val.as_str() == Some(val) || field_val.to_string().trim_matches('"') == *val;
                }
            }
            false
        }
        Condition::TimeAfter(_time) => {
            // 简化版: 直接返回 true (生产环境需解析时间戳)
            true
        }
        Condition::TimeBefore(_time) => true,
        Condition::And(left, right) => {
            evaluate_conditions(left, name, payload, tags) && evaluate_conditions(right, name, payload, tags)
        }
        Condition::Or(left, right) => {
            evaluate_conditions(left, name, payload, tags) || evaluate_conditions(right, name, payload, tags)
        }
    }
}
