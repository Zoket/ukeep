use crate::components::ItemCard;
use crate::router::Route;
use crate::state::InventoryState;
use crate::storage::{export_data, import_data_from_json};
use dioxus::prelude::*;
use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

#[component]
pub fn Home() -> Element {
    let mut inventory = use_context::<InventoryState>().0;

    // 保持 FileReader 存活，防止异步导入回调被取消
    let file_reader_slot = use_signal(|| Option::<FileReader>::None);

    // 控制设置弹出菜单的显示
    let mut show_settings = use_signal(|| false);

    // 错误提示信息
    let mut error_message = use_signal(|| Option::<String>::None);

    // 排序逻辑：按剩余天数升序 (快过期的在前面)
    let mut sorted_items = inventory.read().clone();
    sorted_items.sort_by_key(|item| item.days_remaining());

    // 统计：多少个即将过期 (<=3天)
    let urgent_count = sorted_items
        .iter()
        .filter(|i| i.days_remaining() <= 3)
        .count();

    // Handler: 模拟 "吃掉了"
    let consume_item = move |id: Uuid| {
        inventory.write().retain(|i| i.id() != id);
    };

    // Handler: 模拟 "扔掉了"
    let waste_item = move |id: Uuid| {
        inventory.write().retain(|i| i.id() != id);
    };

    // Handler: 导出数据
    let handle_export = move |_| {
        let items = inventory.read().clone();
        match export_data(&items) {
            Ok(_) => {
                show_settings.set(false);
            }
            Err(e) => {
                error_message.set(Some(format!("导出失败: {}", e)));
            }
        }
    };

    // Handler: 导入数据
    let handle_import = move |_| {
        show_settings.set(false);

        let mut reader_slot = file_reader_slot.clone();

        // 创建隐藏的文件输入元素
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Ok(input) = document.create_element("input") {
                    if let Ok(input) = input.dyn_into::<HtmlInputElement>() {
                        input.set_type("file");
                        input.set_accept(".json");

                        let inventory_clone = inventory.clone();
                        let error_msg_clone = error_message.clone();

                        let onchange = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
                            if let Some(target) = event.target() {
                                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                                    if let Some(files) = input.files() {
                                        if let Some(file) = files.get(0) {
                                            let file = File::from(file);
                                            let mut inventory_inner = inventory_clone.clone();
                                            let mut error_inner = error_msg_clone.clone();
                                            let mut reader_slot_inner = reader_slot.clone();

                                            let reader = read_as_text(&file, move |result| {
                                                match result {
                                                    Ok(text) => {
                                                        match import_data_from_json(&text) {
                                                            Ok(items) => {
                                                                inventory_inner.set(items);
                                                                error_inner.set(None);
                                                            }
                                                            Err(e) => {
                                                                error_inner.set(Some(format!("导入失败: {}", e)));
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        error_inner.set(Some("读取文件失败".to_string()));
                                                    }
                                                }
                                                // 读取完成后清理 FileReader
                                                reader_slot_inner.set(None);
                                            });
                                            // 保持 FileReader 存活直到回调执行
                                            reader_slot.set(Some(reader));
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);

                        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
                        onchange.forget();

                        input.click();
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "flex-col", style: "padding: 16px; max-width: 600px; margin: 0 auto;",
            // --- Header ---
            header {
                // 使用 flex 布局，将设置按钮放在右上角
                div { style: "display: flex; justify-content: space-between; align-items: flex-start;",
                    div {
                        h1 { "我的冰箱 🧊" }
                        span { class: "subtitle",
                            if urgent_count > 0 {
                                "⚠️ 有 {urgent_count} 个物品需要尽快处理"
                            } else {
                                "👏 一切看起来都很新鲜"
                            }
                        }
                    }

                    // 设置按钮和下拉菜单容器
                    div { style: "position: relative;",
                        // 设置按钮
                        button {
                            class: "material-symbols-outlined",
                            style: "background: none; border: none; color: #999; cursor: pointer; padding: 8px; font-size: 24px;",
                            onclick: move |_| show_settings.set(!show_settings()),
                            "settings"
                        }

                        // 下拉菜单
                        if show_settings() {
                            // 透明遮罩层（捕获外部点击）
                            div {
                                style: "position: fixed; inset: 0; z-index: 10; cursor: default;",
                                onclick: move |_| show_settings.set(false),
                            }

                            // 菜单内容
                            div {
                                style: "position: absolute; right: 0; margin-top: 8px; width: 200px; background: white; border-radius: 12px; box-shadow: 0 10px 40px rgba(0,0,0,0.15); z-index: 20; border: 1px solid #f0f0f0; overflow: hidden;",

                                // 导出数据
                                button {
                                    class: "menu-item",
                                    style: "width: 100%; text-align: left; padding: 12px 16px; background: none; border: none; cursor: pointer; display: flex; align-items: center; gap: 8px; font-size: 14px;",
                                    onclick: handle_export,
                                    span { class: "material-symbols-outlined", style: "color: #2196F3; font-size: 20px;", "download" }
                                    span { "导出数据" }
                                }

                                // 分隔线
                                div { style: "height: 1px; background: #f5f5f5;" }

                                // 导入数据
                                button {
                                    class: "menu-item",
                                    style: "width: 100%; text-align: left; padding: 12px 16px; background: none; border: none; cursor: pointer; display: flex; align-items: center; gap: 8px; font-size: 14px;",
                                    onclick: handle_import,
                                    span { class: "material-symbols-outlined", style: "color: #4CAF50; font-size: 20px;", "upload" }
                                    span { "导入数据" }
                                }
                            }
                        }
                    }
                }
            }

            // 错误提示
            if let Some(err) = error_message() {
                div {
                    style: "background: #ffebee; color: #c62828; padding: 12px; border-radius: 8px; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;",
                    span { "{err}" }
                    button {
                        style: "background: none; border: none; color: #c62828; cursor: pointer; font-size: 18px;",
                        onclick: move |_| error_message.set(None),
                        "✕"
                    }
                }
            }

            // --- List View ---
            div { class: "flex-col",
                for item in sorted_items {
                    ItemCard {
                        key: "{item.id()}",
                        item: item.clone(),
                        on_consume: consume_item,
                        on_waste: waste_item
                    }
                }
            }
        }

        // --- FAB ---
        Link { to: Route::AddItem {}, class: "fab",
            span { class: "material-symbols-outlined", "add" }
        }
    }
}
