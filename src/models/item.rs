use chrono::Local;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_quantity() -> u32 {
    1
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Item {
    id: Uuid,
    name: String,
    emoji: String,
    expiry_date: NaiveDate,
    #[serde(default = "default_quantity")]
    quantity: u32,
}

impl Item {
    /// 创建新的 Item 实例
    pub fn new(name: String, emoji: String, expiry_date: NaiveDate) -> Self {
        Self::new_with_quantity(name, emoji, expiry_date, 1)
    }

    /// 创建带数量的 Item 实例
    pub fn new_with_quantity(
        name: String,
        emoji: String,
        expiry_date: NaiveDate,
        quantity: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            emoji,
            expiry_date,
            quantity: quantity.max(1),
        }
    }

    /// 获取 ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取表情符号
    pub fn emoji(&self) -> &str {
        &self.emoji
    }

    /// 获取过期日期
    pub fn expiry_date(&self) -> NaiveDate {
        self.expiry_date
    }

    /// 获取数量
    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    /// 消耗一个单位，返回是否需要移除整个条目
    pub fn consume_one(&mut self) -> bool {
        if self.quantity > 1 {
            self.quantity -= 1;
            false
        } else {
            true
        }
    }

    /// 计算剩余天数：负数表示已过期
    pub fn days_remaining(&self) -> i64 {
        let today = Local::now().date_naive();
        (self.expiry_date - today).num_days()
    }

    /// 获取状态颜色类名 (Tailwind CSS Class)
    pub fn status_class(&self) -> &'static str {
        let days = self.days_remaining();
        if days <= 1 {
            "bg-red-50 border-l-4 border-red-500 shadow-sm" // 🔴 过期 或 剩1天
        } else if days <= 3 {
            "bg-amber-50 border-l-4 border-amber-500 shadow-sm" // 🟡 3天内
        } else {
            "bg-white border border-gray-100 border-l-4 border-l-green-500 shadow-sm" // 🟢 安全
        }
    }

    /// 获取用于展示的时间文本
    pub fn display_deadline(&self) -> String {
        let days = self.days_remaining();
        if days < 0 {
            format!("已过期 {} 天", days.abs())
        } else if days == 0 {
            "今天到期".to_string()
        } else {
            format!("还剩 {} 天", days)
        }
    }
}
