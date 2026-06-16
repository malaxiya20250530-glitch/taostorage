# ============================================================
# 🦀 TaoStorage Docker — 一键部署 P2P 存储节点
# ============================================================
# 构建:
#   docker build -t taostorage .
# 运行:
#   docker run -d -p 8788:8788 -p 3000:3000 -v tao-data:/root/.taostorage taostorage
# ============================================================

# ---- 构建阶段 ----
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev git && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release --bin tao && \
    # 也构建 WASM 浏览器节点
    cargo build --release --bin tao-browser 2>/dev/null || true

# ---- 运行阶段 ----
FROM node:22-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 复制 Rust 二进制
COPY --from=builder /app/target/release/tao /usr/local/bin/tao

# 复制浏览器节点
COPY --from=builder /app/www /opt/taostorage/www

# 安装信令服务器依赖
RUN cd /opt/taostorage/www/signaling-server && npm install --production

WORKDIR /opt/taostorage

EXPOSE 8788 3000 3001

# 默认启动：daemon + 浏览器节点
CMD ["sh", "-c", "\
    tao daemon start --background --network && \
    cd /opt/taostorage/www/signaling-server && \
    node server.js \
"]
