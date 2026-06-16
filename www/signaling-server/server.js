// ============================================================
// 🦀 TaoStorage 信令服务器 (WebSocket)
// 用于浏览器节点之间的 WebRTC 连接协商
// ============================================================
// 启动: node signaling-server/server.js
// 或集成到主 daemon: tao daemon start --enable-signaling
// ============================================================

const http = require('http');
const fs = require('fs');
const path = require('path');

// ============================================================
// 配置
// ============================================================
const PORT = process.env.PORT || 3000;
const WS_PORT = process.env.WS_PORT || 3001;
const DATA_DIR = process.env.TAO_DATA_DIR || './data';

// 节点注册表
const nodes = new Map(); // nodeId -> { ws, peerInfo, lastSeen }
const rooms = new Map(); // roomId -> Set<nodeId>

// ============================================================
// HTTP 服务器 (托管静态文件 + API)
// ============================================================
const server = http.createServer((req, res) => {
    // CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }

    const url = new URL(req.url, `http://${req.headers.host}`);
    const pathname = url.pathname;

    // ---- API 路由 ----
    if (pathname === '/api/stats') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
            online_nodes: nodes.size,
            rooms: rooms.size,
            uptime: process.uptime(),
            version: '0.3.0'
        }));
        return;
    }

    if (pathname === '/api/nodes') {
        const nodeList = Array.from(nodes.entries()).map(([id, info]) => ({
            id,
            role: info.role || 'unknown',
            lastSeen: info.lastSeen,
            address: info.address
        }));
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(nodeList));
        return;
    }

    if (pathname === '/api/invite/generate') {
        const inviteCode = Math.random().toString(36).substring(2, 10).toUpperCase();
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
            code: inviteCode,
            link: `https://tao.storage/?invite=${inviteCode}`,
            expires_in: '7d'
        }));
        return;
    }

    // ---- 静态文件 ----
    let filePath = path.join(__dirname, '..', pathname === '/' ? 'index.html' : pathname);

    const extMap = {
        '.html': 'text/html; charset=utf-8',
        '.css': 'text/css',
        '.js': 'application/javascript',
        '.json': 'application/json',
        '.wasm': 'application/wasm',
        '.png': 'image/png',
        '.svg': 'image/svg+xml',
    };

    const ext = path.extname(filePath);
    const contentType = extMap[ext] || 'application/octet-stream';

    fs.readFile(filePath, (err, data) => {
        if (err) {
            res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
            res.end('404 — 道可道，非常道。但此页不存在。');
            return;
        }
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
});

// ============================================================
// WebSocket 信令服务器
// ============================================================
const WebSocket = require('ws');
const wss = new WebSocket.Server({ port: WS_PORT });

console.log(`🦀 TaoStorage 信令服务器启动中...`);
console.log(`   HTTP:  http://0.0.0.0:${PORT}`);
console.log(`   WS:    ws://0.0.0.0:${WS_PORT}`);

