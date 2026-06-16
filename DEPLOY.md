# 🚀 TaoStorage 部署上线指南

> 一键将 TaoStorage 发布到全球

## 📋 前提条件

你需要在 **Linux / macOS** 机器上操作（或 Termux 已安装 Rust 的环境）。

## 第一步：发布到 GitHub

### 1.1 创建 GitHub 仓库

- 打开 https://github.com/new
- 仓库名: `taostorage`
- 公开仓库
- **不要勾选** Initialize with README（我们已有）

### 1.2 推送代码

```bash
# 在本地机器上
cd taostorage

# 初始化 git
git init
git add .
git commit -m "🎉 TaoStorage v0.3.0 — 道可道，非常道"

# 关联远程仓库
git remote add origin https://github.com/你的用户名/taostorage.git

# 推送
git push -u origin main
```

### 1.3 创建 Release

```bash
# 打标签
git tag v0.3.0
git push origin v0.3.0
```

GitHub Actions 会自动：
- ✅ 运行测试
- ✅ 构建 6 个平台的二进制文件
- ✅ 上传到 Release 页面

---

## 第二步：部署浏览器节点到 GitHub Pages

### 2.1 构建 WASM

```bash
# 安装 wasm-pack (https://rustwasm.github.io/wasm-pack/)
curl -fsSL https://rustwasm.github.io/wasm-pack/installer/init.sh | sh

# 构建 WASM
cd www
wasm-pack build tao-browser --target web --out-dir ../pkg
```

### 2.2 启用 Pages

1. 仓库 → Settings → Pages
2. Source: **GitHub Actions**
3. Actions 中的 `pages.yml` 会自动部署

部署后访问：`https://你的用户名.github.io/taostorage/`

---

## 第三步：注册域名

### 3.1 购买域名

推荐注册 `tao.storage`（约 ¥50/年）：
- 阿里云 / 腾讯云 / Namecheap / Cloudflare

### 3.2 配置 DNS

在域名管理中添加 CNAME 记录：

```
tao.storage → 你的用户名.github.io
www.tao.storage → 你的用户名.github.io
```

### 3.3 配置 Pages 自定义域名

1. GitHub 仓库 → Settings → Pages
2. Custom domain: `tao.storage`
3. 勾选 Enforce HTTPS

---

## 第四步：生成邀请码

```bash
# 构建 tao CLI
cd taostorage
cargo build --release --bin tao
./target/release/tao --version

# 生成创始节点邀请码
./target/release/tao invite generate genesis-node-1
./target/release/tao invite generate genesis-node-2

# 查看排行榜
./target/release/tao reputation
```

---

## 第五步：发布到各大平台

### 5.1 🐦 Twitter / X

```bash
# 生成推文
bash scripts/social-poster.sh twitter
```

复制输出内容发布。

### 5.2 🤖 Hacker News

发帖格式：
```
Title: Show HN: TaoStorage – A P2P storage network inspired by Tao Te Ching

《The Tao that can be told is not the eternal Tao》

I built a decentralized P2P storage network in Rust, inspired by Tao Te Ching.

- Yin-Yang data model with self-healing lifecycle
- libp2p + WASM + WebRTC + Zero-Knowledge Proofs
- One-line install: curl -fsSL https://tao.storage/install.sh | bash
- Browser as node: open the website and you're connected

GitHub: https://github.com/你的用户名/taostorage
Live: https://tao.storage
```

### 5.3 🎮 Reddit r/rust

```bash
bash scripts/social-poster.sh reddit
```

### 5.4 📺 中文社区 (B站/知乎)

```bash
bash scripts/social-poster.sh zhihu
bash scripts/social-poster.sh bilibili
```

---

## 第六步：Docker 部署

```bash
# 单节点
docker build -t taostorage .
docker run -d -p 8788:8788 -p 3000:3000 taostorage

# 多节点集群
docker compose up -d
docker compose logs -f
```

---

## 第七步：提交到包管理器

### npm

```bash
# 发布信令服务器到 npm
cd www/signaling-server
npm publish
```

### Termux 社区

在 Termux 的 `pkg` 源中提交 recipe。

---

## 📊 上线后监控指标

| 指标 | 目标 | 检查方式 |
|:-----|:-----|:---------|
| GitHub Stars | 100 🚀 | 仓库主页 |
| 活跃节点数 | 50+ | `tao reputation` |
| Docker Pulls | 100+ | Docker Hub |
| Pages 访问量 | 1000+/天 | Cloudflare Analytics |

## 🆘 需要帮助？

- 在 GitHub 提交 Issue
- 发起 PR 贡献代码
- 在项目 Discussions 中提问
