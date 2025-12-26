use crate::models::Item;
use dioxus::prelude::*;

/// 全局状态 Context Key
#[derive(Clone, Copy)]
pub struct InventoryState(pub Signal<Vec<Item>>);
