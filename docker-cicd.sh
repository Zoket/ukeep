#!/bin/bash

# uKeep 完整CI/CD脚本
# 从构建镜像到推送部署的完整流程

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env.deploy"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 镜像配置
IMAGE_NAME="ukeep"
IMAGE_TAG="latest"
BASE_IMAGE_NAME="ukeep-base"
CONTAINER_NAME="ukeep"
TARGET_PLATFORM="linux/amd64"

# VPS 配置默认值
VPS_USER=""
VPS_HOST=""
VPS_PORT="22"
VPS_DEPLOY_PATH=""
VPS_SSH_KEY_PATH=""

# SSH 选项（在 load_env 后构建）
SSH_OPTS=""

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

build_ssh_opts() {
    SSH_OPTS="-o ConnectTimeout=10 -o PasswordAuthentication=no -o BatchMode=yes"
    if [[ -n "${VPS_SSH_KEY_PATH}" ]]; then
        if [[ ! -f "${VPS_SSH_KEY_PATH}" ]]; then
            print_error "SSH key not found: ${VPS_SSH_KEY_PATH}"
            exit 1
        fi
        SSH_OPTS="${SSH_OPTS} -i ${VPS_SSH_KEY_PATH}"
    fi
}

load_env() {
    if [[ ! -f "${ENV_FILE}" ]]; then
        print_error "Config file not found: ${ENV_FILE}"
        print_info "Create it from template: cp .env.deploy.example .env.deploy"
        exit 1
    fi
    source "${ENV_FILE}"

    if [[ -z "${VPS_HOST}" || -z "${VPS_USER}" ]]; then
        print_error "VPS_HOST and VPS_USER must be set in ${ENV_FILE}"
        exit 1
    fi
    build_ssh_opts
    print_info "Loaded config from ${ENV_FILE}"
}

check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running"
        exit 1
    fi
}

check_ssh() {
    print_info "Testing SSH connection..."
    if ! ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} "echo 'OK'" > /dev/null 2>&1; then
        print_error "Cannot connect to VPS. Check SSH key and network."
        exit 1
    fi
    print_info "SSH connection OK"
}

ensure_base_image() {
    if ! docker image inspect ${BASE_IMAGE_NAME}:${IMAGE_TAG} > /dev/null 2>&1; then
        print_warn "Base image not found, building..."
        docker build --platform ${TARGET_PLATFORM} -f Dockerfile.base -t ${BASE_IMAGE_NAME}:${IMAGE_TAG} .
    fi
}

build_image() {
    print_step "Building Docker image (platform: ${TARGET_PLATFORM})..."
    ensure_base_image
    docker build --platform ${TARGET_PLATFORM} -t ${IMAGE_NAME}:${IMAGE_TAG} .
    print_info "Build completed"
}

push_image() {
    local image_file="/tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar"
    local scp_opts="-o PasswordAuthentication=no -o BatchMode=yes"
    if [[ -n "${VPS_SSH_KEY_PATH}" ]]; then
        scp_opts="${scp_opts} -i ${VPS_SSH_KEY_PATH}"
    fi

    print_step "Pushing image to VPS..."
    docker save ${IMAGE_NAME}:${IMAGE_TAG} -o ${image_file}
    local size=$(du -h ${image_file} | cut -f1)
    print_info "Image size: ${size}"

    scp -P ${VPS_PORT} ${scp_opts} ${image_file} ${VPS_USER}@${VPS_HOST}:/tmp/
    ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} "docker load -i /tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar && rm /tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar"
    rm ${image_file}
    print_info "Push completed"
}

deploy_on_vps() {
    print_step "Deploying on VPS..."
    ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} << EOF
        cd ${VPS_DEPLOY_PATH}
        docker stop ${CONTAINER_NAME} 2>/dev/null || true
        docker rm ${CONTAINER_NAME} 2>/dev/null || true
        docker run -d --name ${CONTAINER_NAME} -p 80:80 --restart unless-stopped ${IMAGE_NAME}:${IMAGE_TAG}
EOF
    print_info "Deployment completed"
}

verify_deployment() {
    print_step "Verifying deployment..."
    sleep 3
    ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} << EOF
        if docker ps | grep -q ${CONTAINER_NAME}; then
            echo "Container is running"
            docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep ${CONTAINER_NAME}
        else
            echo "Container failed to start"
            docker logs ${CONTAINER_NAME} --tail 20
            exit 1
        fi
EOF
}

show_help() {
    cat << EOF
uKeep 完整CI/CD脚本

用法: $0 [命令]

命令:
    all         完整流程: 构建 → 推送 → 部署 → 验证 (默认)
    build       仅构建镜像
    push        仅推送镜像
    deploy      仅部署 (需已推送镜像)
    verify      验证部署状态
    help        显示帮助

配置文件: .env.deploy (从 .env.deploy.example 复制)

示例:
    $0              # 执行完整流程
    $0 build        # 仅构建
    $0 push         # 仅推送

EOF
}

main() {
    local command=${1:-all}
    local start_time=$(date +%s)

    echo ""
    echo "=========================================="
    echo "  uKeep CI/CD Pipeline"
    echo "=========================================="
    echo ""

    case $command in
        all)
            load_env
            check_docker
            check_ssh
            build_image
            push_image
            deploy_on_vps
            verify_deployment
            ;;
        build)
            check_docker
            build_image
            ;;
        push)
            load_env
            check_docker
            check_ssh
            push_image
            ;;
        deploy)
            load_env
            check_ssh
            deploy_on_vps
            verify_deployment
            ;;
        verify)
            load_env
            check_ssh
            verify_deployment
            ;;
        help|--help|-h)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo ""
    print_info "Pipeline completed in ${duration}s"
}

main "$@"
