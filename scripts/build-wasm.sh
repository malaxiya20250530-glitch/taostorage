#!/usr/bin/env bash
# ============================================================
# 🌐 TaoStorage WASM 编译脚本
# 构建浏览器节点 WebAssembly 模块
# ============================================================
# 依赖: wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
# 用法: ./scripts/build-wasm.sh
# ============================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🌐 构建 TaoStorage WASM 浏览器节点...${NC}"

# 检查 wasm-pack
if ! command -v wasm-pack &>/dev/null; then
    echo -e "${BLUE}📦 安装 wasm-pack...${NC}"
    curl -fsSL https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
fi

# 检查 wasm32 目标
if ! rustup target list --installed | grep -q wasm32; then
    echo -e "${BLUE}📦 安装 wasm32-unknown-unknown 目标...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# 构建
echo -e "${BLUE}🔨 编译中...${NC}"
cd "$(dirname "$0")/../www"
wasm-pack build tao-browser --target web --out-dir ../pkg

echo -e "${GREEN}✅ WASM 构建完成！${NC}"
echo -e "   输出目录: www/pkg/"
echo -e "   启动服务: cd www && npx serve ."
echo -e "   或使用:   tao browser --port 3000"
