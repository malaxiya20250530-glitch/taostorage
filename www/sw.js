// ============================================================
// 🦀 TaoStorage — PWA Service Worker
// 离线缓存 + 快速加载 + 后台同步
// ============================================================

const CACHE_NAME = 'taostorage-v1';
const ASSETS = [
    '/',
    '/index.html',
    '/css/style.css',
    '/manifest.json',
    'https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;600;700;800&display=swap'
];

// 安装：预缓存关键资源
self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME).then(cache => {
            return cache.addAll(ASSETS);
        })
    );
    self.skipWaiting();
});

// 激活：清理旧缓存
self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(keys => {
            return Promise.all(
                keys.filter(key => key !== CACHE_NAME)
                    .map(key => caches.delete(key))
            );
        })
    );
    self.clients.claim();
});

// 拦截请求：缓存优先
self.addEventListener('fetch', event => {
    // 只缓存 GET 请求
    if (event.request.method !== 'GET') return;

    // API 请求不缓存
    if (event.request.url.includes('/api/')) {
        event.respondWith(fetch(event.request));
        return;
    }

    event.respondWith(
        caches.match(event.request).then(cached => {
            // 缓存命中则返回，否则网络获取并缓存
            return cached || fetch(event.request).then(response => {
                return caches.open(CACHE_NAME).then(cache => {
                    cache.put(event.request, response.clone());
                    return response;
                });
            }).catch(() => {
                // 离线时返回缓存的首页
                if (event.request.mode === 'navigate') {
                    return caches.match('/');
                }
                return new Response('离线中', { status: 503 });
            });
        })
    );
});

// 后台同步：离线时存储的数据在网络恢复后同步
self.addEventListener('sync', event => {
    if (event.tag === 'sync-data') {
        event.waitUntil(syncData());
    }
});

async function syncData() {
    const cache = await caches.open(CACHE_NAME);
    const pending = await cache.match('/pending-sync');
    if (pending) {
        const data = await pending.json();
        // 尝试发送数据到服务器
        for (const item of data) {
            try {
                await fetch('/api/store', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(item)
                });
            } catch (e) {
                console.error('Sync failed:', e);
            }
        }
        await cache.delete('/pending-sync');
    }
}
