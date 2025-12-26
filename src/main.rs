#![allow(non_snake_case)]

use dioxus::prelude::*;
use ukeep::router::Route;
use ukeep::state::InventoryState;
use ukeep::utils::generate_mock_data;

static CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    // 初始化全局状态
    use_context_provider(|| InventoryState(Signal::new(generate_mock_data())));

    rsx! {
        // 注入全局样式
        document::Stylesheet { href: CSS }
        // 注入 Material Symbols 字体 (用于图标)
        link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0" }

        Router::<Route> {}
    }
}
