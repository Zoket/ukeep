# ukeep 代码风格与约定

## Rust 代码风格
- **Edition**: Rust 2021
- **格式化**: 使用 `cargo fmt` (rustfmt)
- **Linting**: 使用 `cargo clippy`

## Clippy 特殊配置
项目配置了 `clippy.toml`，包含针对 Dioxus 框架的特殊规则：

### await-holding-invalid-types
禁止在 await 点持有以下类型：
- `generational_box::GenerationalRef` - 读取引用不应跨越 await
- `generational_box::GenerationalRefMut` - 写入引用不应跨越 await
- `dioxus_signals::Write` - 写入信号不应跨越 await

**原因**: 这些类型在 await 期间持有会导致借用冲突，阻止其他读写操作。

## 命名约定
- **结构体**: PascalCase (如 `Item`, `InventoryState`)
- **函数/方法**: snake_case (如 `days_remaining`, `generate_mock_data`)
- **常量**: SCREAMING_SNAKE_CASE (如 `CSS`)
- **组件**: PascalCase (如 `App`, `Home`, `ItemCard`, `AddItem`)

## Dioxus 组件约定
- 组件使用函数式定义
- 使用 `#[component]` 宏标注
- 使用 `rsx!` 宏构建 UI
- 状态管理使用 Dioxus signals

## 代码组织
- 数据模型放在 `src/models/` 目录
- 每个模型有独立文件 (如 `item.rs`)
- 使用 `mod.rs` 导出模块

## 依赖特性
- 明确启用需要的 feature flags
- 例如: chrono 启用 serde, uuid 启用 v4/fast-rng/js
