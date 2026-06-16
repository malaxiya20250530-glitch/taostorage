#!/usr/bin/env bash
# ============================================================
# 🧲 TaoStorage 邀请奖励系统 (独立版)
# 用于非 Rust 环境的邀请码生成和追踪
# ============================================================
# 用法:
#   ./invite.sh generate <node_id>    # 生成邀请码
#   ./invite.sh use <code> <node_id>  # 使用邀请码
#   ./invite.sh leaderboard           # 查看排行榜
#   ./invite.sh stats <node_id>       # 查看节点统计
# ============================================================

set -euo pipefail

INVITE_DIR="${TAO_HOME:-${HOME}/.taostorage}/invites"
INVITE_FILE="${INVITE_DIR}/invites.json"
REPUTATION_FILE="${INVITE_DIR}/reputation.json"
LEADERBOARD_FILE="${INVITE_DIR}/leaderboard.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "${INVITE_DIR}"

# ---- 初始化数据文件 ----
init_data() {
    if [ ! -f "${INVITE_FILE}" ]; then echo '{}' > "${INVITE_FILE}"; fi
    if [ ! -f "${REPUTATION_FILE}" ]; then echo '{}' > "${REPUTATION_FILE}"; fi
}

# ---- 生成邀请码 ----
generate_code() {
    local node_id="$1"
    init_data

    # 生成 8 字符邀请码
    local code=$(echo "${node_id}:$(date +%s):${RANDOM}" | sha256sum | head -c 8 | tr 'a-f' 'A-F')
    local now=$(date +%s)
    local expires=$((now + 7 * 24 * 3600))

    # 写入邀请码
    local invites=$(cat "${INVITE_FILE}")
    invites=$(echo "${invites}" | jq --arg c "$code" --arg n "$node_id" --argjson t "$now" --argjson e "$expires" \
        '. + {($c): {code: $c, inviter: $n, created_at: $t, expires_at: $e, uses: 0, max_uses: 0, active: true}}')
    echo "${invites}" > "${INVITE_FILE}"

    echo -e "${GREEN}✅ 邀请码已生成: ${BOLD}${code}${NC}"
    echo -e "   ${BLUE}邀请链接: https://tao.storage/?invite=${code}${NC}"
    echo -e "   ${YELLOW}有效期: 7 天${NC}"
    echo -e "   ${YELLOW}每邀请一人，信誉 +10 🏆${NC}"
}

