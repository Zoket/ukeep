use crate::models::Item;
use dioxus::events::PointerEvent;
use dioxus::prelude::*;
use uuid::Uuid;

// 滑动阈值常量（像素）
const SWIPE_THRESHOLD: f64 = 80.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SwipeAction {
    Consume, // 吃掉（右滑）
    Waste,   // 扔掉（左滑）
}

#[component]
pub fn ItemCard(
    item: Item,
    on_consume: EventHandler<Uuid>,
    on_waste: EventHandler<Uuid>,
) -> Element {
    let item_id = item.id();

    // 滑动状态
    let mut drag_x = use_signal(|| 0.0_f64);
    let mut start_x = use_signal(|| 0.0_f64);
    let mut is_dragging = use_signal(|| false);
    let mut pending_action = use_signal(|| None::<SwipeAction>);

    // 开始拖动
    let on_pointer_down = move |evt: PointerEvent| {
        is_dragging.set(true);
        start_x.set(evt.client_coordinates().x);
        drag_x.set(0.0);
        pending_action.set(None);
    };

    // 拖动中
    let on_pointer_move = move |evt: PointerEvent| {
        if !*is_dragging.read() {
            return;
        }
        let delta = evt.client_coordinates().x - *start_x.read();
        drag_x.set(delta);

        // 根据滑动方向设置待执行的动作
        if delta >= SWIPE_THRESHOLD {
            // 右滑 -> 吃掉
            pending_action.set(Some(SwipeAction::Consume));
        } else if delta <= -SWIPE_THRESHOLD {
            // 左滑 -> 扔掉
            pending_action.set(Some(SwipeAction::Waste));
        } else {
            pending_action.set(None);
        }
    };

    // 结束拖动
    let on_pointer_up = move |_| {
        if let Some(action) = *pending_action.read() {
            match action {
                SwipeAction::Consume => on_consume.call(item_id),
                SwipeAction::Waste => on_waste.call(item_id),
            }
        }
        drag_x.set(0.0);
        is_dragging.set(false);
        pending_action.set(None);
    };

    // 指针离开卡片区域 - 取消操作
    let on_pointer_leave = move |_| {
        if !*is_dragging.read() {
            return;
        }
        // 取消操作，重置状态
        drag_x.set(0.0);
        is_dragging.set(false);
        pending_action.set(None);
    };

    let drag = *drag_x.read();
    let reveal = (drag.abs() / SWIPE_THRESHOLD).min(1.0); // 0-1 之间的透明度
    let transition = if *is_dragging.read() {
        "none"
    } else {
        "transform 200ms ease-out"
    };

    rsx! {
        div {
            class: "card flex-row {item.status_class()}",
            style: "position: relative; justify-content: space-between; overflow: hidden; touch-action: pan-y;",

            // 背景提示层 - 左侧（吃掉，右滑时显示）
            if drag > 0.0 {
                div {
                    style: format!(
                        "position: absolute; left: 0; top: 0; bottom: 0; width: 100px; display: flex; align-items: center; justify-content: flex-start; padding-left: 16px; color: var(--color-safe); opacity: {reveal}; pointer-events: none; background: linear-gradient(90deg, rgba(72,199,116,0.2), transparent);"
                    ),
                    span { class: "material-symbols-outlined", style: "font-size: 1.5rem;", "restaurant" }
                    span { style: "margin-left: 8px; font-weight: 600; font-size: 0.9rem;", "吃掉" }
                }
            }

            // 背景提示层 - 右侧（扔掉，左滑时显示）
            if drag < 0.0 {
                div {
                    style: format!(
                        "position: absolute; right: 0; top: 0; bottom: 0; width: 100px; display: flex; align-items: center; justify-content: flex-end; padding-right: 16px; color: var(--color-error); opacity: {reveal}; pointer-events: none; background: linear-gradient(270deg, rgba(244,67,54,0.2), transparent);"
                    ),
                    span { style: "margin-right: 8px; font-weight: 600; font-size: 0.9rem;", "扔掉" }
                    span { class: "material-symbols-outlined", style: "font-size: 1.5rem;", "delete" }
                }
            }

            // 可拖动的卡片内容
            div {
                class: "flex-row",
                style: format!(
                    "width: 100%; align-items: stretch; transform: translateX({drag}px); transition: {transition}; cursor: grab; position: relative; z-index: 1;"
                ),
                onpointerdown: on_pointer_down,
                onpointermove: on_pointer_move,
                onpointerup: on_pointer_up,
                onpointercancel: on_pointer_up,
                onpointerleave: on_pointer_leave,

                // 左侧信息
                div { class: "flex-row", style: "flex: 1; padding: 12px;",
                    span { style: "font-size: 2rem;", "{item.emoji()}" }
                    div { class: "flex-col", style: "gap: 2px; margin-left: 12px;",
                        span { style: "font-size: 1.1rem; font-weight: 500;", "{item.name()}" }
                        span { style: "font-size: 0.8rem; opacity: 0.7;", "{item.expiry_date().format(\"%Y-%m-%d\")}" }
                    }
                }

                // 右侧信息
                div { class: "flex-col", style: "align-items: flex-end; justify-content: center; padding: 12px;",
                    span { style: "font-size: 1.2rem; font-weight: bold;", "{item.display_deadline()}" }
                }
            }

            // 可访问性：隐藏的按钮供键盘和屏幕阅读器使用
            button {
                onclick: move |_| on_consume.call(item_id),
                style: "position: absolute; left: -9999px;",
                "aria-label": "吃掉了 (Consumed)",
                "吃掉"
            }
            button {
                onclick: move |_| on_waste.call(item_id),
                style: "position: absolute; left: -9999px;",
                "aria-label": "扔掉了 (Wasted)",
                "扔掉"
            }
        }
    }
}
