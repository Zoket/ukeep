use crate::models::Item;
use chrono::{Duration, Local};

/// 生成模拟数据用于测试
pub fn generate_mock_data() -> Vec<Item> {
    let today = Local::now().date_naive();
    vec![
        Item::new("🥛全脂牛奶".into(), today - Duration::days(2)), // 已过期
        Item::new("🍞切片面包".into(), today + Duration::days(1)), // 临期 (红)
        Item::new("🍓草莓酸奶".into(), today + Duration::days(3)), // 警告 (黄)
        Item::new("🐟三文鱼".into(), today + Duration::days(2)),   // 警告 (黄)
        Item::new(
            "🥚鸡蛋 (12枚)".into(),
            today + Duration::days(10),
        ), // 安全
        Item::new("🍕冷冻披萨".into(), today + Duration::days(60)), // 安全
        Item::new("🍎苹果".into(), today + Duration::days(5)),     // 安全
    ]
}
