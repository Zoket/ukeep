use crate::models::Item;
use crate::router::Route;
use crate::state::InventoryState;
use chrono::{Duration, Local, NaiveDate};
use dioxus::prelude::*;

/// 从物品名称中提取 emoji，如果名称以 emoji 开头则返回该 emoji，否则返回默认值
fn extract_emoji(name: &str) -> String {
    // 获取第一个字符
    if let Some(first_char) = name.chars().next() {
        // 检查是否是 emoji（简单判断：非 ASCII 字符）
        // 更准确的判断可以检查 Unicode 范围，但这里简化处理
        if !first_char.is_ascii() {
            return first_char.to_string();
        }
    }
    // 默认使用纸箱 emoji
    "📦".to_string()
}

#[component]
pub fn AddItem() -> Element {
    let mut inventory = use_context::<InventoryState>().0;
    let navigator = use_navigator();

    // Form State
    let mut name = use_signal(|| "".to_string());
    let mut production_date_str = use_signal(|| Local::now().format("%Y-%m-%d").to_string());
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
            let emoji = extract_emoji(&item_name);

            inventory.write().push(Item::new(
                item_name,
                emoji,
                parsed_date,
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
        div { class: "flex-col", style: "padding: 24px; max-width: 600px; margin: 0 auto; min-height: 100vh; background: white;",

            // Top Bar
            div { class: "flex-row", style: "margin-bottom: 20px;",
                Link { to: Route::Home {}, class: "material-symbols-outlined", style: "text-decoration: none; color: black;", "arrow_back" }
                h2 { style: "flex: 1; text-align: center; margin: 0;", "录入新物品" }
                div { style: "width: 24px;" } // spacer
            }

            // 1. Name Input
            div { class: "flex-col",
                label { "物品名称" }
                div { class: "flex-row",
                    input {
                        r#type: "text",
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                        placeholder: "例如：全麦面包"
                    }
                    button { style: "background:none; border:none;", title: "扫码 (UI Only)",
                        span { class: "material-symbols-outlined", "qr_code_scanner" }
                    }
                }
            }

            // 2. Quick Chips
            div { class: "flex-row", style: "flex-wrap: wrap; gap: 8px;",
                for (n, d, e) in quick_options {
                    span {
                        class: "chip",
                        onclick: move |_| apply_chip(n, d, e),
                        "{n}"
                    }
                }
            }

            div { style: "height: 16px;" } // Spacer

            // 3. Production Date / Entry Date
            div { class: "flex-col",
                label { "生产日期 / 入库日期" }
                input {
                    r#type: "date",
                    value: "{production_date_str}",
                    oninput: move |evt| production_date_str.set(evt.value())
                }
            }

            div { style: "height: 16px;" } // Spacer

            // 4. Expiry Date
            div { class: "flex-col",
                label { "过期日期" }
                // Duration Presets
                div { class: "flex-row", style: "justify-content: space-between;",
                    button { class: "chip", onclick: move |_| add_days(3), "+3天" }
                    button { class: "chip", onclick: move |_| add_days(7), "+7天" }
                    button { class: "chip", onclick: move |_| add_days(15), "+15天" }
                    button { class: "chip", onclick: move |_| add_days(30), "+30天" }
                }
                // Expiry Date Picker
                input {
                    r#type: "date",
                    value: "{expiry_date_str}",
                    oninput: move |evt| expiry_date_str.set(evt.value())
                }
            }

            div { style: "flex: 1;" } // Push button to bottom

            // 5. Submit
            button {
                class: "btn-primary",
                onclick: submit,
                "保 存"
            }
        }
    }
}
