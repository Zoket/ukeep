# ukeep 项目常用命令

## 开发命令

### 启动开发服务器
```bash
dx serve
```
默认平台启动应用（通常是 web）

### 指定平台启动
```bash
dx serve --platform desktop   # 桌面应用
dx serve --platform web        # Web 应用
dx serve --platform mobile     # 移动应用
```

### Tailwind CSS 编译
```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
```
监听 Tailwind CSS 变化并自动编译到 assets 目录

## 构建命令

### 构建项目
```bash
cargo build
```

### 发布构建
```bash
cargo build --release
```

## 代码质量命令

### 运行 Clippy (Linting)
```bash
cargo clippy
```

### 格式化代码
```bash
cargo fmt
```

### 运行测试
```bash
cargo test
```

## 依赖管理

### 更新依赖
```bash
cargo update
```

### 检查依赖
```bash
cargo tree
```

## 系统工具 (macOS/Darwin)
- `ls` - 列出文件
- `cd` - 切换目录
- `grep` - 搜索文本
- `find` - 查找文件
- `git` - 版本控制
