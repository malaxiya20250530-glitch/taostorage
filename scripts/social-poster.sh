#!/usr/bin/env bash
# ============================================================
# 📢 TaoStorage 社交媒体推广文案生成器
# 一键生成推文/帖子，复制即用
# ============================================================
# 用法: ./scripts/social-poster.sh [platform]
#   platform: twitter / zhihu / bilibili / reddit / all
# ============================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  📢 TaoStorage 社交媒体推广文案生成器     ║${NC}"
echo -e "${CYAN}║  复制即用，病毒式传播                      ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
echo ""

generate_twitter() {
    echo -e "${BOLD}🐦 Twitter / X (每条 ≤ 280 字符):${NC}"
    echo "──────────────────────────────────────"
    echo ""
    echo "1. 🦀 你的数据属于谁？Google? Meta? 腾讯？"
    echo ""
    echo "   不。属于你自己。"
    echo ""
    echo "   TaoStorage: 一行命令加入 P2P 存储网络。"
    echo "   存在每个人的手机上，不是某家公司的服务器。"
    echo ""
    echo "   curl -fsSL https://tao.storage/install.sh | bash"
    echo ""
    echo "   #TaoStorage #去中心化 #隐私 #Rust"
    echo ""
    echo "──────────────────────────────────────"
    echo ""
    echo "2. 🌐 浏览器即 P2P 节点。"
    echo ""
    echo "   打开 tao.storage → 自动成为分布式存储网络的一部分。"
    echo "   无需安装，无需注册，打开就是节点。"
    echo ""
    echo "   道可道，非常道。你的数据，你的道。"
    echo ""
    echo "   → https://tao.storage"
    echo ""
    echo "   #Web3 #P2P #存储 #道德经"
    echo ""
    echo "──────────────────────────────────────"
    echo ""
    echo "3. 🧘 我们用《道德经》设计了一个存储系统。"
    echo ""
    echo "   阴 = 数据本体   阳 = 元数据   气 = 生命状态"
    echo "   六十四卦 = 数据生命周期（屯→既济→泰→否→剥→坤）"
    echo ""
    echo "   这不是营销噱头——是真正的 Rust 代码。🦀"
    echo ""
    echo "   https://github.com/malaxiya20250530-glitch/taostorage"
    echo ""
    echo "   #RustLang #哲学 #编程 #开源"
}

generate_zhihu() {
    echo -e "${BOLD}📝 知乎 长文:${NC}"
    echo "──────────────────────────────────────"
    echo ""
    echo "标题：我用《道德经》写了一个去中心化存储系统"
    echo ""
    echo "道可道，非常道——这不是一句口号，是我项目 core/src/unit.rs 里的代码注释。"
    echo ""
    echo "我花了一个月，用 Rust 写了一个 P2P 分布式存储网络。"
    echo "核心设计理念来自《道德经》的阴阳道思想："
    echo ""
    echo "- 阴（Yin）：数据本体，通过 SHA256 内容哈希寻址"
    echo "- 阳（Yang）：元数据，名称、标签、访问热度"
    echo "- 气（Qi）：生命周期状态，六十四卦映射数据从写入到归档"
    echo ""
    echo "每个数据单元自带一个'气'守护进程，根据局部信息做自愈决策："
    echo "副本不足 → 复制；冗余不足 → 紧急修复；热度下降 → 冷归档"
    echo ""
    echo "技术栈：Rust + libp2p + sled + WASM + WebRTC"
    echo ""
    echo "最酷的是，打开网页（tao.storage）就能成为网络节点，"
    echo "不需要安装任何东西——浏览器就是你的 P2P 节点。"
    echo ""
    echo "GitHub: https://github.com/malaxiya20250530-glitch/taostorage"
    echo "安装: curl -fsSL https://tao.storage/install.sh | bash"
    echo ""
    echo "你的数据，你的道。"
}

generate_reddit() {
    echo -e "${BOLD}🤖 Reddit r/rust 帖:${NC}"
    echo "──────────────────────────────────────"
    echo ""
    echo "Title: 🦀 TaoStorage — A P2P storage network inspired by Tao Te Ching"
    echo ""
    echo "\"The Tao that can be told is not the eternal Tao\""
    echo ""
    echo "I built a decentralized P2P storage network in Rust,"
    echo "inspired by the Yin-Yang philosophy from Tao Te Ching."
    echo ""
    echo "Key concepts:"
    echo "• Yin (阴) = data payload + content addressing (SHA256)"
    echo "• Yang (阳) = metadata, tags, access heat"
    echo "• Qi (气) = lifecycle state machine with self-healing"
    echo "• 6 hexagrams = data lifecycle (birth → stable → hot → cold → repair → archive)"
    echo ""
    echo "Tech: Rust + libp2p + sled + WASM + WebRTC + Zero-Knowledge Proofs"
    echo ""
    echo "What makes it spread:"
    echo "• One-line install: curl -fsSL https://tao.storage/install.sh | bash"
    echo "• Browser node: open tao.storage → become a P2P node instantly"
    echo "• npm / Docker / GitHub Action — embed anywhere"
    echo ""
    echo "GitHub: https://github.com/malaxiya20250530-glitch/taostorage"
    echo "Live demo: https://tao.storage"
    echo ""
    echo "Your data, your Tao. 🦀"
}

generate_bilibili() {
    echo -e "${BOLD}📺 B站 视频简介:${NC}"
    echo "──────────────────────────────────────"
    echo ""
    echo "【一行命令加入全球P2P网络】我用Rust写了去中心化存储！"
    echo ""
    echo "🦀 TaoStorage — 道可道，非常道"
    echo ""
    echo "你的数据属于谁？Google？腾讯？还是你自己？"
    echo ""
    echo "这个视频介绍了一个用 Rust 写的 P2P 分布式存储网络："
    echo "✅ 一行命令安装，自动成为节点"
    echo "✅ 浏览器打开就是节点，无需安装"
    echo "✅ 阴阳道哲学设计，六十四卦管理数据"
    echo "✅ 零知识证明保证存储真实性"
    echo ""
    echo "安装体验：curl -fsSL https://tao.storage/install.sh | bash"
    echo "代码开源：链接在简介"
    echo ""
    echo "你的数据，你的道。"
}

# ---- 平台选择 ----
case "${1:-all}" in
    twitter|x)
        generate_twitter
        ;;
    zhihu|zh)
        generate_zhihu
        ;;
    reddit|r/rust)
        generate_reddit
        ;;
    bilibili|b站|b)
        generate_bilibili
        ;;
    all|*)
        generate_twitter
        echo ""
        echo -e "${CYAN}══════════════════════════════════════════${NC}"
        echo ""
        generate_zhihu
        echo ""
        echo -e "${CYAN}══════════════════════════════════════════${NC}"
        echo ""
        generate_reddit
        echo ""
        echo -e "${CYAN}══════════════════════════════════════════${NC}"
        echo ""
        generate_bilibili
        ;;
esac

echo ""
echo -e "${GREEN}📢 复制上面的文案，直接发布！${NC}"
echo -e "${YELLOW}💡 修改 [malaxiya20250530-glitch] 为你的 GitHub 用户名${NC}"