# ---- 使用邀请码 ----
use_code() {
    local code="$1"
    local new_node_id="$2"
    init_data

    local invites=$(cat "${INVITE_FILE}")

    # 检查邀请码是否存在
    local inviter=$(echo "${invites}" | jq -r ".[\"${code}\"].inviter // empty")
    if [ -z "${inviter}" ]; then
        echo -e "${RED}❌ 邀请码无效: ${code}${NC}"
        return 1
    fi

    # 检查是否过期
    local expires=$(echo "${invites}" | jq -r ".[\"${code}\"].expires_at // 0")
    local now=$(date +%s)
    if [ "${now}" -gt "${expires}" ]; then
        echo -e "${RED}❌ 邀请码已过期${NC}"
        return 1
    fi

    # 检查是否激活
    local active=$(echo "${invites}" | jq -r ".[\"${code}\"].active // false")
    if [ "${active}" != "true" ]; then
        echo -e "${RED}❌ 邀请码已失效${NC}"
        return 1
    fi

    # 更新使用次数
    invites=$(echo "${invites}" | jq ".[\"${code}\"].uses += 1" )
    echo "${invites}" > "${INVITE_FILE}"

    # 奖励邀请方 (+10)
    local reputation=$(cat "${REPUTATION_FILE}")
    local inviter_score=$(echo "${reputation}" | jq -r ".[\"${inviter}\"].score // 0")
    inviter_score=$((inviter_score + 10))
    reputation=$(echo "${reputation}" | jq \
        --arg n "${inviter}" \
        --argjson s "${inviter_score}" \
        --arg d "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --arg desc "邀请节点 ${new_node_id}" \
        '.[$n].score = $s | .[$n].invites += 1 | .[$n].events += [{"timestamp": $d, "event": "invite", "delta": 10, "description": $desc}]')
    echo "${reputation}" > "${REPUTATION_FILE}"

    # 奖励新节点 (+5)
    local new_score=$(echo "${reputation}" | jq -r ".[\"${new_node_id}\"].score // 0")
    new_score=$((new_score + 5))
    reputation=$(echo "${reputation}" | jq \
        --arg n "${new_node_id}" \
        --argjson s "${new_score}" \
        --arg d "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --arg desc "通过邀请码 ${code} 加入" \
        '.[$n].score = $s | .[$n].events += [{"timestamp": $d, "event": "joined", "delta": 5, "description": $desc}]')
    echo "${reputation}" > "${REPUTATION_FILE}"

    echo -e "${GREEN}🎉 邀请成功！${NC}"
    echo -e "   ${BLUE}邀请方 ${inviter}: 信誉 +10 🏆${NC}"
    echo -e "   ${BLUE}新节点 ${new_node_id}: 信誉 +5 🏆${NC}"
}

# ---- 排行榜 ----
show_leaderboard() {
    init_data
    local reputation=$(cat "${REPUTATION_FILE}")

    echo ""
    echo -e "${CYAN}══════════════════════════════════════${NC}"
    echo -e "${CYAN}  🏆 TaoStorage 节点排行榜${NC}"
    echo -e "${CYAN}══════════════════════════════════════${NC}"
    echo ""

    # 按分数排序
    echo "${reputation}" | jq -r '
        to_entries
        | sort_by(-.value.score)
        | to_entries[]
        | "\(.key + 1)|\(.value.key)|\(.value.value.score)|\(.value.value.invites // 0)"
    ' 2>/dev/null | while IFS='|' read rank node score invites; do
        local icon=""
        if [ "${rank}" -eq 1 ]; then icon="🥇";
        elif [ "${rank}" -eq 2 ]; then icon="🥈";
        elif [ "${rank}" -eq 3 ]; then icon="🥉";
        else icon="   "; fi

        # 等级
        local rank_name="凡"
        if [ "${score}" -ge 1000 ]; then rank_name="圣";
        elif [ "${score}" -ge 200 ]; then rank_name="玄";
        elif [ "${score}" -ge 50 ]; then rank_name="道";
        elif [ "${score}" -ge 10 ]; then rank_name="士"; fi

        printf "  %s #%-2d  %-20s  🏆 %4d  %s  🤝 %d 邀请\n" \
            "${icon}" "${rank}" "${node:0:20}" "${score}" "${rank_name}" "${invites:-0}"
    done

    echo ""
    echo -e "   ${YELLOW}等级: 凡(0) → 士(10) → 道(50) → 玄(200) → 圣(1000)${NC}"
}

# ---- 节点统计 ----
show_stats() {
    local node_id="$1"
    init_data
    local reputation=$(cat "${REPUTATION_FILE}")

    local score=$(echo "${reputation}" | jq -r ".[\"${node_id}\"].score // 0")
    local invites=$(echo "${reputation}" | jq -r ".[\"${node_id}\"].invites // 0")

    local rank_name="凡"
    if [ "${score}" -ge 1000 ]; then rank_name="圣 🏆";
    elif [ "${score}" -ge 200 ]; then rank_name="玄";
    elif [ "${score}" -ge 50 ]; then rank_name="道";
    elif [ "${score}" -ge 10 ]; then rank_name="士"; fi

    echo ""
    echo -e "${CYAN}══════════════════════════════════════${NC}"
    echo -e "${CYAN}  📊 节点 ${node_id} 统计${NC}"
    echo -e "${CYAN}══════════════════════════════════════${NC}"
    echo -e "   节点 ID:    ${node_id}"
    echo -e "   信誉分:     ${BOLD}${score}${NC} 🏆"
    echo -e "   等级:       ${BOLD}${rank_name}${NC}"
    echo -e "   邀请数:     ${invites} 🤝"
    echo ""

    # 最近事件
    echo -e "   ${BLUE}最近活动:${NC}"
    echo "${reputation}" | jq -r ".[\"${node_id}\"].events // [] | reverse | .[0:5][] | \"    \(.timestamp[0:19]) \(.description)\"" 2>/dev/null || echo "    暂无记录"
}

# ---- 导出所有数据 ----
export_data() {
    init_data
    cat "${REPUTATION_FILE}"
}

# ============================================================
# 主入口
# ============================================================
case "${1:-help}" in
    generate|gen)
        if [ -z "${2:-}" ]; then
            echo -e "${RED}用法: $0 generate <node_id>${NC}"
            exit 1
        fi
        generate_code "$2"
        ;;
    use)
        if [ -z "${2:-}" ] || [ -z "${3:-}" ]; then
            echo -e "${RED}用法: $0 use <code> <new_node_id>${NC}"
            exit 1
        fi
        use_code "$2" "$3"
        ;;
    leaderboard|rank|lb)
        show_leaderboard
        ;;
    stats|status)
        if [ -z "${2:-}" ]; then
            echo -e "${RED}用法: $0 stats <node_id>${NC}"
            exit 1
        fi
        show_stats "$2"
        ;;
    export)
        export_data
        ;;
    *)
        echo -e "${CYAN}🧲 TaoStorage 邀请奖励系统${NC}"
        echo ""
        echo "用法:"
        echo "  $0 generate <node_id>       生成邀请码"
        echo "  $0 use <code> <node_id>     使用邀请码"
        echo "  $0 leaderboard              查看排行榜"
        echo "  $0 stats <node_id>          查看节点统计"
        echo "  $0 export                   导出数据"
        echo ""
        echo "示例:"
        echo "  $0 generate tao-node-1"
        echo "  $0 use AB12CD34 tao-node-2"
        echo "  $0 leaderboard"
        ;;
esac
