<div align="center">

# 🦀 TaoStorage — 道可道，非常道

**个人数据仓库 · 分布式 P2P 存储网络 · 浏览器即节点**

[![CI](https://github.com/malaxiya20250530-glitch/taostorage/actions/workflows/ci.yml/badge.svg)](https://github.com/malaxiya20250530-glitch/taostorage/actions)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Node](https://img.shields.io/badge/Node.js-%3E%3D22-brightgreen?logo=node.js)](https://nodejs.org)
[![WASM](https://img.shields.io/badge/WASM-ready-purple?logo=webassembly)](https://webassembly.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)](#-contributing)

```text
╔══════════════════════════════════════════════════╗
║   道可道，非常道。                                ║
║   名可名，非常名。                                ║
║                                                  ║
║   你的数据，你的道。                              ║
║   Your Data, Your Tao.                           ║
╚══════════════════════════════════════════════════╝
```

### 🚀 **一行命令，加入全球 P2P 存储网络**

```bash
curl -fsSL https://tao.storage/install.sh | bash
```

### 🌐 **浏览器即节点，打开即用**

**[https://tao.storage](https://tao.storage)** — 无需安装，打开就是 P2P 节点

</div>

---

## 📋 目录

- [为什么是 TaoStorage？](#-为什么是-taostorage)
- [病毒式传播](#-病毒式传播)
- [架构总览](#-架构总览)
- [快速开始](#-快速开始)
- [CLI 命令](#-cli-命令)
- [浏览器节点](#-浏览器节点)
- [邀请奖励系统](#-邀请奖励系统)
- [WASM 开发](#-wasm-开发)
- [设计哲学](#-设计哲学)
- [路线图](#-路线图)
- [贡献指南](#-贡献指南)

---

## 🎯 为什么是 TaoStorage？

### 你的数据属于谁？

| 平台 | 数据归谁？ | 你能控制吗？ |
|:----:|:----------:|:-----------:|
| ☁️ Google Drive | Google | ❌ |
| ☁️ iCloud | Apple | ❌ |
| ☁️ 百度网盘 | 百度 | ❌ |
| 🦀 **TaoStorage** | **你自己** | **✅ 完全控制** |

**TaoStorage** 不是一个云服务，而是一个 **去中心化的 P2P 存储协议**。你的数据不存放在某个公司的服务器上，而是加密分片存储在**全球节点的手机、电脑、浏览器**里。

### 它用《道德经》的思想设计数据存储

```text
道生一 → 阴阳 → 气 → 万物
  │        │       │      │
 存储    数据+   守护    分布式
 引擎    元数据  进程    网络
```

这不是营销噱头——**六十四卦**映射数据生命周期，**阴阳气**三位一体建模数据单元，**Qi(气)引擎**做自愈决策。它真的把哲学变成了代码。

---

## 🦠 病毒式传播

### 方式 1: 一行命令安装

```bash
# 任何设备，一行命令加入网络
curl -fsSL https://tao.storage/install.sh | bash
```

自动检测 OS/架构，下载二进制或源码编译，启动守护进程，加入 P2P 网络。

### 方式 2: 浏览器即节点

**打开 [https://tao.storage](https://tao.storage) → 自动成为 P2P 节点**。

无需安装，无需注册，打开就是节点。每个访问者都是存储网络的一部分。

### 方式 3: 嵌入到任何地方

| 方式 | 一行集成 |
|:-----|:---------|
| **npm** | `npm install taostorage` |
| **Docker** | `docker run taostorage/node` |
| **GitHub Action** | 在 CI/CD 中自动部署节点 |
| **VS Code** | 插件市场搜索 TaoStorage |

### 方式 4: 邀请裂变

```bash
tao invite generate                         # 生成邀请码
tao invite use <CODE> <NEW_NODE>            # 使用邀请码
tao leaderboard                             # 查看排行榜
```

**每邀请一人，信誉 +10 🏆**。节点越多，网络越强。

---

## 🏗️ 架构总览

```
taostorage/
├── 📦 core/          ← 核心引擎（数据模型 + 存储 + 索引）
│   ├── unit.rs       ← 阴阳气：DataUnit 三位一体
│   ├── storage.rs    ← 坤：sled 嵌入式存储
│   ├── qi.rs         ← 气机决策引擎
│   ├── index.rs      ← 标签索引/搜索
│   ├── erasure.rs    ← 纠删码（Reed-Solomon）
│   └── metadata.rs   ← 名称索引
│
├── 🔐 crypto/        ← 加密模块
│   ├── encrypt.rs    ← 对称/非对称加密
│   ├── homomorphic.rs← 同态加密
│   └── zk_proof.rs   ← 零知识存储证明
│
├── 🌐 network/       ← P2P 网络（libp2p）
│   ├── swarm.rs      ← 网络事件/命令总线
│   ├── dht.rs        ← Kademlia DHT
│   ├── protocol.rs   ← 自定义协议
│   └── obfuscation.rs← 流量混淆
│
├── 🤝 consensus/     ← 共识机制
│   ├── proof.rs      ← 存储证明
│   ├── reputation.rs ← 信誉系统
│   └── balance.rs    ← 副本平衡
│
├── ⚡ wasm_host/     ← WASM 沙箱（智能合约）
│
├── 🖥️  cli/          ← 命令行 + 守护进程
│   ├── main.rs        ← CLI 入口（200+ 行命令处理）
│   └── daemon.rs      ← HTTP API + P2P 节点
│
├── 🌍 www/           ← 浏览器节点
│   ├── index.html     ← 主界面
│   ├── tao-browser/   ← Rust→WASM 浏览器核心
│   └── signaling-server/ ← WebRTC 信令服务器
│
├── 🧲 invite-system/ ← 邀请奖励系统
│
├── 📜 scripts/       ← 部署脚本
│   ├── install.sh    ← 一键安装
│   └── invite.sh     ← 邀请管理（纯 Shell）
│
└── 🏭 .github/       ← CI/CD 自动化
    └── workflows/
        ├── ci.yml      ← 测试 + 检查
        ├── release.yml ← 跨平台发布
        └── pages.yml   ← WASM 部署到 Pages
```

---

## 🚀 快速开始

### 安装

```bash
# 方式 1: 一行命令（推荐）
curl -fsSL https://tao.storage/install.sh | bash

# 方式 2: 从源码编译
git clone https://github.com/malaxiya20250530-glitch/taostorage.git
cd taostorage
cargo build --release --bin tao
./target/release/tao --help

# 方式 3: Docker
docker run -v $HOME/.taostorage:/root/.taostorage ghcr.io/malaxiya20250530-glitch/taostorage:latest
```

### 新手三部曲

```bash
# 1️⃣ 写入你的第一条"道"
tao put 道 "道可道，非常道" --tag welcome --tag genesis

# 2️⃣ 读取
tao get 道

# 3️⃣ 启动守护进程，加入网络
tao daemon start --network
```

### 浏览器节点（无需安装）

打开 **[https://tao.storage](https://tao.storage)** 或在本地运行：

```bash
# 使用 Rust CLI 启动
tao browser --port 3000 --ws-port 3001

# 或直接使用 Node.js
cd www/signaling-server && npm install && npm start
```

---

## 📟 CLI 命令

### 数据操作

```bash
tao put <key> <value> --tag <tag1> --tag <tag2>  # 写入/更新
tao get <key>                                      # 读取
tao list                                           # 列出所有
tao search <query>                                 # 模糊搜索
tao by-tag <tag>                                   # 标签查询
tao tag-cloud                                      # 标签云
tao delete <key>                                   # 删除
```

### 备份

```bash
tao export <path>                                  # 导出 JSON 备份
tao import <path>                                  # 导入恢复
```

### 守护进程

```bash
tao daemon start                                   # 前台启动
tao daemon start --background                      # 后台启动
tao daemon start --network                         # 启用 P2P 网络
tao daemon status                                  # 查看状态
tao daemon stop                                    # 停止
```

### 邀请系统

```bash
tao invite generate <node_id>                      # 生成邀请码
tao invite use <CODE> <new_node_id>                # 使用邀请码
tao invite leaderboard                             # 全球排行榜
tao reputation <node_id>                           # 查看信誉
# 或简写:
tao reputation                                     # 显示排行榜
```

### 浏览器节点

```bash
tao browser --port 3000                            # 启动浏览器节点服务器
```

---

## 🌍 浏览器节点

TaoStorage 可以在 **浏览器中运行**，每个访问者自动成为 P2P 网络的轻节点。

### 技术原理

```
┌─────────────────────────────────────────────────┐
│                浏览器节点                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ 数据核心  │  │ 存储引擎  │  │ 节点通信  │       │
│  │ (WASM)   │  │ (IndexedDB)│  │ (WebRTC) │       │
│  └──────────┘  └──────────┘  └──────────┘       │
│         │              │              │           │
│         ▼              ▼              ▼           │
│  ┌──────────────────────────────────────────┐    │
│  │        信令服务器 (WebSocket)             │    │
│  └──────────────────────────────────────────┘    │
│         │              │              │           │
│         ▼              ▼              ▼           │
│  Node A ◄────── WebRTC P2P ──────► Node B       │
└─────────────────────────────────────────────────┘
```

### 功能

| 功能 | 支持 |
|:----:|:----:|
| 📝 读写数据 | ✅ |
| 🔍 搜索 | ✅ |
| 🏷️ 标签 | ✅ |
| 🔗 P2P 通信 | ✅ (WebRTC) |
| 💾 导出备份 | ✅ |
| 📂 导入备份 | ✅ |
| 📤 邀请 | ✅ |
| 🏆 排行榜 | ✅ |
| 📱 PWA 离线可用 | ✅ |
| 🌙 深色模式 | ✅ |

---

## 🧲 邀请奖励系统

### 等级体系

| 等级 | 所需信誉 | 称号 | 特权 |
|:----:|:--------:|:----|:-----|
| 🥚 **凡** | 0-9 | 学徒 | 基础存储 |
| 🥷 **士** | 10-49 | 修士 | 标签查询 |
| 🧙 **道** | 50-199 | 得道者 | 高级搜索 |
| 🧝 **玄** | 200-999 | 玄妙境 | P2P 优先级 |
| 🦸 **圣** | 1000+ | 圣人 | 全部功能 + 命名权 |

### 徽章

| 徽章 | 条件 | 奖励 |
|:----:|:-----|:----:|
| 🏅 创始节点 | 前 1024 个节点 | 永久 VIP |
| 📢 传道者 | 邀请 10 人 | +50 信誉 |
| 🗿 宗师 | 邀请 100 人 | +50 信誉 |
| 💾 存储大师 | 存储 1GB | +50 信誉 |
| 🐢 长寿节点 | 在线 30 天 | +50 信誉 |

### 积分规则

| 行为 | 积分 |
|:----|:----:|
| ✅ 被邀请加入 | +5 |
| 🤝 邀请一人 | +10 |
| 🏅 获得徽章 | +50 |
| 💾 存储 1MB | +1 |
| ⏱️ 在线 1小时 | +1 |

---

## 🧘 设计哲学

### 阴阳气三位一体

```rust
DataUnit {
    yin:  Yin,    // 阴 — 数据本体（payload + 内容哈希）
    yang: Yang,   // 阳 — 元数据（名称、标签、热度）
    qi:   Qi,     // 气 — 状态（卦象、副本策略）
}
```

### 六十四卦 → 数据生命周期

```
🌱 屯 (Zhun)   ── 初生：数据刚写入
✅ 既济 (Jiji) ── 功成：达到标准保护
🔥 泰 (Tai)    ── 通泰：热数据，高频访问
❄️ 否 (Pi)     ── 闭塞：冷数据，已压缩
🚨 剥 (Bo)     ── 剥落：冗余不足，紧急修复
🏛️ 坤 (Kun)    ── 归藏：已归档
```

### Qi(气)决策引擎

每个 DataUnit 自带的"气"守护进程，根据局部信息做决策：

```rust
fn decide(qi: &Qi) -> QiAction {
    if qi.hexagram == Bo { return EmergencyRebuild; }
    if qi.replica_count < qi.target_replicas { return Replicate; }
    // ...
}
```

数据像生命体一样**自我修复、自我平衡、自我优化**。

---

## 🗺️ 路线图

```
v0.1 ─── v0.2 ─── v0.3 ─── v0.4 ─── v0.5 ─── ...
 │        │        │        │        │
JSON    SQLite   Rust +   CRDT +    DAO +
文件     标签    P2P 网络  离线优先  存储挖矿
        WASM       │
        加密        │
        共识      浏览器
                  节点 🚀
                  邀请
                  系统
```

| 版本 | 状态 | 亮点 |
|:----:|:----:|:------|
| v0.1 | ✅ | JSON 文件存储，原型验证 |
| v0.2 | ✅ | SQLite + 标签系统 |
| **v0.3** | **🚀 当前** | **Rust 重写 + P2P 网络 + WASM + 邀请系统** |
| v0.4 | 🔜 | CRDT 冲突解决 + 离线优先 |
| v0.5 | 🔮 | DAO 治理 + 存储挖矿代币 |

---

## 🤝 贡献指南

### 想加入？太棒了！

```bash
# Fork + Clone
git clone https://github.com/你的用户名/taostorage.git
cd taostorage

# 编译
cargo build

# 运行测试
cargo test

# 启动节点
cargo run -- daemon start
```

### 贡献方式

| 领域 | 适合谁 |
|:-----|:-------|
| 🦀 **Rust 核心** | Rustacean |
| 🌐 **P2P 网络** | libp2p 专家 |
| 🧮 **密码学** | 加密爱好者 |
| 🎨 **前端/WASM** | Web 开发者 |
| 📝 **文档/翻译** | 写作者 |
| 🐛 **测试/反馈** | 每个人 |

### 行为准则

- 尊重他人
- 拥抱哲学 (可以吐槽代码，但别吐槽"道")
- PR 合并即获得节点命名权

---

## 📄 许可

Apache-2.0 License — 完全开源，永远自由。

---

<div align="center">

### 🌟 给个 Star，加入"道"的网络

[![GitHub stars](https://img.shields.io/github/stars/malaxiya20250530-glitch/taostorage?style=social)](https://github.com/malaxiya20250530-glitch/taostorage)

**道可道，非常道。你的数据，你的道。**

```text
curl -fsSL https://tao.storage/install.sh | bash
```

**[https://tao.storage](https://tao.storage)** — 打开就是节点

</div>
