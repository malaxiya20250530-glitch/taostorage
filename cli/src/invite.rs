// ============================================================
// 🧲 TaoStorage 邀请奖励系统 — CLI 接口
// ============================================================

pub const INVITE_FILE: &str = "invites.json";
pub const REPUTATION_FILE: &str = "reputation.json";

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRecord {
    pub code: String,
    pub inviter: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub uses: u32,
    pub max_uses: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReputationRecord {
    pub score: u64,
    pub invites: u64,
    pub events: Vec<ReputationEvent>,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: String,
    pub event: String,
    pub delta: i64,
    pub description: String,
}

type InviteDb = HashMap<String, InviteRecord>;
type ReputationDb = HashMap<String, ReputationRecord>;

fn load_json<T: serde::de::DeserializeOwned>(path: &PathBuf, default: T) -> T {
    if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(default)
    } else {
        default
    }
}

fn save_json<T: serde::Serialize>(path: &PathBuf, data: &T) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(path, json);
    }
}

pub fn cmd_generate(data_dir: &PathBuf, node_id: &str) {
    let path = data_dir.join("invites").join(INVITE_FILE);
    let mut invites: InviteDb = load_json(&path, InviteDb::new());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 生成 8 字符邀请码
    let raw = format!("{}:{}:{}", node_id, now, rand::random::<u64>());
    let hash = sha2::Sha256::digest(raw.as_bytes());
    let code = hex::encode(&hash[..4]).to_uppercase();

    let record = InviteRecord {
        code: code.clone(),
        inviter: node_id.to_string(),
        created_at: now,
        expires_at: now + 7 * 24 * 3600,
        uses: 0,
        max_uses: 0,
        active: true,
    };

    invites.insert(code.clone(), record);
    save_json(&path, &invites);

    println!("✅ 邀请码已生成: {}", code);
    println!("   邀请链接: https://tao.storage/?invite={}", code);
    println!("   有效期: 7 天");
    println!("   每邀请一人，信誉 +10 🏆");
}

pub fn cmd_use_code(data_dir: &PathBuf, code: &str, new_node_id: &str) {
    let invites_path = data_dir.join("invites").join(INVITE_FILE);
    let reps_path = data_dir.join("invites").join(REPUTATION_FILE);

    let mut invites: InviteDb = load_json(&invites_path, InviteDb::new());
    let mut reps: ReputationDb = load_json(&reps_path, ReputationDb::new());

    let record = match invites.get_mut(code) {
        Some(r) if r.active => r,
        Some(_) => { println!("❌ 邀请码已失效"); return; }
        None => { println!("❌ 邀请码无效: {}", code); return; }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now > record.expires_at {
        record.active = false;
        save_json(&invites_path, &invites);
        println!("❌ 邀请码已过期");
        return;
    }

    record.uses += 1;
    let inviter_id = record.inviter.clone();
    let now_str = format!("{}", now);

    // 邀请方 +10
    let inviter_rep = reps.entry(inviter_id.clone()).or_default();
    inviter_rep.score += 10;
    inviter_rep.invites += 1;
    inviter_rep.events.push(ReputationEvent {
        timestamp: now_str.clone(),
        event: "invite".into(),
        delta: 10,
        description: format!("邀请节点 {}", new_node_id),
    });

    // 新节点 +5
    let new_rep = reps.entry(new_node_id.to_string()).or_default();
    new_rep.score += 5;
    new_rep.events.push(ReputationEvent {
        timestamp: now_str,
        event: "joined".into(),
        delta: 5,
        description: format!("通过邀请码 {} 加入", code),
    });

    // 检查徽章
    check_badges(&mut reps, &inviter_id);
    check_badges(&mut reps, new_node_id);

    save_json(&invites_path, &invites);
    save_json(&reps_path, &reps);

    println!("🎉 邀请成功！");
    println!("   邀请方 {}: 信誉 +10 🏆", inviter_id);
    println!("   新节点 {}: 信誉 +5 🏆", new_node_id);
}

pub fn cmd_leaderboard(data_dir: &PathBuf) {
    let reps_path = data_dir.join("invites").join(REPUTATION_FILE);
    let reps: ReputationDb = load_json(&reps_path, ReputationDb::new());

    let mut entries: Vec<(&String, &ReputationRecord)> = reps.iter().collect();
    entries.sort_by(|a, b| b.1.score.cmp(&a.1.score));

    println!("\n🏆 TaoStorage 节点排行榜");
    println!("{}", "─".repeat(50));

    for (i, (node_id, rep)) in entries.iter().take(20).enumerate() {
        let icon = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "   ",
        };
        let rank = rank_name(rep.score);
        println!("  {} #{} {:<20} 🏆 {:>4} {} 🤝 {}邀",
            icon, i + 1, node_id, rep.score, rank, rep.invites);
    }

    println!("\n  等级: 凡(0) → 士(10) → 道(50) → 玄(200) → 圣(1000)");
}

pub fn cmd_reputation(data_dir: &PathBuf, node_id: &str) {
    let reps_path = data_dir.join("invites").join(REPUTATION_FILE);
    let reps: ReputationDb = load_json(&reps_path, ReputationDb::new());

    let rep = reps.get(node_id).cloned().unwrap_or_default();

    println!("\n📊 节点 {} 统计", node_id);
    println!("{}", "─".repeat(40));
    println!("  信誉分:   {} 🏆", rep.score);
    println!("  等级:     {}", rank_name(rep.score));
    println!("  邀请数:   {} 🤝", rep.invites);

    if !rep.badges.is_empty() {
        print!("  徽章:     ");
        for b in &rep.badges { print!("[{}] ", b); }
        println!();
    }

    println!("\n  最近活动:");
    for event in rep.events.iter().rev().take(5) {
        println!("    {} {}", event.timestamp, event.description);
    }
}

fn check_badges(reps: &mut ReputationDb, node_id: &str) {
    let rep = reps.get(node_id);
    let (score, invites) = match rep {
        Some(r) => (r.score, r.invites),
        None => return,
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let rep = reps.get_mut(node_id).unwrap();

    if invites >= 100 && !rep.badges.contains(&"宗师".into()) {
        rep.badges.push("宗师".into());
        rep.score += 50;
        rep.events.push(ReputationEvent {
            timestamp: now.clone(),
            event: "badge".into(),
            delta: 50,
            description: "获得徽章: 宗师 (邀请100人)".into(),
        });
    } else if invites >= 10 && !rep.badges.contains(&"传道者".into()) {
        rep.badges.push("传道者".into());
        rep.score += 50;
        rep.events.push(ReputationEvent {
            timestamp: now.clone(),
            event: "badge".into(),
            delta: 50,
            description: "获得徽章: 传道者 (邀请10人)".into(),
        });
    }
}

fn rank_name(score: u64) -> &'static str {
    match score {
        0..=9 => "凡",
        10..=49 => "士",
        50..=199 => "道",
        200..=999 => "玄",
        _ => "圣",
    }
}
