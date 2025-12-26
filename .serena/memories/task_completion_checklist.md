# 任务完成检查清单

当完成一个开发任务后，应执行以下步骤：

## 1. 代码质量检查

### 格式化代码
```bash
cargo fmt
```
确保代码符合 Rust 标准格式

### 运行 Clippy
```bash
cargo clippy
```
检查并修复所有 clippy 警告，特别注意：
- await-holding-invalid-types 规则
- Dioxus signals 的正确使用

## 2. 编译检查

### 开发构建
```bash
cargo build
```
确保代码可以成功编译

### 检查所有平台特性
```bash
cargo build --features web
cargo build --features desktop
cargo build --features mobile
```

## 3. 测试

### 运行测试套件
```bash
cargo test
```

## 4. 前端资源

### 重新编译 Tailwind CSS (如果修改了样式)
```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css
```

## 5. 运行验证

### 启动开发服务器测试
```bash
dx serve
```
手动验证功能是否正常工作

## 6. 代码审查要点
- 确保没有未使用的导入
- 检查错误处理是否完善
- 验证 Dioxus signals 没有跨 await 持有
- 确认 UI 组件响应式正常
- 检查是否有潜在的性能问题

## 7. 文档
- 如果添加了新的公共 API，确保添加文档注释
- 更新 README.md (如果需要)
