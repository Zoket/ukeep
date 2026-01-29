# Docker 部署指南

本文档说明如何使用 Docker 将 ukeep 应用部署到 VPS。

## 文件说明

| 文件 | 说明 |
|------|------|
| `Dockerfile` | 多阶段构建 (Tailwind → Rust/WASM → Nginx) |
| `Dockerfile.base` | 预构建基础镜像 (Rust + dioxus-cli + wasm 工具链) |
| `docker-compose.yml` | Docker Compose 配置 |
| `nginx.conf` | Nginx 配置 (SPA、WASM、PWA 支持) |
| `.dockerignore` | Docker 构建忽略文件 |
| `docker-deploy.sh` | 本地构建/运行脚本 |
| `docker-push.sh` | 推送镜像到 VPS 脚本 |
| `docker-cicd.sh` | 完整 CI/CD 流程脚本 |
| `.github/workflows/deploy.yml` | GitHub Actions 自动部署 |

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

### 方法 1: GitHub Actions 自动部署 (推荐)

推送到 `main` 分支后自动构建并部署到 VPS。

#### 配置步骤

1. **配置 GitHub Secrets** (Settings → Secrets and variables → Actions):

| Secret 名称 | 说明 | 示例 |
|------------|------|------|
| `VPS_HOST` | VPS IP 或域名 | `192.168.1.100` |
| `VPS_USER` | SSH 用户名 | `devuser` |
| `VPS_PORT` | SSH 端口 | `22` |
| `VPS_SSH_KEY` | SSH 私钥 | `-----BEGIN OPENSSH...` |
| `GHCR_TOKEN` | GitHub PAT (read:packages) | `ghp_xxxx` |

2. **生成 SSH 密钥** (如果没有):
```bash
ssh-keygen -t ed25519 -C "github-actions"
# 将公钥添加到 VPS 的 ~/.ssh/authorized_keys
# 将私钥内容复制到 VPS_SSH_KEY secret
```

3. **创建 GitHub PAT**:
   - GitHub → Settings → Developer settings → Personal access tokens
   - 权限: `read:packages`
   - 复制 token 到 `GHCR_TOKEN` secret

4. **VPS 首次配置**:
```bash
# 确保 Docker 已安装
docker --version

# 测试 ghcr.io 登录
echo YOUR_GHCR_TOKEN | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
```

#### 工作流程
```
Push to main → Build Image → Push to ghcr.io → SSH Deploy to VPS
```

### 方法 2: 本地 CI/CD 脚本

使用提供的脚本手动部署:

```bash
# 完整流程: 构建 → 推送 → 部署
./docker-cicd.sh

# 仅推送已构建的镜像
./docker-push.sh

# 推送并部署
./docker-push.sh deploy
```

配置脚本中的 VPS 信息:
```bash
VPS_USER="devuser"
VPS_HOST="your-vps-ip"
VPS_PORT="22"
VPS_DEPLOY_PATH="/home/devuser/docker_compose/ukeep"
```

### 方法 3: 直接在 VPS 上构建

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

### 方法 4: 本地构建后推送到 Registry

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
