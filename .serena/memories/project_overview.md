# ukeep 项目概览

## 项目目的
ukeep 是一个库存管理应用，用于跟踪物品及其过期日期。用户可以添加物品、查看剩余天数，并根据过期状态进行分类管理。

## 技术栈
- **语言**: Rust (edition 2021)
- **UI 框架**: Dioxus 0.6.0 (支持 web/desktop/mobile 多平台)
- **样式**: Tailwind CSS
- **依赖库**:
  - chrono 0.4.42 (日期时间处理，启用 serde 特性)
  - uuid 1.19.0 (唯一标识符生成，启用 v4, fast-rng, js 特性)

## 项目结构
```
src/
├── main.rs                    # 应用入口 (26 行)
├── lib.rs                     # 库入口，导出所有模块
├── components/                # UI 组件
│   ├── mod.rs
│   └── item_card.rs          # ItemCard 组件
├── pages/                     # 页面组件
│   ├── mod.rs
│   ├── home.rs               # Home 页面 (库存列表)
│   └── add_item.rs           # AddItem 页面 (添加物品)
├── models/                    # 数据模型
│   ├── mod.rs
│   └── item.rs               # Item 数据结构
├── router.rs                  # 路由配置
├── state.rs                   # 全局状态管理 (InventoryState)
└── utils.rs                   # 工具函数 (generate_mock_data)
```

## 核心数据模型
- **Item**: 物品结构体
  - 私有字段: id, name, emoji, expiry_date
  - 构造函数: `new(name, emoji, expiry_date)`
  - Getter 方法: `id()`, `name()`, `emoji()`, `expiry_date()`
  - 业务方法: `days_remaining()`, `status_class()`, `display_deadline()`

## 模块说明
- **components**: 可复用的 UI 组件
- **pages**: 页面级组件，对应路由
- **models**: 数据模型和业务逻辑
- **router**: 路由配置
- **state**: 全局状态管理
- **utils**: 工具函数和辅助方法
