use crate::components::ItemCard;
use crate::router::Route;
use crate::state::InventoryState;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn Home() -> Element {
    let mut inventory = use_context::<InventoryState>().0;

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

    rsx! {
        div { class: "flex-col", style: "padding: 16px; max-width: 600px; margin: 0 auto;",
            // --- Header ---
            header {
                h1 { "我的冰箱 🧊" }
                span { class: "subtitle",
                    if urgent_count > 0 {
                        "⚠️ 有 {urgent_count} 个物品需要尽快处理"
                    } else {
                        "👏 一切看起来都很新鲜"
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
