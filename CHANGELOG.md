# Changelog

## [0.3.0] — 2026-06-16 🚀

### 🎯 重大更新 — "万物归道"版本

这个版本将 TaoStorage 从单机工具升级为**全球 P2P 存储网络**。

#### 🌐 P2P 网络
- ✨ libp2p 全栈网络：Kademlia DHT + mDNS + 自定义协议
- 🔐 传输加密 + 流量混淆
- 📡 信令服务器：WebSocket 节点注册和 WebRTC 连接协商

#### 🌍 浏览器节点
- ✨ 全新 `tao browser` 命令 — 一键启动 HTTP + WebSocket 服务
- 🖥️ 完整的 PWA 浏览器界面：数据 CRUD、搜索、标签、备份
- 🧩 Rust→WASM 核心：阴阳气数据模型在浏览器中运行
- 🔗 WebRTC P2P 通信：浏览器之间直连
- 📱 PWA 离线可用 + 深色主题

#### 🧲 邀请奖励系统
- ✨ `tao invite generate` / `tao invite use` / `tao reputation`
- 🏆 5 级等级体系：凡→士→道→玄→圣
- 🏅 徽章系统：传道者、宗师、存储大师、长寿节点
- 📊 全球排行榜
- 📜 Shell 独立版脚本（零依赖）

#### 🚀 部署和传播
- ✨ 一键安装脚本：`curl -fsSL https://tao.storage/install.sh | bash`
- 🐳 Docker + Docker Compose 支持
- 🏭 GitHub Actions CI/CD（测试 + 跨平台构建 + WASM 部署）
- 📢 社交媒体推广文案生成器

#### 🦀 Rust 核心
- ✨ WASM 沙箱宿主（wasmi 运行时）
- ✨ 零知识存储证明（ZK Proof）
- ✨ 同态加密引擎
- ✨ BLS 签名预留
- ⚡ 性能优化：sled 嵌入式存储

#### 📝 文档
- ✨ README 完全重写（病毒式传播导向）
- ✨ SECURITY 安全策略文档
- ✨ GitHub Issue 模板
- ✨ CHANGELOG 添加

## [0.2.0] — 2026-06-12

### 新增
- SQLite 存储引擎替代 JSON 文件
- 标签系统和按标签查询
- 完整的中文文档
- 设计哲学文档
- 贡献指南

### 改进
- 目录结构调整
- 更好的错误处理

## [0.1.0] — 2026-06-10

### 初始发布
- JSON 文件存储引擎
- 基础 CRUD 操作
- 模糊搜索
- 导出/导入备份
