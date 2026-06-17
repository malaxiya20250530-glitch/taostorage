#!/usr/bin/env bash
# ============================================================
# 🎬 TaoStorage 完整演示启动器
# 一键启动: CLI + 信令服务器 + 浏览器节点
# ============================================================
# 用法: bash demo.sh
# ============================================================

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'
CYAN='\033[0;36m'; YELLOW='\033[1;33m'; BOLD='\033[1m'; NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAO_BIN="${PROJECT_DIR}/target/debug/tao"
SIG_DIR="${PROJECT_DIR}/www/signaling-server"

cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 停止服务...${NC}"
    kill $TAO_PID 2>/dev/null || true
    kill $SIG_PID 2>/dev/null || true
    wait 2>/dev/null || true
    echo -e "${GREEN}✅ 已停止${NC}"
    exit 0
}
trap cleanup INT TERM

# 检查二进制
if [ ! -f "$TAO_BIN" ]; then
    echo -e "${RED}❌ 请先编译: cargo build --bin tao${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  🎬 TaoStorage 完整演示                      ║${NC}"
echo -e "${CYAN}║  道可道，非常道                               ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${BOLD}Version:${NC} $($TAO_BIN --version 2>/dev/null)"
echo ""

# 1. 写入创始数据
echo -e "${BLUE}📝 初始化数据...${NC}"
$TAO_BIN put 道 "道可道，非常道" --tag welcome --tag genesis --tag tao 2>/dev/null
$TAO_BIN put 阴阳 "一阴一阳之谓道" --tag philosophy --tag tao 2>/dev/null
$TAO_BIN put 万物 "道生一，一生二，二生三，三生万物" --tag philosophy --tag tao 2>/dev/null
$TAO_BIN put 欢迎 "欢迎来到 TaoStorage 网络" --tag welcome 2>/dev/null

# 2. 启动信令服务器
echo -e "${BLUE}📡 启动信令服务器...${NC}"
cd "${SIG_DIR}"
npm install --silent 2>/dev/null
node server.js &
SIG_PID=$!
cd "${PROJECT_DIR}"
sleep 1

echo ""
echo -e "${GREEN}${BOLD}✅ TaoStorage 已就绪！${NC}"
echo ""
echo -e "  📊  ${BOLD}数据统计:${NC}"
$TAO_BIN stats 2>/dev/null | grep -E "总|唯|热"
echo ""
echo -e "  🏷️   ${BOLD}标签云:${NC}"
$TAO_BIN tag-cloud 2>/dev/null
echo ""
echo -e "  🌐  ${BOLD}浏览器节点:${NC} http://localhost:3000"
echo -e "  🔌  ${BOLD}信令:${NC}         ws://localhost:3001"
echo -e "  🔗  ${BOLD}邀请:${NC}         $($TAO_BIN invite generate demo 2>/dev/null | grep -oP 'https://[^ ]+' || bash scripts/invite.sh generate demo 2>/dev/null | grep -oP 'https://[^ ]+')"
echo ""
echo -e "${YELLOW}  按 Ctrl+C 停止所有服务${NC}"

# 保持运行
wait
