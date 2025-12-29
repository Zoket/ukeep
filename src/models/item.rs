use chrono::Local;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Item {
    id: Uuid,
    name: String,
    emoji: String,
    expiry_date: NaiveDate,
}

impl Item {
    /// 创建新的 Item 实例
    pub fn new(name: String, emoji: String, expiry_date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            emoji,
            expiry_date,
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

    /// 计算剩余天数：负数表示已过期
    pub fn days_remaining(&self) -> i64 {
        let today = Local::now().date_naive();
        (self.expiry_date - today).num_days()
    }

    /// 获取状态颜色类名 (CSS Class)
    pub fn status_class(&self) -> &'static str {
        let days = self.days_remaining();
        if days <= 1 {
            "status-error" // 🔴 过期 或 剩1天
        } else if days <= 3 {
            "status-warning" // 🟡 3天内
        } else {
            "status-safe" // 🟢 安全
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
