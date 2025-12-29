// Service Worker for uKeep PWA
// 提供离线访问和资源缓存功能

const CACHE_NAME = 'ukeep-v1';
const ASSETS_TO_CACHE = [
  '/',
  '/assets/main.css',
  '/assets/icon-192.png',
  '/assets/icon-512.png',
];

// 安装 Service Worker 时缓存核心资源
self.addEventListener('install', (event) => {
  console.log('[SW] Installing Service Worker...');
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      console.log('[SW] Caching app shell');
      return cache.addAll(ASSETS_TO_CACHE);
    })
  );
  // 强制激活新的 Service Worker
  self.skipWaiting();
});

// 激活时清理旧缓存
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating Service Worker...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log('[SW] Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
  // 立即控制所有页面
  return self.clients.claim();
});

// 拦截网络请求，优先使用缓存（Cache First 策略）
self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => {
      // 如果缓存中有，直接返回
      if (response) {
        return response;
      }

      // 否则发起网络请求
      return fetch(event.request).then((response) => {
        // 只缓存成功的 GET 请求
        if (
          !response ||
          response.status !== 200 ||
          response.type === 'error' ||
          event.request.method !== 'GET'
        ) {
          return response;
        }

        // 克隆响应并缓存
        const responseToCache = response.clone();
        caches.open(CACHE_NAME).then((cache) => {
          cache.put(event.request, responseToCache);
        });

        return response;
      }).catch(() => {
        // 网络失败时，如果是导航请求，返回缓存的首页
        if (event.request.mode === 'navigate') {
          return caches.match('/');
        }
      });
    })
  );
});
