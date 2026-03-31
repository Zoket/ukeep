use crate::models::Item;
use dioxus::events::PointerEvent;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use uuid::Uuid;
use wasm_bindgen::JsCast;

// ItemCard 组件：单个物品的“卡片”，支持左右滑动手势
// - 向右滑：代表“吃掉了”，数量>1时会弹出数量选择菜单
// - 向左滑：代表“扔掉了”
// - 松手时根据滑动方向和位置触发回调

// 触发左右滑动操作的最小水平位移（像素）
const SWIPE_THRESHOLD: f64 = 80.0;
// 下拉菜单中单个选项的高度（像素）
const DROPDOWN_ITEM_H: f64 = 48.0;
// 下拉菜单最小宽度（像素）
const DROPDOWN_W: f64 = 72.0;
// 菜单相对锚点的水平间距（像素）
const DROPDOWN_GAP: f64 = 12.0;
// 菜单距离视口边缘的最小留白（像素）
const VIEWPORT_PAD: f64 = 12.0;
// “划出屏幕”判定的边缘留白（像素）
const OFFSCREEN_MARGIN: f64 = 8.0;
// 菜单中最多显示的离散数量选项个数
const MAX_VISIBLE_ITEMS: u32 = 10;

// 滑动行为的枚举：向右消费 / 向左丢弃
#[derive(Clone, Copy, PartialEq, Eq)]
enum SwipeAction {
    // 吃掉：向右滑动
    Consume,
    // 扔掉：向左滑动
    Waste,
}

// 从浏览器 window 中获取当前视口宽高
fn viewport_size() -> (f64, f64) {
    web_sys::window()
        .map(|w| {
            // inner_width / inner_height 返回 JsValue，这里尽量转成 f64，失败就用默认值
            let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(390.0);
            let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(844.0);
            (width, height)
        })
        // 如果连 window 都拿不到（极端情况），使用一个常见移动端视口默认值
        .unwrap_or((390.0, 844.0))
}

// 根据物品数量构建“可选择的消费数量”步骤，例如 [1,2,3,5,10, quantity]
fn build_steps(quantity: u32) -> Vec<u32> {
    // 数量较少时，直接生成 1..=quantity 的线性列表
    if quantity <= MAX_VISIBLE_ITEMS {
        (1..=quantity).collect()
    } else {
        // 数量很多时，用几个常用步长 + “全部”
        let mut steps: Vec<u32> = [1, 2, 3, 5, 10]
            .iter()
            .copied()
            // 只保留小于总数量的项
            .filter(|&v| v < quantity)
            .collect();
        // 再把“全部”放在最后一个选项
        steps.push(quantity);
        steps
    }
}

// 根据锚点位置和选项个数计算菜单外框的 left / top / height
fn menu_frame(anchor_x: f64, anchor_y: f64, count: usize) -> (f64, f64, f64) {
    let (vw, vh) = viewport_size();
    // 菜单总高度 = 单项高度 * 项数
    let h = DROPDOWN_ITEM_H * count as f64;
    // 限制菜单左侧坐标不能超出视口左右边界
    let max_left = (vw - DROPDOWN_W - VIEWPORT_PAD).max(VIEWPORT_PAD);
    // 限制菜单顶部坐标不能超出视口上下边界
    let max_top = (vh - h - VIEWPORT_PAD).max(VIEWPORT_PAD);
    // 菜单左侧在锚点右侧稍微偏移 DROPDOWN_GAP
    let left = (anchor_x + DROPDOWN_GAP).clamp(VIEWPORT_PAD, max_left);
    // 菜单竖直方向以锚点为中心稍微往上偏移
    let top = (anchor_y - DROPDOWN_ITEM_H / 2.0).clamp(VIEWPORT_PAD, max_top);
    (left, top, h)
}

// 在数量菜单中，根据指针位置 (px, py) 命中具体的数量选项
fn hit_test(px: f64, py: f64, ax: f64, ay: f64, steps: &[u32]) -> Option<u32> {
    let (left, top, h) = menu_frame(ax, ay, steps.len());
    // 水平方向使用更宽松的命中区域（min-width + padding），提高命中容错
    let effective_w = DROPDOWN_W + 32.0;
    // 不在菜单矩形区域内，直接返回 None
    if px < left || px > left + effective_w || py < top || py > top + h {
        return None;
    }
    // 纵向通过 y 偏移所在的“行号”来推算 index
    let idx = ((py - top) / DROPDOWN_ITEM_H).floor() as usize;
    // 从 steps 中取出对应数量
    steps.get(idx).copied()
}

