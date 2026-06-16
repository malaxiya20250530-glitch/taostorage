#!/usr/bin/env bash
# ============================================================
# 🦀 TaoStorage 一键安装脚本
# "道可道，非常道" — 一行命令加入全球 P2P 存储网络
# ============================================================
# 用法:
#   curl -fsSL https://tao.storage/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/<user>/taostorage/main/scripts/install.sh | bash
# ============================================================

set -euo pipefail

# ---- 颜色 ----
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ---- Banner ----
cat << "EOF"
╔══════════════════════════════════════════════════╗
║   🦀  TaoStorage — 道可道，非常道                ║
║   个人数据仓库 + P2P 分布式存储网络              ║
╚══════════════════════════════════════════════════╝
EOF
echo ""

# ---- 辅助函数 ----
info()  { echo -e "${BLUE}ℹ️${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# ---- 路径配置 ----
TAO_HOME="${HOME}/.taostorage"
TAO_BIN_DIR="${HOME}/.tao/bin"
TAO_BIN="${TAO_BIN_DIR}/tao"
TAO_VERSION="latest"
REPO="malaxiya20250530-glitch/taostorage"

# ---- 检测平台 ----
detect_platform() {
    ARCH="$(uname -m)"
    OS="$(uname -s)"
    case "${OS}" in
        Linux)  OS="unknown-linux-gnu" ;;
        Darwin) OS="apple-darwin" ;;
        *)      err "不支持的操作系统: ${OS}"; exit 1 ;;
    esac
    case "${ARCH}" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)  err "不支持的架构: ${ARCH} (支持 x86_64 / aarch64)"; exit 1 ;;
    esac
    TARGET="${ARCH}-${OS}"
    info "检测到系统: ${TARGET}"
}

# ---- 检查依赖 ----
check_deps() {
    local missing=()
    for cmd in curl tar; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        warn "缺少依赖: ${missing[*]}"
        if command -v apt &>/dev/null; then
            info "尝试安装依赖..."
            sudo apt update && sudo apt install -y "${missing[@]}"
        elif command -v pkg &>/dev/null; then
            info "Termux 环境，安装依赖..."
            pkg install -y "${missing[@]}"
        else
            err "请手动安装: ${missing[*]}"
            exit 1
        fi
    fi
    ok "依赖检查通过"
}

# ---- 下载二进制 ----
download_binary() {
    info "下载 TaoStorage ${TAO_VERSION} (${TARGET})..."

    # 尝试从 GitHub Releases 下载
    local url
    if [ "${TAO_VERSION}" = "latest" ]; then
        url="https://github.com/${REPO}/releases/latest/download/tao-${TARGET}.tar.gz"
    else
        url="https://github.com/${REPO}/releases/download/${TAO_VERSION}/tao-${TARGET}.tar.gz"
    fi

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    local archive="${tmp_dir}/tao.tar.gz"

    if curl -fsSL "${url}" -o "${archive}" 2>/dev/null; then
        ok "二进制下载成功"
    else
        warn "预编译二进制不可用，进入源码安装模式..."
        build_from_source
        return
    fi

    # 解压安装
    mkdir -p "${TAO_BIN_DIR}"
    tar xzf "${archive}" -C "${tmp_dir}"
    cp "${tmp_dir}/tao" "${TAO_BIN}"
    chmod +x "${TAO_BIN}"
    rm -rf "${tmp_dir}"
    ok "安装到: ${TAO_BIN}"
}

# ---- 源码编译 ----
build_from_source() {
    info "🦀 从源码编译 TaoStorage..."

    # 检查 Rust
    if ! command -v rustc &>/dev/null; then
        info "安装 Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "${HOME}/.cargo/env"
    fi

    # 克隆或更新
    local src_dir="${HOME}/taostorage-src"
    if [ -d "${src_dir}" ]; then
        info "更新源代码..."
        cd "${src_dir}" && git pull
    else
        info "克隆源代码..."
        git clone --depth 1 "https://github.com/${REPO}.git" "${src_dir}"
        cd "${src_dir}"
    fi

    # 编译
    info "编译 (首次编译可能需要 5-10 分钟)..."
    cargo build --release --bin tao

    # 安装
    mkdir -p "${TAO_BIN_DIR}"
    cp "target/release/tao" "${TAO_BIN}"
    chmod +x "${TAO_BIN}"
    ok "源码编译完成，安装到: ${TAO_BIN}"
}

