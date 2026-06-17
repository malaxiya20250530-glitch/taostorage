#!/usr/bin/env bash
# ============================================================
# 🔗 TaoStorage × 幻觉检测 — 端到端集成演示
# ============================================================

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAO_BIN="${PROJECT_DIR}/target/debug/tao"

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  🔗 TaoStorage × 幻觉检测 集成演示           ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# 1. 测试 TaoStorage 可用
echo -e "${BLUE}1️⃣  检测 TaoStorage...${NC}"
if ! command -v "${TAO_BIN}" &>/dev/null; then
    echo -e "${RED}❌ tao CLI 不可用${NC}"
    exit 1
fi
echo -e "   ${GREEN}✅ tao $(${TAO_BIN} --version)${NC}"
echo ""

# 2. 写入一条模拟检测结果
echo -e "${BLUE}2️⃣  存储模拟检测结果到 TaoStorage...${NC}"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

${TAO_BIN} put "audit:${TIMESTAMP}" '{
  "query": "上海有多少人口？",
  "response_length": 45,
  "hallucination_score": 0.35,
  "hallucination_ratio": 0.65,
  "claims_count": 1,
  "verdicts": [
    {"claim": "上海有5000万人口", "verdict": "contradicted", "confidence": 0.92}
  ],
  "warnings": ["数值严重偏离事实: 上海实际人口约2500万"]
}' --tag audit --tag hallucination --tag critical 2>/dev/null

${TAO_BIN} put "audit:2026-06-17T03:25:00Z" '{
  "query": "Python 创始人是谁？",
  "hallucination_score": 0.95,
  "verdicts": [{"claim": "Guido van Rossum", "verdict": "verified", "confidence": 0.98}]
}' --tag audit --tag hallucination --tag clean 2>/dev/null

${TAO_BIN} put "audit:2026-06-17T03:20:00Z" '{
  "query": "月球距离地球多远？",
  "hallucination_score": 0.88,
  "verdicts": [{"claim": "约38.4万公里", "verdict": "verified", "confidence": 0.95}]
}' --tag audit --tag hallucination --tag clean 2>/dev/null

echo -e "   ${GREEN}✅ 已写入 3 条审计记录${NC}"
echo ""

# 3. 查询审计记录
echo -e "${BLUE}3️⃣  查询审计追踪...${NC}"
echo ""
echo -e "  ${BOLD}所有审计:${NC}"
${TAO_BIN} search audit: 2>/dev/null | head -10
echo ""
echo -e "  ${BOLD}严重告警:${NC}"
${TAO_BIN} by-tag critical 2>/dev/null | head -5
echo ""
echo -e "  ${BOLD}通过记录:${NC}"
${TAO_BIN} by-tag clean 2>/dev/null | head -5
echo ""

# 4. 统计
echo -e "${BLUE}4️⃣  系统统计${NC}"
echo ""
${TAO_BIN} stats 2>/dev/null | head -6
echo ""
${TAO_BIN} tag-cloud 2>/dev/null | head -6
echo ""

# 5. 打通说明
echo -e "${CYAN}══════════════════════════════════════════════${NC}"
echo -e "  🔗 集成完成！数据流:"
echo ""
echo "  LLM 回答 → 幻觉检测 → TaoStorage → 审计追溯"
echo ""
echo -e "  ${BOLD}命令速查:${NC}"
echo "  tao get audit:2026        查看某条审计详情"
echo "  tao by-tag critical       查看所有严重告警"
echo "  tao by-tag clean          查看通过记录"
echo "  tao tag-cloud             审计标签统计"
echo "  tao search hallucination  全文搜索"
echo "  tao stats                 存储统计"
echo ""
echo -e "  ${BOLD}在线仪表盘:${NC}"
echo "  https://malaxiya20250530-glitch.github.io/taostorage/audit.html"
echo ""
echo -e "  ${BOLD}Python 集成:${NC}"
echo "  python3 integration/tao_storage.py   # 测试集成层"
echo "  python3 -c \"from integration.gateway_hook import patch_gateway; patch_gateway()\"  # 修补网关自动审计"
echo ""
echo -e "${CYAN}══════════════════════════════════════════════${NC}"