// 判断指针是否已经接近屏幕右边缘（用于“快速吃掉 1 个”）
fn is_offscreen(px: f64) -> bool {
    let (vw, _) = viewport_size();
    px >= vw - OFFSCREEN_MARGIN
}

/// 单个库存条目的卡片组件，支持左右滑动与数量菜单
/// - 右滑：触发 `on_consume`，数量>1 时可选择消费数量
/// - 左滑：触发 `on_waste`，表示丢弃该条目
#[component]
pub fn ItemCard(
    item: Item,
    // 吃掉回调：携带物品 ID 和要消费的数量
    on_consume: EventHandler<(Uuid, u32)>,
    // 扔掉回调：只需要物品 ID
    on_waste: EventHandler<Uuid>,
) -> Element {
    // 预先取出常用字段，避免多次方法调用
    let item_id = item.id();
    let quantity = item.quantity();

    // 当前水平拖拽偏移量（相对于按下时的起点，单位：像素）
    let mut drag_x = use_signal(|| 0.0_f64);
    // PointerDown 时记录的起始 X 坐标，用于后续计算 delta
    let mut start_x = use_signal(|| 0.0_f64);
    // 当前是否处于拖拽中
    let mut is_dragging = use_signal(|| false);
    // 当前推断出的滑动动作（向左/向右），用于 PointerUp 时决定触发哪种行为
    let mut pending_action = use_signal(|| None::<SwipeAction>);

    // 当前正在跟踪的 Pointer 的 ID，用来忽略其它手指/设备的事件
    let mut pointer_id = use_signal(|| None::<i32>);
    // 被 set_pointer_capture 捕获事件的元素，用于结束时释放捕获
    let mut captured_el = use_signal(|| None::<web_sys::Element>);
    // 数量菜单的锚点坐标（一般是第一次右滑达到阈值时的指针位置）
    let mut menu_anchor = use_signal(|| None::<(f64, f64)>);
    // 当前 hover 命中的数量（用于高亮菜单项）
    let mut hovered = use_signal(|| None::<u32>);

    // 根据物品数量预先计算好可选的消费数量列表（memo 避免重复计算）
    let steps = use_memo(move || build_steps(quantity));

    // 重置所有与拖拽和菜单相关的状态
    let mut reset = move || {
        // 如果之前捕获过 Pointer，需要先释放 capture
        if let (Some(el), Some(pid)) = (captured_el.read().clone(), *pointer_id.read()) {
            let _ = el.release_pointer_capture(pid);
        }
        // 清空 capture 相关状态
        captured_el.set(None);
        pointer_id.set(None);
        // 将拖拽相关状态恢复到初始值
        drag_x.set(0.0);
        is_dragging.set(false);
        pending_action.set(None);
        // 菜单与 hover 状态也一并清掉
        menu_anchor.set(None);
        hovered.set(None);
    };

    // Pointer 按下：开始一轮新的拖拽序列
    let on_pointer_down = move |evt: PointerEvent| {
        if *is_dragging.read() {
            return;
        }
        // 阻止浏览器原生拖拽（emoji 等内容会触发），否则后续 pointermove 被抑制
        evt.prevent_default();
        let raw: web_sys::PointerEvent = evt.as_web_event();
        let cx = evt.client_coordinates().x;

        is_dragging.set(true);
        start_x.set(cx);
        drag_x.set(0.0);
        pending_action.set(None);
        menu_anchor.set(None);
        hovered.set(None);
        pointer_id.set(Some(raw.pointer_id()));
        captured_el.set(None);

        // 使用 target()（实际点击的子元素）而非 current_target()（事件委托根节点），
        // 确保后续 pointer 事件从该子元素冒泡经过卡片 div，Dioxus 才能正确路由到处理器
        if let Some(el) = raw.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = el.set_pointer_capture(raw.pointer_id());
            captured_el.set(Some(el));
        }
    };

    // Pointer 移动：更新拖拽偏移，并根据方向/距离更新 pending_action 和菜单状态
    let on_pointer_move = move |evt: PointerEvent| {
        // 如果当前不在拖拽中，忽略 move 事件
        if !*is_dragging.read() {
            return;
        }
        let raw: web_sys::PointerEvent = evt.as_web_event();
        // 只处理当前记录的 pointer（避免多指场景的干扰）
        if Some(raw.pointer_id()) != *pointer_id.read() {
            return;
        }

        // 当前指针的坐标
        let cx = evt.client_coordinates().x;
        let cy = evt.client_coordinates().y;
        // 水平位移 = 当前 X - 起点 X
        let delta = cx - *start_x.read();
        // 保存最新的拖拽偏移，用于 UI 展示
        drag_x.set(delta);

        // 当前是否已经激活过数量菜单（锚点是否存在）
        let menu_active = menu_anchor.read().is_some();

        // 向右滑动距离达到阈值：进入“吃掉”模式
        if delta >= SWIPE_THRESHOLD {
            pending_action.set(Some(SwipeAction::Consume));
            // 数量>1时才显示数量菜单
            if quantity > 1 {
                // 如果还没有锚点，则用当前指针位置作为锚点
                let anchor = (*menu_anchor.read()).unwrap_or((cx, cy));
                if !menu_active {
                    menu_anchor.set(Some(anchor));
                }
                // 根据指针位置在菜单中做 hit-test，更新 hover 项
                hovered.set(hit_test(cx, cy, anchor.0, anchor.1, &steps.read()));
            }
        // 已经激活菜单，但手指又往回滑：菜单继续显示，只更新 hover 项
        } else if menu_active {
            // 菜单已激活，即使手指回退也保持菜单显示并继续 hit-test
            pending_action.set(Some(SwipeAction::Consume));
            let anchor = (*menu_anchor.read()).unwrap();
            hovered.set(hit_test(cx, cy, anchor.0, anchor.1, &steps.read()));
        // 向左滑动达到阈值：进入“扔掉”模式
        } else if delta <= -SWIPE_THRESHOLD {
            pending_action.set(Some(SwipeAction::Waste));
        // 滑动距离尚未达到任何一侧阈值：还没有明确动作
        } else {
            pending_action.set(None);
        }
    };

    // Pointer 松开：根据 pending_action 和位置信息真正触发回调
    let on_pointer_up = move |evt: PointerEvent| {
        // 如果当前不在拖拽中，忽略
        if !*is_dragging.read() {
            return;
        }
        let raw: web_sys::PointerEvent = evt.as_web_event();
        // 只处理之前记录的 pointer
        if Some(raw.pointer_id()) != *pointer_id.read() {
            return;
        }

        // 松手时指针的最终位置
        let cx = evt.client_coordinates().x;
        let cy = evt.client_coordinates().y;

        match *pending_action.read() {
            // 右滑 & 数量>1：优先根据菜单命中数量决定消费多少
            Some(SwipeAction::Consume) if quantity > 1 => {
                if let Some(anchor) = *menu_anchor.read() {
                    if let Some(count) = hit_test(cx, cy, anchor.0, anchor.1, &steps.read()) {
                        // 命中了某个数量选项
                        on_consume.call((item_id, count));
                    } else if is_offscreen(cx) {
                        // 没有命中菜单，但滑到屏幕外：快速默认吃掉 1 个
                        on_consume.call((item_id, 1));
                    }
                // 菜单还没出现，但已经划出屏幕：也按吃掉 1 个处理
                } else if is_offscreen(cx) {
                    on_consume.call((item_id, 1));
                }
            }
            // 右滑 & 数量==1：直接吃掉 1 个
            Some(SwipeAction::Consume) => on_consume.call((item_id, 1)),
            // 左滑：扔掉整个条目
            Some(SwipeAction::Waste) => on_waste.call(item_id),
            // 没有明确动作：什么都不做，只恢复位置
            None => {}
        }

        // 无论是否触发动作，最后都要重置状态
        reset();
    };

    // Pointer 被取消（例如系统打断）：安全地结束当前拖拽
    let on_pointer_cancel = move |evt: PointerEvent| {
        if !*is_dragging.read() {
            return;
        }
        let raw: web_sys::PointerEvent = evt.as_web_event();
        if Some(raw.pointer_id()) != *pointer_id.read() {
            return;
        }
        reset();
    };

    // Pointer 离开元素范围：如果当前元素没有 pointer capture，则视为结束拖拽
    let on_pointer_leave = move |_: PointerEvent| {
        // 如果已经捕获了 pointer，就说明仍可以在元素外收到事件，不需要 reset
        if !*is_dragging.read() || captured_el.read().is_some() {
            return;
        }
        reset();
    };

    // 当前拖拽偏移量，用于平移前景卡片 & 决定背景显示
    let drag = *drag_x.read();
    // 背景提示的显隐程度，0..=1，根据拖拽距离线性变化
    let reveal = (drag.abs() / SWIPE_THRESHOLD).min(1.0);
    // 拖拽中关闭过渡动画，松手后恢复过渡使卡片平滑回弹
    let transition = if *is_dragging.read() { "none" } else { "transform 200ms ease-out" };

    // 是否需要显示数量菜单（数量>1 且锚点已存在）
    let show_menu = quantity > 1 && menu_anchor.read().is_some();
    // 预先根据锚点计算菜单外框位置和高度
    let menu_data = if show_menu {
        (*menu_anchor.read()).map(|(ax, ay)| menu_frame(ax, ay, steps.read().len()))
    } else {
        None
    };
    // 当前 hover 的数量值，用于控制菜单项高亮
    let hovered_val = *hovered.read();

    rsx! {
        // 外层容器：控制卡片整体背景（状态色）、圆角、间距等
        div {
            class: "relative flex items-center justify-between overflow-hidden touch-pan-y rounded-xl mb-3 select-none {item.status_class()}",

            // 右滑背景：左侧的“吃掉了”提示区域
            if drag > 0.0 {
                div {
                    class: "absolute left-0 top-0 bottom-0 w-24 flex items-center justify-start pl-4 text-green-600 bg-gradient-to-r from-green-100/50 to-transparent",
                    style: "opacity: {reveal}; pointer-events: none;",
                    span { class: "material-symbols-outlined text-2xl", "restaurant" }
                    span { class: "ml-2 font-semibold text-sm", "吃掉了" }
                }
            }

            // 左滑背景：右侧的“扔掉了”提示区域
            if drag < 0.0 {
                div {
                    class: "absolute right-0 top-0 bottom-0 w-24 flex items-center justify-end pr-4 text-red-600 bg-gradient-to-l from-red-100/50 to-transparent",
                    style: "opacity: {reveal}; pointer-events: none;",
                    span { class: "mr-2 font-semibold text-sm", "扔掉了" }
                    span { class: "material-symbols-outlined text-2xl", "delete" }
                }
            }

            // 数量选择菜单（右滑且数量>1 时出现），使用 fixed 以避免受列表滚动影响
            if let Some((ml, mt, mh)) = menu_data {
                div {
                    class: "fixed z-50 overflow-hidden rounded-2xl bg-white/95 shadow-xl ring-1 ring-green-200 backdrop-blur-sm",
                    style: "left:{ml}px;top:{mt}px;min-width:{DROPDOWN_W}px;width:max-content;height:{mh}px;pointer-events:none;padding-inline:8px;",
                    // 遍历 steps，渲染每一个数量选项
                    for &count in steps.read().iter() {
                        div {
                            key: "{item_id}-menu-{count}",
                            class: if hovered_val == Some(count) {
                                // hover 项采用绿色高亮
                                "flex items-center justify-center bg-green-500 text-white text-sm font-semibold"
                            } else {
                                "flex items-center justify-center border-b border-gray-100 text-gray-700 text-sm font-medium last:border-b-0"
                            },
                            style: "height:{DROPDOWN_ITEM_H}px;",
                            // 当数量过大时，最后一个选项显示“全部(xxx)”
                            if count == quantity && quantity > MAX_VISIBLE_ITEMS {
                                "全部({count})"
                            } else {
                                "{count}"
                            }
                        }
                    }
                }
            }

            // 前景卡片：真正可拖拽的内容区域
            div {
                class: "flex items-stretch w-full z-10 cursor-grab active:cursor-grabbing bg-transparent",
                // 根据 drag 动态平移卡片，同时控制回弹动画
                style: format!("transform: translateX({drag}px); transition: {transition};"),
                // 绑定 pointer 相关事件
                onpointerdown: on_pointer_down,
                onpointermove: on_pointer_move,
                onpointerup: on_pointer_up,
                onpointercancel: on_pointer_cancel,
                onpointerleave: on_pointer_leave,

                // 左侧：emoji + 名称 + 数量
                div { class: "flex-1 flex items-center p-4",
                    span { class: "text-3xl mr-4", "{item.emoji()}" }
                    div { class: "flex flex-col gap-0.5",
                        div { class: "flex items-center gap-2",
                            span { class: "text-lg font-medium text-gray-900", "{item.name()}" }
                            span { class: "inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-600",
                                "x{quantity}"
                            }
                        }
                        // 展示到期日期（yyyy-mm-dd）
                        span { class: "text-xs text-gray-500", "{item.expiry_date().format(\"%Y-%m-%d\")}" }
                    }
                }

                // 右侧：距离过期的描述（例如“还剩 X 天”）
                div { class: "flex flex-col items-end justify-center p-4",
                    span { class: "text-sm font-bold text-gray-700", "{item.display_deadline()}" }
                }
            }

            // 为无障碍/键盘用户提供的隐藏按钮，等价于左右滑动
            button {
                onclick: move |_| on_consume.call((item_id, 1)),
                class: "sr-only",
                "aria-label": "吃掉了 (Consumed)",
                "吃掉了"
            }
            button {
                onclick: move |_| on_waste.call(item_id),
                class: "sr-only",
                "aria-label": "扔掉了 (Wasted)",
                "扔掉了"
            }
        }
    }
}
