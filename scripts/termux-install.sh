#!/usr/bin/env bash
# ============================================================
# 📱 TaoStorage Termux 一键安装
# 专为 Android Termux 环境优化
# ============================================================
# 用法: bash termux-install.sh
# ============================================================

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  📱 TaoStorage Termux 一键安装             ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
echo ""

TAO_BIN_DIR="${HOME}/.tao/bin"

# 1. 安装依赖
echo -e "${BLUE}📦 安装依赖...${NC}"
pkg update -y 2>/dev/null
pkg install -y nodejs jq curl 2>&1 | tail -3

# 2. 复制二进制
echo -e "${BLUE}🦀 安装 tao CLI...${NC}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [ -f "${PROJECT_DIR}/target/debug/tao" ]; then
    mkdir -p "${TAO_BIN_DIR}"
    cp "${PROJECT_DIR}/target/debug/tao" "${TAO_BIN_DIR}/tao"
    chmod +x "${TAO_BIN_DIR}/tao"
    echo -e "${GREEN}✅ tao CLI 已安装到 ${TAO_BIN_DIR}${NC}"
else
    echo -e "${RED}❌ 找不到编译好的 tao 二进制${NC}"
    echo "   请先运行: cargo build --bin tao"
    exit 1
fi

# 3. 安装信令服务器
echo -e "${BLUE}📡 安装信令服务器...${NC}"
cd "${PROJECT_DIR}/www/signaling-server"
npm install --silent 2>/dev/null
echo -e "${GREEN}✅ 信令服务器已就绪${NC}"

# 4. 配置 PATH
SHELL_RC="${HOME}/.bashrc"
if ! grep -q "TAO_HOME" "${SHELL_RC}" 2>/dev/null; then
    echo "" >> "${SHELL_RC}"
    echo "# TaoStorage" >> "${SHELL_RC}"
    echo "export PATH=\"\${PATH}:${TAO_BIN_DIR}\"" >> "${SHELL_RC}"
    echo "export TAO_HOME=\"\${HOME}/.taostorage\"" >> "${SHELL_RC}"
fi

export PATH="${PATH}:${TAO_BIN_DIR}"
export TAO_HOME="${HOME}/.taostorage"

# 5. 初始化数据
echo -e "${BLUE}📝 写入创始数据...${NC}"
tao put 道 "道可道，非常道" --tag welcome --tag genesis 2>/dev/null || true
tao put 阴阳 "一阴一阳之谓道" --tag philosophy 2>/dev/null || true
tao put 万物 "道生一，一生二，二生三，三生万物" --tag philosophy 2>/dev/null || true

# 6. 生成邀请码
INVITE_CODE=$(tao invite generate termux-node 2>/dev/null | grep -oP '[A-Z0-9]{8}' | head -1 || bash "${PROJECT_DIR}/scripts/invite.sh" generate termux-node 2>/dev/null | grep -oP '[A-Z0-9]{8}' | head -1 || echo "")

# 7. 完成
echo ""
echo -e "${GREEN}${BOLD}🎉 TaoStorage 已就绪！${NC}"
echo ""
echo -e "  ${BOLD}快速开始:${NC}"
echo -e "  ${GREEN}tao stats${NC}              查看存储统计"
echo -e "  ${GREEN}tao list${NC}               列出所有数据"
echo -e "  ${GREEN}tao search 道${NC}          搜索数据"
echo -e "  ${GREEN}tao tag-cloud${NC}          查看标签云"
echo ""
echo -e "  ${BOLD}🌐 启动浏览器节点:${NC}"
echo -e "  ${GREEN}cd www/signaling-server && npm start${NC}"
echo ""
if [ -n "${INVITE_CODE}" ]; then
    echo -e "  ${BOLD}🔗 邀请链接:${NC} https://tao.storage/?invite=${INVITE_CODE}"
fi
echo ""
echo -e "${CYAN}道可道，非常道 — 你的数据，你的道。${NC}"