wss.on('connection', (ws, req) => {
    const clientAddr = req.socket.remoteAddress;
    let nodeId = null;
    let roomId = null;

    console.log(`🔗 新连接: ${clientAddr}`);

    // ---- 心跳 ----
    const heartbeat = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'ping' }));
        }
    }, 30000);

    ws.on('message', (raw) => {
        try {
            const msg = JSON.parse(raw);
            handleMessage(ws, msg);
        } catch (e) {
            ws.send(JSON.stringify({ type: 'error', message: '无效消息格式' }));
        }
    });

    ws.on('close', () => {
        clearInterval(heartbeat);
        if (nodeId) {
            nodes.delete(nodeId);
            // 通知同房间节点
            broadcastToRoom(roomId, {
                type: 'peer_left',
                node_id: nodeId
            });
            console.log(`🔌 节点断开: ${nodeId} (在线: ${nodes.size})`);
        }
    });

    // ============================================================
    // 消息处理
    // ============================================================
    function handleMessage(ws, msg) {
        switch (msg.type) {
            case 'register':
                nodeId = msg.node_id || `anon-${Date.now()}`;
                roomId = msg.room || 'public';
                nodes.set(nodeId, {
                    ws,
                    role: msg.role || 'browser',
                    room: roomId,
                    address: clientAddr,
                    lastSeen: Date.now(),
                    version: msg.version || '0.3.0'
                });

                // 加入房间
                if (!rooms.has(roomId)) rooms.set(roomId, new Set());
                rooms.get(roomId).add(nodeId);

                // 返回确认 + 现有节点列表
                const peersInRoom = Array.from(rooms.get(roomId))
                    .filter(id => id !== nodeId && nodes.has(id));

                ws.send(JSON.stringify({
                    type: 'registered',
                    node_id: nodeId,
                    peers: peersInRoom,
                    online_count: nodes.size
                }));

                // 广播新节点上线
                broadcastToRoom(roomId, {
                    type: 'peer_joined',
                    node_id: nodeId,
                    role: msg.role || 'browser'
                }, nodeId);

                console.log(`✅ 节点注册: ${nodeId} (${msg.role}) — 在线: ${nodes.size}`);
                break;

            case 'offer':
                // 转发 WebRTC offer 给目标节点
                forwardToPeer(msg.target_id, {
                    type: 'offer',
                    from: nodeId,
                    sdp: msg.sdp
                });
                break;

            case 'answer':
                forwardToPeer(msg.target_id, {
                    type: 'answer',
                    from: nodeId,
                    sdp: msg.sdp
                });
                break;

            case 'ice_candidate':
                forwardToPeer(msg.target_id, {
                    type: 'ice_candidate',
                    from: nodeId,
                    candidate: msg.candidate
                });
                break;

            case 'pong':
                // 心跳回复
                if (nodeId && nodes.has(nodeId)) {
                    nodes.get(nodeId).lastSeen = Date.now();
                }
                break;

            case 'store':
                // 广播一条数据到房间 (简化版 DHT)
                broadcastToRoom(roomId, {
                    type: 'data_stored',
                    from: nodeId,
                    key: msg.key,
                    hash: msg.hash,
                    tags: msg.tags || []
                }, nodeId);
                break;

            case 'search':
                // 广播搜索请求
                broadcastToRoom(roomId, {
                    type: 'search_request',
                    from: nodeId,
                    query: msg.query,
                    request_id: msg.request_id
                }, nodeId);
                break;

            case 'search_result':
                forwardToPeer(msg.target_id, {
                    type: 'search_result',
                    from: nodeId,
                    request_id: msg.request_id,
                    results: msg.results
                });
                break;

            default:
                ws.send(JSON.stringify({
                    type: 'error',
                    message: `未知消息类型: ${msg.type}`
                }));
        }
    }

    function forwardToPeer(targetId, payload) {
        const peer = nodes.get(targetId);
        if (peer && peer.ws.readyState === WebSocket.OPEN) {
            peer.ws.send(JSON.stringify(payload));
        } else {
            ws.send(JSON.stringify({
                type: 'error',
                message: `节点 ${targetId} 不在线`
            }));
        }
    }

    function broadcastToRoom(room, payload, excludeId) {
        const members = rooms.get(room);
        if (!members) return;
        for (const memberId of members) {
            if (memberId === excludeId) continue;
            const member = nodes.get(memberId);
            if (member && member.ws.readyState === WebSocket.OPEN) {
                member.ws.send(JSON.stringify(payload));
            }
        }
    }
});

// ============================================================
// 启动 HTTP + WS
// ============================================================
server.listen(PORT, '0.0.0.0', () => {
    console.log(`✅ TaoStorage 服务器运行中`);
    console.log(`   🌐 浏览器节点: http://0.0.0.0:${PORT}`);
    console.log(`   🔌 信令: ws://0.0.0.0:${WS_PORT}`);
    console.log(`   🦀 道可道，非常道`);
});

// 优雅关闭
process.on('SIGTERM', () => {
    console.log('\n🛑 关闭服务器...');
    wss.close();
    server.close();
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('\n🛑 关闭服务器...');
    wss.close();
    server.close();
    process.exit(0);
});
