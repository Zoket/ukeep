# Docker 部署指南

本文档说明当前仓库中 Docker 相关文件的新目录结构，以及本地构建、Compose 和 VPS 部署方式。

## 目录说明

Docker 相关文件已统一放到 `docker/` 目录:

| 文件 | 说明 |
|------|------|
| `docker/Dockerfile` | 多阶段构建 (Tailwind → Rust/WASM → Nginx) |
| `docker/Dockerfile.base` | 预构建基础镜像 (Rust + dioxus-cli + wasm 工具链) |
| `docker/docker-compose.yml` | Docker Compose 配置 |
| `docker/nginx.conf` | Nginx 配置 (SPA、WASM、PWA 支持) |
| `docker/docker-deploy.sh` | 本地构建/运行脚本 |
| `docker/docker-push.sh` | 推送镜像到 VPS 脚本 |
| `docker/docker-cicd.sh` | 完整 CI/CD 流程脚本 |
| `docker/.env.deploy.example` | VPS 部署配置模板 |
| `.dockerignore` | Docker 构建忽略文件，保留在仓库根目录供根上下文构建使用 |
| `.github/workflows/deploy.yml` | GitHub Actions 自动部署 |

## 本地构建和测试

### 快速开始

```bash
# 一键构建并运行
./docker/docker-deploy.sh

# 或者分步执行
./docker/docker-deploy.sh build
./docker/docker-deploy.sh run
./docker/docker-deploy.sh logs
./docker/docker-deploy.sh restart
./docker/docker-deploy.sh clean
```

### 手动构建

先构建基础镜像:

```bash
docker build -f docker/Dockerfile.base -t ukeep-base:latest .
```

再构建主镜像:

```bash
docker build -f docker/Dockerfile -t ukeep:latest .
```

运行容器:

```bash
docker run -d -p 8080:80 --name ukeep ukeep:latest
```

访问 `http://localhost:8080` 查看应用。

### 使用 Docker Compose

```bash
docker compose -f docker/docker-compose.yml up -d
```

## VPS 部署

### 方法 1: GitHub Actions 自动部署

推送到 `main` 分支后自动构建并部署到 VPS。

配置 GitHub Secrets:

| Secret 名称 | 说明 | 示例 |
|------------|------|------|
| `VPS_HOST` | VPS IP 或域名 | `192.168.1.100` |
| `VPS_USER` | SSH 用户名 | `devuser` |
| `VPS_PORT` | SSH 端口 | `22` |
| `VPS_SSH_KEY` | SSH 私钥 | `-----BEGIN OPENSSH...` |
| `GHCR_TOKEN` | GitHub PAT (read:packages) | `ghp_xxxx` |

首次配置 VPS:

```bash
docker --version
echo YOUR_GHCR_TOKEN | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
```

### 方法 2: 本地 CI/CD 脚本

```bash
# 完整流程: 构建 → 推送 → 部署
./docker/docker-cicd.sh

# 仅推送已构建的镜像
./docker/docker-push.sh

# 推送并部署
./docker/docker-push.sh deploy
```

部署配置文件:

```bash
cp docker/.env.deploy.example docker/.env.deploy
```

脚本默认读取 `docker/.env.deploy`。如果本地还保留旧的根目录 `.env.deploy`，脚本也会兼容读取，但建议迁移到 `docker/` 下。

### 方法 3: 直接在 VPS 上构建

```bash
rsync -avz --exclude 'target' --exclude 'node_modules' ./ user@your-vps:/path/to/ukeep/
ssh user@your-vps
cd /path/to/ukeep
docker compose -f docker/docker-compose.yml up -d
```

### 方法 4: 本地构建后推送到 Registry

```bash
docker build -f docker/Dockerfile -t your-registry/ukeep:latest .
docker push your-registry/ukeep:latest
docker pull your-registry/ukeep:latest
docker run -d -p 80:80 --name ukeep your-registry/ukeep:latest
```

## 配置说明

### 端口映射

默认映射到 `8080:80`，可以在 `docker/docker-compose.yml` 中修改:

```yaml
ports:
  - "80:80"
```

### Nginx 配置

`docker/nginx.conf` 包含以下特性:

- SPA 路由回退到 `index.html`
- WASM MIME 类型配置
- PWA manifest 和 Service Worker 处理
- Gzip 压缩
- 静态资源缓存
- 安全头
- `/health` 健康检查端点

## 故障排查

查看日志:

```bash
docker logs ukeep
```

进入容器:

```bash
docker exec -it ukeep sh
```

检查构建产物:

```bash
docker exec -it ukeep ls -la /usr/share/nginx/html
```

## 常见问题

### 构建失败: `Failed to strip binary`

原因: 构建环境缺少 `wasm-strip`。

解决方案: `docker/Dockerfile.base` 已安装 `binaryen`，包含 `wasm-strip` 和 `wasm-opt`。

### 构建时间过长

可优先使用 Docker 缓存，并启用 BuildKit:

```bash
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile .
```

### 镜像体积较大

当前已经使用多阶段构建和 `nginx:alpine`，最终镜像通常只包含静态产物和 Nginx 运行环境。
