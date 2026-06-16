#!/usr/bin/env bash
# ============================================================
# 🚀 TaoStorage — 一键发布到 GitHub
# ============================================================
# 用法: bash scripts/publish.sh
# 前提: 已安装 git + 已创建 GitHub 仓库
# ============================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 仓库配置（修改为你的 GitHub 用户名）
GITHUB_USER="${GITHUB_USER:-malaxiya20250530-glitch}"
REPO_NAME="taostorage"
REMOTE_URL="https://github.com/${GITHUB_USER}/${REPO_NAME}.git"

echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  🚀 TaoStorage 一键发布                      ║${NC}"
echo -e "${CYAN}║  发布到: ${REMOTE_URL}  ${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# 获取项目根目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_DIR}"

# ---- 检查 git ----
if ! command -v git &>/dev/null; then
    echo -e "${RED}❌ 需要 git，请先安装${NC}"
    exit 1
fi

# ---- 检查是否已有 .git ----
if [ -d ".git" ]; then
    echo -e "${YELLOW}⚠️  仓库已初始化，跳过 git init${NC}"
else
    echo -e "${BLUE}📦 初始化 git 仓库...${NC}"
    git init
    git checkout -b main
fi

# ---- 检查远程仓库 ----
if git remote -v | grep -q origin; then
    echo -e "${BLUE}🔗 远程仓库已配置${NC}"
else
    echo -e "${BLUE}🔗 添加远程仓库: ${REMOTE_URL}${NC}"
    git remote add origin "${REMOTE_URL}"
fi

# ---- 构建 WASM（如果 wasm-pack 可用） ----
if command -v wasm-pack &>/dev/null; then
    echo -e "${BLUE}🌐 构建 WASM 浏览器节点...${NC}"
    cd www
    wasm-pack build tao-browser --target web --out-dir ../pkg 2>/dev/null || \
        echo -e "${YELLOW}⚠️  WASM 构建跳过（可在 CI 中自动构建）${NC}"
    cd "${PROJECT_DIR}"
fi

# ---- 创建 .gitignore（如果缺失） ----
if [ ! -f ".gitignore" ]; then
    cat > .gitignore << 'GI'
target/
www/pkg/
node_modules/
.vscode/
.idea/
*.swp
.DS_Store
Thumbs.db
*.db
*.log
tao-backup*.json
GI
fi

# ---- 提交 ----
echo -e "${BLUE}📝 创建提交...${NC}"
git add -A
git status

echo ""
echo -e "${CYAN}══════════════════════════════════════════${NC}"
echo -e "${CYAN}  即将推送以下文件${NC}"
echo -e "${CYAN}══════════════════════════════════════════${NC}"
echo ""

read -p "是否提交并推送? (y/N): " confirm
if [ "${confirm}" = "y" ] || [ "${confirm}" = "Y" ]; then
    git commit -m "🎉 TaoStorage v0.3.0 — 道可道，非常道

- P2P 分布式存储网络 (libp2p)
- 浏览器节点 (WASM + WebRTC)
- 邀请奖励系统
- 一键安装脚本
- CI/CD 全自动化"

    echo -e "${BLUE}🚀 推送到 GitHub...${NC}"
    git push -u origin main

    echo ""
    echo -e "${GREEN}✅ 发布成功！${NC}"
    echo ""
    echo -e "   仓库: ${CYAN}https://github.com/${GITHUB_USER}/${REPO_NAME}${NC}"
    echo -e "   Pages: ${CYAN}https://${GITHUB_USER}.github.io/${REPO_NAME}${NC}"
    echo ""
    echo -e "${YELLOW}📢 下一步:${NC}"
    echo -e "   1. 创建 Release: git tag v0.3.0 && git push origin v0.3.0"
    echo -e "   2. 注册域名: tao.storage → GitHub Pages"
    echo -e "   3. 发布到社交媒体: bash scripts/social-poster.sh"
    echo -e "   4. 看 CI 运行: https://github.com/${GITHUB_USER}/${REPO_NAME}/actions"
else
    echo -e "${YELLOW}⏸️  已取消推送。${NC}"
    echo "   手动提交: git commit -m '消息' && git push"
fi
