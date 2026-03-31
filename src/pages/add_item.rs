use crate::models::Item;
use crate::router::Route;
use crate::state::InventoryState;
use chrono::{Duration, Local, NaiveDate};
use dioxus::prelude::*;

#[component]
pub fn AddItem() -> Element {
    let mut inventory = use_context::<InventoryState>().0;
    let navigator = use_navigator();

    // Form State
    let mut name = use_signal(|| "".to_string());
    let mut production_date_str = use_signal(|| Local::now().format("%Y-%m-%d").to_string());
    let mut quantity_str = use_signal(|| "1".to_string());
    let mut expiry_date_str = use_signal(|| Local::now().format("%Y-%m-%d").to_string());

    // Quick Chips Data
    let quick_options = vec![
        ("🥛 牛奶", 7, "🥛"),
        ("🥬 蔬菜", 5, "🥬"),
        ("🍞 面包", 3, "🍞"),
        ("🥚 鸡蛋", 15, "🥚"),
        ("🥩 生肉", 2, "🥩"),
    ];

    let submit = move |_| {
        if name.read().is_empty() {
            return;
        }

        if let Ok(parsed_date) = NaiveDate::parse_from_str(&expiry_date_str.read(), "%Y-%m-%d") {
            let item_name = name.read().clone();
            let quantity = quantity_str
                .read()
                .parse::<u32>()
                .ok()
                .filter(|q| *q >= 1)
                .unwrap_or(1);

            inventory.write().push(Item::new_with_quantity(
                item_name,
                parsed_date,
                quantity,
            ));
            navigator.go_back();
        }
    };

    // Helper: 点击 Chip 自动填入名称和过期日期
    let mut apply_chip = move |n: &str, days: i64, _e: &str| {
        name.set(n.to_string());
        // 基于生产日期计算过期日期
        if let Ok(prod_date) = NaiveDate::parse_from_str(&production_date_str.read(), "%Y-%m-%d") {
            let exp_date = prod_date + Duration::days(days);
            expiry_date_str.set(exp_date.format("%Y-%m-%d").to_string());
        }
    };

    // Helper: 快速设置过期日期（基于生产日期 + X天）
    let mut add_days = move |days: i64| {
        if let Ok(prod_date) = NaiveDate::parse_from_str(&production_date_str.read(), "%Y-%m-%d") {
            let exp_date = prod_date + Duration::days(days);
            expiry_date_str.set(exp_date.format("%Y-%m-%d").to_string());
        }
    };

    rsx! {
        div { class: "flex flex-col p-6 max-w-2xl mx-auto min-h-screen bg-white",

            // Top Bar
            div { class: "flex items-center mb-6",
                Link { to: Route::Home {}, class: "material-symbols-outlined text-gray-600 p-2 -ml-2 rounded-full hover:bg-gray-100 transition-colors", "arrow_back" }
                h2 { class: "flex-1 text-center text-lg font-semibold text-gray-900 pr-8", "录入新物品" } // pr-8 balances the back button width visually
            }

            // 1. Name Input
            div { class: "flex flex-col mb-6",
                label { class: "block text-sm font-medium text-gray-700 mb-2", "物品名称" }
                div { class: "flex items-center gap-3",
                    input {
                        r#type: "text",
                        class: "flex-1 bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all",
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                        placeholder: "例如：全麦面包"
                    }
                    button {
                        class: "p-3 bg-gray-100 rounded-xl text-gray-600 hover:bg-gray-200 transition-colors",
                        title: "扫码 (UI Only)",
                        span { class: "material-symbols-outlined", "qr_code_scanner" }
                    }
                }
            }

            // 2. Quick Chips
            div { class: "flex flex-wrap gap-2 mb-8",
                for (n, d, e) in quick_options {
                    button {
                        class: "px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-full text-sm font-medium transition-colors cursor-pointer active:scale-95 border-none",
                        onclick: move |_| apply_chip(n, d, e),
                        "{n}"
                    }
                }
            }

            // 3. Production Date / Entry Date
            div { class: "flex flex-col mb-6",
                label { class: "block text-sm font-medium text-gray-700 mb-2", "生产日期 / 入库日期" }
                input {
                    r#type: "date",
                    class: "w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all",
                    value: "{production_date_str}",
                    oninput: move |evt| production_date_str.set(evt.value())
                }
            }

            // 4. Quantity
            div { class: "flex flex-col mb-6",
                label { class: "block text-sm font-medium text-gray-700 mb-2", "数量" }
                input {
                    r#type: "number",
                    min: "1",
                    step: "1",
                    class: "w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all",
                    value: "{quantity_str}",
                    oninput: move |evt| quantity_str.set(evt.value())
                }
            }

            // 5. Expiry Date
            div { class: "flex flex-col mb-8",
                label { class: "block text-sm font-medium text-gray-700 mb-2", "过期日期" }
                // Duration Presets
                div { class: "flex flex-wrap gap-2 mb-3",
                    button { class: "px-3 py-1.5 bg-blue-50 text-blue-600 rounded-lg text-xs font-medium hover:bg-blue-100 transition-colors border-none cursor-pointer", onclick: move |_| add_days(3), "+3天" }
                    button { class: "px-3 py-1.5 bg-blue-50 text-blue-600 rounded-lg text-xs font-medium hover:bg-blue-100 transition-colors border-none cursor-pointer", onclick: move |_| add_days(7), "+7天" }
                    button { class: "px-3 py-1.5 bg-blue-50 text-blue-600 rounded-lg text-xs font-medium hover:bg-blue-100 transition-colors border-none cursor-pointer", onclick: move |_| add_days(15), "+15天" }
                    button { class: "px-3 py-1.5 bg-blue-50 text-blue-600 rounded-lg text-xs font-medium hover:bg-blue-100 transition-colors border-none cursor-pointer", onclick: move |_| add_days(30), "+30天" }
                }
                // Expiry Date Picker
                input {
                    r#type: "date",
                    class: "w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all",
                    value: "{expiry_date_str}",
                    oninput: move |evt| expiry_date_str.set(evt.value())
                }
            }

            div { class: "flex-1" } // Push button to bottom

            // 6. Submit
            button {
                class: "w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-4 rounded-xl shadow-lg shadow-blue-600/20 transition-all active:scale-95 text-lg mt-4",
                onclick: submit,
                "保 存"
            }
        }
    }
}