# ---- 初始化配置 ----
setup_config() {
    mkdir -p "${TAO_HOME}"

    # 创建默认配置文件
    if [ ! -f "${TAO_HOME}/config.json" ]; then
        cat > "${TAO_HOME}/config.json" << CONFIG
{
  "version": "0.3.0",
  "dataDir": "${TAO_HOME}/data",
  "apiPort": 8788,
  "p2p": {
    "enabled": true,
    "listenAddr": "/ip4/0.0.0.0/tcp/0",
    "bootstrapNodes": []
  },
  "inviteCode": "$(openssl rand -hex 4 2>/dev/null || echo "tao-$(date +%s)-$$")"
}
CONFIG
        ok "配置文件已创建: ${TAO_HOME}/config.json"
    fi

    # 添加到 PATH
    local shell_rc
    case "${SHELL}" in
        */zsh)  shell_rc="${HOME}/.zshrc" ;;
        */bash) shell_rc="${HOME}/.bashrc" ;;
        *)      shell_rc="${HOME}/.profile" ;;
    esac

    if ! grep -q "TAO_HOME" "${shell_rc}" 2>/dev/null; then
        {
            echo ""
            echo "# TaoStorage — 个人数据仓库"
            echo "export PATH=\"\${PATH}:${TAO_BIN_DIR}\""
            echo "export TAO_HOME=\"${TAO_HOME}\""
        } >> "${shell_rc}"
        ok "已添加到 PATH (${shell_rc})"
    fi

    export PATH="${PATH}:${TAO_BIN_DIR}"
}

# ---- 启动守护进程 ----
start_daemon() {
    if command -v "${TAO_BIN}" &>/dev/null; then
        info "启动 TaoStorage 守护进程..."
        "${TAO_BIN}" daemon start --background 2>/dev/null || true
        ok "守护进程已启动 (API: http://127.0.0.1:8788)"
    fi
}

# ---- 引导加入网络 ----
join_network() {
    echo ""
    echo -e "${CYAN}══════════════════════════════════════════${NC}"
    echo -e "${CYAN}  🌐 你已成为 TaoStorage 网络的一个节点！${NC}"
    echo -e "${CYAN}══════════════════════════════════════════${NC}"
    echo ""

    if command -v "${TAO_BIN}" &>/dev/null; then
        # 存入第一条数据
        "${TAO_BIN}" put 道 "道可道，非常道" --tag welcome --tag genesis 2>/dev/null || true
        echo ""
        echo -e "${GREEN}📝 已写入第一条数据: '道可道，非常道'${NC}"
        echo ""
        echo -e "  ${BOLD}快速开始:${NC}"
        echo -e "  ${GREEN}tao put 阴阳 \"一阴一阳之谓道\" --tag philosophy${NC}"
        echo -e "  ${GREEN}tao get 道${NC}"
        echo -e "  ${GREEN}tao list${NC}"
        echo -e "  ${GREEN}tao stats${NC}"
        echo -e "  ${GREEN}tao daemon status${NC}"
    fi

    echo ""
    echo -e "  ${YELLOW}📢 分享邀请:${NC}"
    echo -e "  让朋友也加入网络:"
    echo -e "  ${BLUE}curl -fsSL https://tao.storage/install.sh | bash${NC}"
    echo ""
    echo -e "  ${YELLOW}🌟 每邀请一个人，你的节点信誉 +1${NC}"
    echo -e "  ${YELLOW}🌐 节点越多，网络越强！${NC}"
}

# ---- 清理旧版本 ----
cleanup_old() {
    local old_bin="${HOME}/.cargo/bin/tao"
    if [ -f "${old_bin}" ]; then
        warn "发现旧版本，移除..."
        rm -f "${old_bin}"
    fi
}

# ============================================================
# 主流程
# ============================================================
main() {
    echo -e "${BOLD}🚀 开始安装 TaoStorage v0.3.0...${NC}"
    echo ""

    detect_platform
    check_deps
    cleanup_old
    download_binary
    setup_config
    start_daemon
    join_network

    echo ""
    echo -e "${GREEN}${BOLD}🎉 安装完成！重新打开终端即可使用 'tao' 命令。${NC}"
    echo ""
    echo -e "${CYAN}道可道，非常道 — 你的数据，你的道。${NC}"
}

main "$@"
