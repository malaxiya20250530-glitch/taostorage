<div align="center">

# 🦀 TaoStorage

**个人数据仓库 CLI / Personal Data Warehouse CLI**

[![Node](https://img.shields.io/badge/Node.js-≥22-339933?logo=node.js)](https://nodejs.org)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)](#)

> **道可道，非常道** — 一个极简的个人数据仓库，纯 CLI 操作，零外部依赖。  
> **The Tao that can be told is not the eternal Tao** — A minimalist personal data warehouse CLI, zero external dependencies.

</div>

---

## 📋 目录 / Table of Contents

- [设计哲学 / Philosophy](#-设计-philosophy)
- [技术架构 / Architecture](#-技术架构-architecture)
- [快速开始 / Quick Start](#-快速开始-quick-start)
- [命令参考 / Command Reference](#-命令参考-command-reference)
- [数据模型 / Data Model](#-数据模型-data-model)
- [技术栈 / Tech Stack](#-技术栈-tech-stack)
- [路线图 / Roadmap](#-路线图-roadmap)
- [许可 / License](#-许可-license)

---

## 🧘 设计哲学 / Philosophy

**TaoStorage 不是一个数据库，而是一个"可进化的数据结构种子"。**  
**TaoStorage is not a database — it's an "evolvable data structure seed."**

取名自《道德经》的 **"道"**，核心思想是：  
Named after the **"Tao"** from Tao Te Ching, the core idea is:

```
道生一 → 阴阳 → 气 → 万物
  ↓
key/value 存储
  ↓
标签、索引、查询
  ↓
P2P、CRDT、分布式
```

| 概念 | 解释 |
|------|------|
| **道 (Tao)** | 数据存储的本质规律 — 存、取、查 |
| **阴阳 (Yin-Yang)** | 数据本体 (value) + 元数据 (key/tags) |
| **气 (Qi)** | 数据生命周期、决策、自愈 |
| **万物 (Everything)** | 分布式、P2P、CRDT 等高级特性 |

**v0.1 & v0.2 只做一件事：先能存、能取、能查。**  
所有复杂分布式特性留到后续版本。

---

## 🏗 技术架构 / Architecture

### 系统架构图 / System Architecture

```
┌─────────────────────────────────────────────────┐
│                   bin/tao.js                      │
│               CLI 入口 / Entry Point               │
│              (14 commands)                        │
├─────────────┬───────────┬───────────┬────────────┤
│   store.js   │ indexer.js │ export.js │ import.js  │
│   CRUD 引擎  │  查询引擎   │  导出引擎  │  导入引擎   │
│   CRUD       │  Query     │  Export   │  Import    │
├─────────────┴───────────┴───────────┴────────────┤
│                   db.js                           │
│              SQLite Connection                    │
│         node:sqlite (Node.js built-in)            │
├──────────────────────────────────────────────────┤
│                   tao.db                          │
│              WAL Mode                             │
│    items table + tags table + 3 indexes           │
└──────────────────────────────────────────────────┘
```

### 模块职责 / Module Responsibilities

| 模块 Module | 文件 File | 职责 Responsibility |
|-------------|-----------|-------------------|
| **数据库层 Database** | `core/db.js` | SQLite 连接管理、Schema 初始化、WAL 模式 / Connection management, schema init |
| **存储引擎 Storage** | `core/store.js` | CRUD、事务、批量导入、统计 / CRUD, transactions, bulk import |
| **查询引擎 Query** | `core/indexer.js` | 模糊搜索、标签组合、时间范围、标签云 / Search, tag queries, tag cloud |
| **导出引擎 Export** | `core/export.js` | JSON 备份导出 / JSON backup export |
| **导入引擎 Import** | `core/import.js` | 事务覆盖导入、格式校验 / Transactional import, validation |
| **CLI 入口 CLI** | `bin/tao.js` | 14 个命令分发、参数解析、格式化输出 / Command dispatch, arg parsing |

### 数据流 / Data Flow

**写入 / Write:**
```
tao put note "hello" --tag demo
  ↓
store.put(key, value, tags)
  ↓
BEGIN TRANSACTION
  ├── INSERT INTO items (id, key, value, ...)
  └── INSERT INTO tags (item_id, tag)
COMMIT
  ↓
✅ 已存储 / Stored [note] → abc12345... [demo]
```

**读取 / Read:**
```
tao get note
  ↓
store.get(key)
  ↓
SELECT * FROM items WHERE key = ? ORDER BY created_at DESC
  ↓
SELECT tag FROM tags WHERE item_id = ?
  ↓
格式化输出 / Formatted output
```

---

## 🚀 快速开始 / Quick Start

### 前提条件 / Prerequisites

- **Node.js ≥ 22**（使用内置 `node:sqlite`，零依赖安装 / uses built-in `node:sqlite`, zero install)
- **Termux / macOS / Linux / Windows** 均可

```bash
# 1. 克隆 / Clone
git clone https://github.com/malaxiya20250530-glitch/taostorage.git
cd taostorage

# 2. 直接使用，零依赖！/ Ready to go, zero deps!
node bin/tao.js init

# 3. 写入数据 / Write data
node bin/tao.js put note "道可道，非常道" --tag philosophy,chinese
node bin/tao.js put note "The Tao that can be told" --tag philosophy,english
node bin/tao.js put todo "Buy milk" --tag shopping,daily
node bin/tao.js put code "console.log('hello tao')" --tag javascript,example

# 4. 读取 / Read
node bin/tao.js get note

# 5. 浏览 / List all
node bin/tao.js list

# 6. 搜索 / Search
node bin/tao.js search Tao

# 7. 标签云 / Tag cloud
node bin/tao.js tags

# 8. 备份 / Export
node bin/tao.js export backup.json

# 9. 恢复 / Import
node bin/tao.js import backup.json

# 10. 统计 / Stats
node bin/tao.js stats
```

### 安装到 PATH（可选）/ Install to PATH (optional)

```bash
npm link
# 或 / or
alias tao='node /path/to/taostorage/bin/tao.js'

# 之后可以直接用 / Then use directly:
tao init
tao put note "hello" --tag demo
```

---

## 📖 命令参考 / Command Reference

### `tao init`
> **初始化 SQLite 数据库 / Initialize SQLite database**  
> 创建 `items` 和 `tags` 表及索引 / Creates tables and indexes

### `tao put <key> <value> [--tag a,b]`
> **写入数据 / Write data**  
> `key` — 分类/索引键 / classification key  
> `value` — 实际内容 / actual content  
> `--tag` — 逗号分隔的标签 / comma-separated tags

### `tao get <key>`
> **按 key 查询 / Query by key**  
> 返回所有匹配项，按时间倒序 / Returns all matches, newest first

### `tao list [options]`
> **浏览数据 / Browse data**  
> 支持多种过滤和排序 / Supports filtering & sorting:

| 选项 Option | 类型 Type | 说明 Description |
|-------------|-----------|------------------|
| `--key` | string | 按 key 过滤 / Filter by key |
| `--tag` | string | 按标签过滤 / Filter by tag |
| `--limit` | number | 限制条数 / Limit results |
| `--sort` | string | 排序字段 (默认 `created_at`) / Sort field |
| `--order` | ASC/DESC | 排序方向 / Sort direction |

### `tao delete <key>`
> **删除数据 / Delete data**  
> 关联标签自动级联删除 / Tags auto-deleted via CASCADE

### `tao search <query>`
> **全字段模糊搜索 / Full-text fuzzy search**  
> 匹配 key / value / tag 三个字段 / Matches key, value, and tag fields

### `tao tag <tagname>`
> **按标签查询 / Query by tag**

### `tao tags`
> **标签云 / Tag cloud**  
> 显示所有标签及其使用次数（含 ASCII 条图）/ Shows all tags with counts and ASCII bar chart

### `tao export [path]`
> **导出备份 / Export backup**  
> 导出为 JSON（含数据、标签、统计）/ Exports to JSON with data, tags, stats

### `tao import <path>`
> **导入恢复 / Import backup**  
> 事务覆盖模式，失败自动回滚 / Transactional overwrite, auto-rollback on failure

### `tao stats`
> **统计信息 / Statistics**  
> 总条数、唯一 Key 数、标签数、热门排行 / Total items, unique keys, tags, top rankings

### `tao help`
> **显示帮助 / Show help**

---

## 📐 数据模型 / Data Model

### SQLite Schema

```sql
-- 数据表 / Data table
CREATE TABLE items (
  id         TEXT PRIMARY KEY,      -- UUID
  key        TEXT NOT NULL,         -- 分类键 / classification key
  value      TEXT NOT NULL,         -- 数据内容 / data content
  created_at INTEGER NOT NULL,      -- 创建时间 / created timestamp
  updated_at INTEGER NOT NULL       -- 更新时间 / updated timestamp
);

-- 标签表（一对多）/ Tags table (one-to-many)
CREATE TABLE tags (
  item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  tag     TEXT NOT NULL,
  PRIMARY KEY (item_id, tag)
);

-- 索引 / Indexes
CREATE INDEX idx_items_key ON items(key);
CREATE INDEX idx_tags_tag ON tags(tag);
CREATE INDEX idx_items_created ON items(created_at);
```

### 数据示例 / Example Data

```json
{
  "id": "467b1952-db46-4b95-b63f-e1e7fae65e7f",
  "key": "note",
  "value": "道可道，非常道",
  "tags": ["chinese", "philosophy"],
  "created_at": 1781245122570,
  "updated_at": 1781245122570
}
```

### 备份文件格式 / Backup Format

```json
{
  "version": "0.2.0",
  "exportedAt": 1781245136214,
  "stats": {
    "totalItems": 7,
    "uniqueKeys": 5,
    "totalTags": 11
  },
  "data": {
    "items": [ ... ],
    "tags": [ { "item_id": "...", "tag": "..." }, ... ]
  }
}
```

---

## 🛠 技术栈 / Tech Stack

| 技术 Technology | 说明 Description |
|----------------|------------------|
| **Node.js** | ≥ 22, 运行时 / Runtime |
| **node:sqlite** | Node.js 内置 SQLite（零外部依赖 / zero external deps） |
| **WAL 模式** | Write-Ahead Logging，读写并发友好 / concurrent read/write friendly |
| **ESM 模块** | 原生 ES Module，`import/export` |
| **GitHub** | 源码托管 / Source hosting |

### 为什么不用 / Why not

| 技术 | 原因 Reason |
|------|-------------|
| better-sqlite3 | 需要 native 编译，Termux 上容易出问题 / requires native compilation |
| SQL.js | WASM 版本，Node 内置 SQLite 更轻量 / Node built-in is lighter |
| JSON 文件 | v0.1 的存储方式，v0.2 已升级到 SQLite / v0.1 approach, v0.2 upgraded |
| PostgreSQL / MySQL | 太重，单机场景不需要 C/S 架构 / too heavy for single-machine CLI |

---

## 📊 对比 / Comparison: v0.1 → v0.2

| 特性 Feature | v0.1 (JSON) | v0.2 (SQLite) |
|-------------|:-----------:|:-------------:|
| 存储引擎 / Storage | JSON 文件 | SQLite (WAL) |
| 外部依赖 / Dependencies | **零 Zero** | **零 Zero** |
| 事务 / Transactions | ❌ | ✅ BEGIN/COMMIT/ROLLBACK |
| 标签系统 / Tags | 简单数组 | 独立表 + 索引 / Separate table |
| 标签云 / Tag Cloud | ❌ | ✅ ASCII 条图 / Bar chart |
| 排序 / Sorting | ❌ | ✅ `--sort key --order ASC` |
| 分页 / Pagination | ❌ | ✅ `--limit N` |
| 模糊搜索 / Search | 内存遍历 / In-memory | SQL LIKE 扫描 |
| 标签 AND 查询 / Tag AND | ❌ | ✅ `byTagsAnd()` |
| 外键约束 / Foreign Keys | ❌ | ✅ CASCADE |
| 数据文件 / Data file | db.json + index.json | 单一 `.db` (~32KB) |

---

## 🗺 路线图 / Roadmap

```
v0.1 ──→ v0.2 ──→ v0.3 ──→ v0.4 ──→ ...
JSON     SQLite    P2P sync   CRDT
                  Kademlia   Offline-first
```

| 版本 | 特性 | 状态 |
|------|------|------|
| **v0.1** | JSON 存储 + 基本 CRUD + 导出导入 | ✅ 完成 |
| **v0.2** | SQLite + 标签系统 + 模糊搜索 + 排序分页 | ✅ **当前版本** |
| **v0.3** | P2P 同步 (Kademlia DHT + libp2p) | 🔜 规划中 |
| **v0.4** | CRDT 冲突解决 + 端到端加密 + 离线优先 | 🔮 远期 |

### v0.3 规划 / v0.3 Plans

- 🌐 **P2P 同步 / P2P Sync** — 基于 Kademlia DHT 的多节点数据同步
- 🔗 **libp2p 网络 / libp2p Networking** — 节点自动发现、数据分片传输
- 📍 **内容寻址 / Content Addressing** — 数据通过哈希寻址，去重存储

### v0.4 规划 / v0.4 Plans

- 🔄 **CRDT 冲突解决 / CRDT Conflict Resolution** — 离线编辑自动合并
- 🔐 **端到端加密 / E2E Encryption** — 存储节点零知识
- 📱 **移动端适配 / Mobile Support** — Termux 后台服务

---

## 📄 许可 / License

Apache-2.0

---

<div align="center">

**TaoStorage** — *道可道，非常道 / The Tao that can be told is not the eternal Tao*

<sub>《道德经》第一章 / Tao Te Ching, Chapter 1</sub>

</div>
