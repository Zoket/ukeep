# Docker 部署指南

本文档说明如何使用 Docker 将 ukeep 应用部署到 VPS。

## 文件说明

- `Dockerfile`: 多阶段构建配置
  - Stage 1: 使用 Node.js 编译 Tailwind CSS
  - Stage 2: 使用 Rust 构建 WASM 应用(使用 `dx bundle` 进行生产优化)
  - Stage 3: 使用 Nginx 提供静态文件服务
  - 包含 binaryen 工具集(wasm-strip, wasm-opt)用于 WASM 优化

- `docker-compose.yml`: Docker Compose 配置文件
- `nginx.conf`: Nginx 服务器配置(支持 SPA 路由)
- `.dockerignore`: Docker 构建时忽略的文件

## 本地构建和测试

### 快速开始(推荐)

使用提供的部署脚本:

```bash
# 一键构建并运行
./docker-deploy.sh

# 或者分步执行
./docker-deploy.sh build    # 仅构建
./docker-deploy.sh run      # 仅运行
./docker-deploy.sh logs     # 查看日志
./docker-deploy.sh restart  # 重启
./docker-deploy.sh clean    # 清理
```

### 手动构建

### 1. 构建镜像

```bash
docker build -t ukeep:latest .
```

### 2. 运行容器

```bash
docker run -d -p 8080:80 --name ukeep ukeep:latest
```

访问 http://localhost:8080 查看应用。

### 3. 使用 Docker Compose

```bash
docker-compose up -d
```

## VPS 部署

### 方法 1: 直接在 VPS 上构建

1. 将代码上传到 VPS:
```bash
rsync -avz --exclude 'target' --exclude 'node_modules' ./ user@your-vps:/path/to/ukeep/
```

2. SSH 登录 VPS 并构建:
```bash
ssh user@your-vps
cd /path/to/ukeep
docker-compose up -d
```

### 方法 2: 本地构建后推送

1. 本地构建镜像:
```bash
docker build -t your-registry/ukeep:latest .
```

2. 推送到镜像仓库:
```bash
docker push your-registry/ukeep:latest
```

3. 在 VPS 上拉取并运行:
```bash
docker pull your-registry/ukeep:latest
docker run -d -p 80:80 --name ukeep your-registry/ukeep:latest
```

## 配置说明

### 端口映射

默认映射到 8080 端口,可以在 `docker-compose.yml` 中修改:

```yaml
ports:
  - "80:80"  # 直接使用 80 端口
```

### Nginx 配置

`nginx.conf` 包含以下特性:
- SPA 路由支持(所有路由回退到 index.html)
- WASM MIME 类型配置
- PWA 支持:
  - manifest.json 正确的 MIME 类型(application/manifest+json)
  - Service Worker 不缓存策略(确保更新及时生效)
- Gzip/Brotli 压缩
- 静态资源缓存(1年)
- 安全头配置
- 健康检查端点(/health)

### 健康检查

容器包含健康检查配置,可以通过以下方式查看:

```bash
docker ps  # 查看 STATUS 列的健康状态
```

## 故障排查

### 查看日志

```bash
docker logs ukeep
```

### 进入容器

```bash
docker exec -it ukeep sh
```

### 检查构建产物

```bash
docker exec -it ukeep ls -la /usr/share/nginx/html
```

## 性能优化

镜像已经包含以下优化:
- 多阶段构建(最终镜像仅包含必要文件)
- Rust release 模式优化(opt-level="z", LTO)
- Gzip/Brotli 压缩
- 静态资源缓存
- 最小化的 Alpine Linux 基础镜像

## 注意事项

1. 首次构建可能需要较长时间(下载依赖、编译 Rust)
2. 确保 VPS 有足够的内存(建议至少 2GB)
3. 如果构建失败,检查 Docker 日志获取详细错误信息

## 常见问题

### 1. 构建失败: "Failed to strip binary"

**原因**: Docker 镜像缺少 `wasm-strip` 工具

**解决方案**: 已在 Dockerfile 中添加 `binaryen` 包，包含 wasm-strip 和 wasm-opt 工具

详细说明请查看 [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)

### 2. 构建时间过长

**优化建议**:
- 使用 Docker 构建缓存
- 使用 `cargo-binstall` 加速 CLI 安装(已在 Dockerfile 中实现)
- 考虑使用 Docker BuildKit: `DOCKER_BUILDKIT=1 docker build .`

### 3. 镜像体积过大

**当前优化**:
- 多阶段构建(最终镜像仅包含静态文件)
- 使用 Alpine Linux 基础镜像
- `dx bundle` 自动优化 WASM 文件

**预期镜像大小**: 约 30-50MB(nginx:alpine + 优化后的静态文件)
