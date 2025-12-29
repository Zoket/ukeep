use crate::models::Item;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

const STORAGE_KEY: &str = "ukeep_inventory";

/// 从 LocalStorage 加载库存数据
/// 如果读取失败或数据损坏，返回空数组并清空存储
pub fn load_inventory() -> Vec<Item> {
    match LocalStorage::get::<Vec<Item>>(STORAGE_KEY) {
        Ok(items) => items,
        Err(e) => {
            // 数据损坏或不存在，清空存储
            log::warn!("Failed to load inventory: {:?}", e);
            let _ = LocalStorage::delete(STORAGE_KEY);
            Vec::new()
        }
    }
}

/// 保存库存数据到 LocalStorage
pub fn save_inventory(items: &Vec<Item>) {
    if let Err(e) = LocalStorage::set(STORAGE_KEY, items) {
        log::error!("Failed to save inventory: {:?}", e);
    }
}

/// 清空所有存储数据
pub fn clear_storage() {
    let _ = LocalStorage::delete(STORAGE_KEY);
}

/// 导出数据为 JSON 文件并触发下载
pub fn export_data(items: &Vec<Item>) -> Result<(), String> {
    // 序列化为 JSON
    let json_str = serde_json::to_string_pretty(items)
        .map_err(|e| format!("序列化失败: {}", e))?;

    // 创建 Blob
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(&json_str));

    let blob_options = BlobPropertyBag::new();
    blob_options.set_type("application/json");

    let blob = Blob::new_with_str_sequence_and_options(&array, &blob_options)
        .map_err(|_| "创建 Blob 失败".to_string())?;

    // 创建下载链接
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "创建 URL 失败".to_string())?;

    // 获取 window 和 document
    let window = web_sys::window().ok_or("无法获取 window")?;
    let document = window.document().ok_or("无法获取 document")?;

    // 创建临时 <a> 元素
    let anchor = document
        .create_element("a")
        .map_err(|_| "创建元素失败")?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "类型转换失败")?;

    // 设置下载属性
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("ukeep_backup_{}.json", timestamp);

    anchor.set_href(&url);
    anchor.set_download(&filename);
    anchor.click();

    // 清理 URL
    Url::revoke_object_url(&url).map_err(|_| "清理 URL 失败")?;

    Ok(())
}

/// 从 JSON 字符串导入数据
pub fn import_data_from_json(json_str: &str) -> Result<Vec<Item>, String> {
    serde_json::from_str::<Vec<Item>>(json_str)
        .map_err(|e| format!("数据格式错误: {}", e))
}
