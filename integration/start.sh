#!/usr/bin/env bash
# ============================================================
# 🚀 TaoStorage 全栈启动 — 存储 + 检测 + 仪表盘
# ============================================================

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cleanup() { echo -e "\n${RED}🛑 停止所有服务${NC}"; kill $TAO_PID $GATEWAY_PID $SIG_PID 2>/dev/null; exit 0; }
trap cleanup INT TERM

echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  🚀 TaoStorage 全栈启动                      ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# 1. 写入样例数据
echo -e "${BLUE}1️⃣  初始化数据...${NC}"
TAO_BIN="${PROJECT_DIR}/target/debug/tao"
if [ -f "$TAO_BIN" ]; then
    $TAO_BIN put 道 "道可道，非常道" --tag welcome --tag genesis --tag tao 2>/dev/null
    $TAO_BIN put 阴阳 "一阴一阳之谓道" --tag philosophy --tag tao 2>/dev/null
    
    # 写入审计样例
    $TAO_BIN put "audit:demo:$(date -u +%Y%m%dT%H%M%S)" '{
      "query":"上海人口有多少？",
      "hallucination_score":0.35,
      "verdicts":[{"claim":"上海有5000万人口","verdict":"contradicted","confidence":0.92}],
      "warnings":["数值严重偏离事实"]
    }' --tag audit --tag hallucination --tag critical 2>/dev/null
    
    echo -e "   ${GREEN}✅ 数据已就绪${NC}"
fi

# 2. 启动信令服务器
echo -e "${BLUE}2️⃣  启动信号服务器 (端口 3000)...${NC}"
cd "${PROJECT_DIR}/www/signaling-server"
npm install --silent 2>/dev/null
node server.js &
SIG_PID=$!
cd "${PROJECT_DIR}"
sleep 1
echo -e "   ${GREEN}✅ 浏览器节点: http://localhost:3000${NC}"

# 3. 启动幻觉检测网关
echo -e "${BLUE}3️⃣  启动检测网关 (端口 8800)...${NC}"
GATEWAY="${HOME}/hallucination_detector/awareness_gateway.py"
if [ -f "$GATEWAY" ]; then
    python3 "$GATEWAY" --port 8800 --mock &
    GATEWAY_PID=$!
    sleep 2
    echo -e "   ${GREEN}✅ 检测网关: http://localhost:8800${NC}"
else
    echo -e "   ${YELLOW}⚠️ 检测网关未安装${NC}"
fi

echo ""
echo -e "${CYAN}══════════════════════════════════════════════${NC}"
echo -e "  ${BOLD}所有服务已启动${NC}"
echo ""
echo -e "  🌐 浏览器节点:   http://localhost:3000"
echo -e "  🔍 检测网关:     http://localhost:8800/health"
echo -e "  📊 审计仪表盘:   http://localhost:3000/audit.html"
echo -e "  🦀 CLI:          cd ~/taostorage && ./target/debug/tao"
echo ""
echo -e "  ${BOLD}快捷命令:${NC}"
echo -e "  tao by-tag hallucination   查看检测记录"
echo -e "  tao by-tag critical       查看严重告警"
echo -e "  tao search audit:         搜索审计日志"
echo -e "  tao stats                 存储统计"
echo ""
echo -e "${YELLOW}  按 Ctrl+C 停止所有服务${NC}"
echo -e "${CYAN}══════════════════════════════════════════════${NC}"

wait
