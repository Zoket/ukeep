#!/bin/bash

# uKeep Docker 构建和部署脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
IMAGE_NAME="ukeep"
IMAGE_TAG="latest"
BASE_IMAGE_NAME="ukeep-base"
CONTAINER_NAME="ukeep"
PORT="8080"

# 函数：打印带颜色的消息
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 函数：检查 Docker 是否运行
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker first."
        exit 1
    fi
    print_info "Docker is running"
}

# 函数：检查并构建 base 镜像
ensure_base_image() {
    if ! docker image inspect ${BASE_IMAGE_NAME}:${IMAGE_TAG} > /dev/null 2>&1; then
        print_warn "Base image not found, building it first..."
        docker build -f Dockerfile.base -t ${BASE_IMAGE_NAME}:${IMAGE_TAG} .
        print_info "Base image built successfully"
    else
        print_info "Base image exists: ${BASE_IMAGE_NAME}:${IMAGE_TAG}"
    fi
}

# 函数：构建 base 镜像
build_base_image() {
    print_info "Building base image: ${BASE_IMAGE_NAME}:${IMAGE_TAG}"
    docker build -f Dockerfile.base -t ${BASE_IMAGE_NAME}:${IMAGE_TAG} .
    print_info "Base image built successfully"
}

# 函数：构建镜像
build_image() {
    ensure_base_image
    print_info "Building Docker image: ${IMAGE_NAME}:${IMAGE_TAG}"
    docker build -t ${IMAGE_NAME}:${IMAGE_TAG} .
    print_info "Build completed successfully"
}

# 函数：停止并删除旧容器
stop_container() {
    if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        print_info "Stopping and removing existing container: ${CONTAINER_NAME}"
        docker stop ${CONTAINER_NAME} > /dev/null 2>&1 || true
        docker rm ${CONTAINER_NAME} > /dev/null 2>&1 || true
    fi
}

# 函数：运行容器
run_container() {
    print_info "Starting container: ${CONTAINER_NAME}"
    docker run -d \
        --name ${CONTAINER_NAME} \
        -p ${PORT}:80 \
        --restart unless-stopped \
        ${IMAGE_NAME}:${IMAGE_TAG}

    print_info "Container started successfully"
    print_info "Application is available at: http://localhost:${PORT}"
}

# 函数：查看日志
show_logs() {
    print_info "Showing container logs (Ctrl+C to exit)"
    docker logs -f ${CONTAINER_NAME}
}

# 函数：显示帮助
show_help() {
    cat << EOF
uKeep Docker 构建和部署脚本

用法: $0 [命令]

命令:
    build       构建 Docker 镜像
    build-base  构建/重建 base 镜像 (包含所有构建工具)
    run         运行容器
    restart     重启容器
    stop        停止容器
    logs        查看容器日志
    clean       清理容器和镜像
    deploy      构建并运行 (默认)
    help        显示此帮助信息

示例:
    $0              # 构建并运行
    $0 build        # 仅构建镜像
    $0 build-base   # 重建 base 镜像
    $0 logs         # 查看日志

EOF
}

# 主逻辑
main() {
    local command=${1:-deploy}

    case $command in
        build)
            check_docker
            build_image
            ;;
        build-base)
            check_docker
            build_base_image
            ;;
        run)
            check_docker
            stop_container
            run_container
            ;;
        restart)
            check_docker
            stop_container
            run_container
            ;;
        stop)
            check_docker
            stop_container
            print_info "Container stopped"
            ;;
        logs)
            check_docker
            show_logs
            ;;
        clean)
            check_docker
            stop_container
            print_info "Removing image: ${IMAGE_NAME}:${IMAGE_TAG}"
            docker rmi ${IMAGE_NAME}:${IMAGE_TAG} > /dev/null 2>&1 || true
            print_info "Cleanup completed"
            ;;
        deploy)
            check_docker
            build_image
            stop_container
            run_container
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
