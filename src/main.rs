#![allow(non_snake_case)]

use dioxus::prelude::*;
use ukeep::router::Route;
use ukeep::state::InventoryState;
use ukeep::storage::{load_inventory, save_inventory};

static CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    // 从 LocalStorage 加载数据初始化全局状态
    use_context_provider(|| InventoryState(Signal::new(load_inventory())));

    // 获取全局状态用于监听变化
    let inventory = use_context::<InventoryState>().0;

    // 自动保存：监听状态变化，自动持久化到 LocalStorage
    use_effect(move || {
        let items = inventory.read().clone();
        save_inventory(&items);
    });

    // 注册 Service Worker (PWA 支持)
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::JsFuture;
            use web_sys::window;

            if let Some(window) = window() {
                let navigator = window.navigator().service_worker();
                wasm_bindgen_futures::spawn_local(async move {
                    let promise = navigator.register("/assets/sw.js");
                    match JsFuture::from(promise).await {
                        Ok(_) => log::info!("[PWA] Service Worker registered successfully"),
                        Err(e) => log::error!("[PWA] Service Worker registration failed: {:?}", e),
                    }
                });
            }
        }
    });

    rsx! {
        // 注入全局样式
        document::Stylesheet { href: CSS }
        // 注入 Material Symbols 字体 (用于图标)
        link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0" }

        // PWA 支持
        document::Link { rel: "manifest", href: "/assets/manifest.json" }
        document::Meta { name: "theme-color", content: "#2563eb" }
        document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
        document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "default" }
        document::Meta { name: "apple-mobile-web-app-title", content: "uKeep" }
        document::Link { rel: "apple-touch-icon", href: "/assets/icon-512.png" }

        Router::<Route> {}
    }
}
